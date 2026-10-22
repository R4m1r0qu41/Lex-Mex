import {readFile,writeFile,mkdir,rename,lstat,realpath,open,rm} from 'node:fs/promises';
import {resolve,join,dirname,basename,sep} from 'node:path';
import {createHash,randomUUID} from 'node:crypto';
import {pathToFileURL} from 'node:url';
import {safeRelative,releaseId} from './reader-core.mjs';
const hash=b=>createHash('sha256').update(b).digest('hex');
const rootFiles=new Set(['index.html','reader.css','reader.js','reader-core.mjs','latest.json','llms.txt']);
const optionalRootFiles=new Set(['lex-mex.svg']);
async function regular(path){const s=await lstat(path);if(!s.isFile()||s.isSymbolicLink())throw Error(`Expected regular file: ${path}`);}
async function confined(root,path){const parts=path.split('/');let current=root;for(const part of parts){current=join(current,part);try{const s=await lstat(current);if(s.isSymbolicLink())throw Error(`Symlink in publication path: ${current}`);}catch(e){if(e.code!=='ENOENT')throw e;}}}
async function canonicalParent(path){try{return await realpath(path);}catch(e){if(e.code!=='ENOENT')throw e;return join(await canonicalParent(dirname(path)),basename(path));}}
export async function stagePublication(input,destination){
  input=await realpath(resolve(input));destination=resolve(destination);
  try{const s=await lstat(destination);if(s.isSymbolicLink()||!s.isDirectory())throw Error('Publication destination must be a real directory.');}catch(e){if(e.code!=='ENOENT')throw e;}
  destination=join(await canonicalParent(dirname(destination)),basename(destination));
  if(destination===input||destination.startsWith(input+sep)||input.startsWith(destination+sep))throw Error('Input and destination must be separate trees.');
  await regular(join(input,'publication.json'));
  const receiptBytes=await readFile(join(input,'publication.json'));const receipt=JSON.parse(receiptBytes);
  if(receipt.schema_version!=='reader-publication-v1'||!Array.isArray(receipt.files))throw Error('Invalid publication receipt.');
  const id=releaseId(receipt.release_id);const payload={...receipt};delete payload.publication_id;
  if(hash(Buffer.from(JSON.stringify(payload,null,2)+'\n'))!==receipt.publication_id)throw Error('Publication receipt digest mismatch.');
  const files=new Map();const prefix=`releases/${id}/`;
  for(const item of receipt.files){
    const path=safeRelative(item.path);
    if(files.has(path))throw Error('Duplicate publication path.');
    const local=path.startsWith(prefix)?path.slice(prefix.length):null;
    if(!rootFiles.has(path)&&!optionalRootFiles.has(path)&&!(local&&/^(?:manifest\.json|index\.md|coverage\.json|sources\/[a-f0-9]{64}\.pdf|mx\/[a-z0-9-]+\/(?:index\.md|instrument\.json|provision-[a-f0-9]{64}\.(?:md|txt)))$/.test(local)))throw Error(`Non-public path: ${path}`);
    await confined(input,path);await regular(join(input,path));const bytes=await readFile(join(input,path));
    if(hash(bytes)!==item.sha256||bytes.length!==item.bytes)throw Error(`Publication file mismatch: ${path}`);
    files.set(path,bytes);
  }
  for(const path of [...rootFiles,prefix+'manifest.json'])if(!files.has(path))throw Error(`Missing required publication file: ${path}`);
  const latest=JSON.parse(files.get('latest.json'));if(latest.release_id!==id||latest.manifest_path!==prefix+'manifest.json')throw Error('Latest pointer mismatch.');
  const manifest=JSON.parse(files.get(prefix+'manifest.json'));const mp={...manifest};delete mp.release_id;
  if(manifest.release_id!==id||hash(Buffer.from(JSON.stringify(mp,null,2)+'\n'))!==id||hash(files.get(prefix+'manifest.json'))!==receipt.manifest_sha256)throw Error('Release manifest mismatch.');
  for(const file of manifest.files){const bytes=files.get(prefix+safeRelative(file.path));if(!bytes||hash(bytes)!==file.sha256||bytes.length!==file.bytes)throw Error('Release file missing or inconsistent.');}
  // Check the full destination before any writes. Prior pinned releases remain intact.
  await confined('/',destination.slice(1));
  for(const [path,bytes] of files){await confined(destination,path);try{await regular(join(destination,path));if(path.startsWith('releases/')&&hash(await readFile(join(destination,path)))!==hash(bytes))throw Error(`Immutable release collision: ${path}`);}catch(e){if(e.code!=='ENOENT')throw e;}}
  await confined(destination,'publication.json');
  try{await regular(join(destination,'publication.json'));}catch(e){if(e.code!=='ENOENT')throw e;}
  await mkdir(destination,{recursive:true});const lock=join(destination,'.stage-lock');const handle=await open(lock,'wx');
  const temporary=[];
  async function atomic(path,bytes){const target=join(destination,path);await mkdir(dirname(target),{recursive:true});const temp=target+'.'+randomUUID()+'.tmp';temporary.push(temp);await writeFile(temp,bytes,{flag:'wx'});await rename(temp,target);}
  try{
    for(const [path,bytes] of files)if(path!=='latest.json')await atomic(path,bytes);
    await atomic('publication.json',receiptBytes);
    await atomic('latest.json',files.get('latest.json'));
  }finally{await handle.close();await rm(lock);for(const temp of temporary)await rm(temp,{force:true});}
  return {publicationId:receipt.publication_id,releaseId:id,destination};
}
if(process.argv[1]&&import.meta.url===pathToFileURL(resolve(process.argv[1])).href){
  const args=process.argv.slice(2);const value=key=>args[args.indexOf(key)+1];
  if(!args.includes('--publication')||!args.includes('--destination'))throw Error('Usage: stage.mjs --publication DIR --destination DIR');
  console.log(JSON.stringify(await stagePublication(value('--publication'),value('--destination')),null,2));
}
