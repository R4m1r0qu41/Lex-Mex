import {readFile,writeFile,mkdir,rename,rm,mkdtemp,access} from 'node:fs/promises';
import {dirname,resolve,join,sep} from 'node:path';
import {createHash} from 'node:crypto';
import {fileURLToPath,pathToFileURL} from 'node:url';
import {safeRelative,releaseId} from './reader-core.mjs';
const sha=bytes=>createHash('sha256').update(bytes).digest('hex');
const assets=['index.html','reader.css','reader.js','reader-core.mjs','lex-mex.svg'];
const allowed=/^(?:index\.md|coverage\.json|sources\/[a-f0-9]{64}\.pdf|mx\/[a-z0-9-]+\/(?:index\.md|instrument\.json|provision-[a-f0-9]{64}\.(?:md|txt)))$/;
export async function packageReader(input,output) {
  input=resolve(input);output=resolve(output);
  if(output===input||output.startsWith(input+sep))throw new Error('Publication output must be outside its release input.');
  try{await access(output);throw new Error('Publication output already exists; refusing overwrite.');}catch(error){if(error.code!=='ENOENT')throw error;}
  const latest=JSON.parse(await readFile(join(input,'latest.json'),'utf8'));const id=releaseId(latest.release_id);
  const relative=`releases/${id}/`;const manifestBytes=await readFile(join(input,relative,'manifest.json'));
  const manifest=JSON.parse(manifestBytes);if(manifest.schema_version!=='public-release-v1'||!Array.isArray(manifest.files))throw new Error('Not a public release manifest.');if(manifest.release_id!==id)throw new Error('Manifest release mismatch.');
  const payload={...manifest};delete payload.release_id;
  if(sha(Buffer.from(JSON.stringify(payload,null,2)+'\n'))!==id)throw new Error('Manifest content ID mismatch.');
  if(!/^\/(?:[a-zA-Z0-9_-]+\/)*$/.test(manifest.base_path))throw new Error('Invalid public base path.');
  const content=new Map();content.set('latest.json',Buffer.from(JSON.stringify(latest,null,2)+'\n'));
  content.set('llms.txt',await readFile(join(input,'llms.txt')));content.set(relative+'manifest.json',manifestBytes);
  for(const file of manifest.files){
    const path=safeRelative(file.path);if(!allowed.test(path))throw new Error('Non-public artifact in release manifest.');
    const bytes=await readFile(join(input,relative,path));if(sha(bytes)!==file.sha256||bytes.length!==file.bytes)throw new Error(`Release file integrity failed: ${path}`);
    content.set(relative+path,bytes);
  }
  const source=dirname(fileURLToPath(import.meta.url));
  for(const asset of assets){let bytes=await readFile(join(source,asset));if(asset==='index.html')bytes=Buffer.from(bytes.toString().replace('content="/lex-mex/"',`content="${manifest.base_path}"`));content.set(asset,bytes);}
  const descriptor={schema_version:'reader-publication-v1',release_id:id,corpus_commit:manifest.corpus_commit,base_path:manifest.base_path,manifest_sha256:sha(manifestBytes),files:[...content].sort(([a],[b])=>a.localeCompare(b)).map(([path,bytes])=>({path,sha256:sha(bytes),bytes:bytes.length}))};
  const publicationId=sha(Buffer.from(JSON.stringify(descriptor,null,2)+'\n'));descriptor.publication_id=publicationId;
  content.set('publication.json',Buffer.from(JSON.stringify(descriptor,null,2)+'\n'));
  await mkdir(dirname(output),{recursive:true});const staging=await mkdtemp(join(dirname(output),'.lex-mex-publication-'));
  try{
    for(const [path,bytes] of content){const target=join(staging,safeRelative(path));await mkdir(dirname(target),{recursive:true});await writeFile(target,bytes);}
    await mkdir(output);await rename(staging,output);
  }catch(error){await rm(staging,{recursive:true,force:true});throw error;}
  return {publicationId,releaseId:id,output};
}
if(process.argv[1]&&import.meta.url===pathToFileURL(resolve(process.argv[1])).href){
  const args=process.argv.slice(2);const input=args[args.indexOf('--release')+1];const output=args[args.indexOf('--output')+1];
  if(!args.includes('--release')||!args.includes('--output')||!input||!output){console.error('Usage: node reader/package.mjs --release DIR --output NEW_DIR');process.exitCode=1;}
  else packageReader(input,output).then(result=>console.log(JSON.stringify(result,null,2))).catch(error=>{console.error(error.message);process.exitCode=1;});
}
