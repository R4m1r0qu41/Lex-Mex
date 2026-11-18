import test from 'node:test';import assert from 'node:assert/strict';import {mkdtemp,mkdir,writeFile,readFile,rm} from 'node:fs/promises';import {join} from 'node:path';import {tmpdir} from 'node:os';import {createHash} from 'node:crypto';import {packageReader} from './package.mjs';import {stagePublication} from './stage.mjs';
const sha=data=>createHash('sha256').update(data).digest('hex');const encode=value=>JSON.stringify(value,null,2)+'\n';
async function fixture(root,path='index.md'){
 const bytes=Buffer.from('# Corpus\n');const payload={base_path:'/lex-mex/',committed_at:'2026-10-07',corpus_commit:'test',coverage:'selected_partial',files:[{path,sha256:sha(bytes),bytes:bytes.length}],instruments:[],schema_version:'public-release-v1'};
 const id=sha(encode(payload));const dir=join(root,'releases',id);await mkdir(join(dir,...path.split('/').slice(0,-1)),{recursive:true});await writeFile(join(dir,path),bytes);await writeFile(join(dir,'manifest.json'),encode({...payload,release_id:id}));await writeFile(join(root,'latest.json'),encode({release_id:id,manifest_path:`releases/${id}/manifest.json`}));await writeFile(join(root,'llms.txt'),'# Corpus\n');return {id,dir};
}
test('package contains only publication assets with reproducible descriptor and refuses overwrite',async t=>{
 const root=await mkdtemp(join(tmpdir(),'lex-package-'));t.after(()=>rm(root,{recursive:true,force:true}));const input=join(root,'input');await fixture(input);const first=await packageReader(input,join(root,'one'));const second=await packageReader(input,join(root,'two'));assert.equal(first.publicationId,second.publicationId);
 const descriptor=JSON.parse(await readFile(join(root,'one','publication.json'),'utf8'));assert.ok(descriptor.files.some(f=>f.path==='index.html'));assert.ok(!descriptor.files.some(f=>f.path.includes('serve.mjs')||f.path.includes('review-queue')));
 for(const file of descriptor.files)assert.equal(sha(await readFile(join(root,'one',file.path))),file.sha256);
 await assert.rejects(packageReader(input,join(root,'one')),/already exists/);
});
test('tampered release file is rejected without publication output',async t=>{
 const root=await mkdtemp(join(tmpdir(),'lex-tamper-'));t.after(()=>rm(root,{recursive:true,force:true}));const input=join(root,'input');const {dir}=await fixture(input);await writeFile(join(dir,'index.md'),'changed');await assert.rejects(packageReader(input,join(root,'out')),/integrity/);
});
test('private canonical artifacts cannot pass the public file allowlist',async t=>{
 const root=await mkdtemp(join(tmpdir(),'lex-private-'));t.after(()=>rm(root,{recursive:true,force:true}));const input=join(root,'input');await fixture(input,'review-queue.json');await assert.rejects(packageReader(input,join(root,'out')),/Non-public/);
});
test('publication never carries agent routes and keeps the release discovery file verbatim',async t=>{
 const root=await mkdtemp(join(tmpdir(),'lex-no-agents-'));t.after(()=>rm(root,{recursive:true,force:true}));const input=join(root,'input');await fixture(input);await packageReader(input,join(root,'out'));
 const descriptor=JSON.parse(await readFile(join(root,'out','publication.json'),'utf8'));
 assert.ok(!descriptor.files.some(f=>f.path.startsWith('agents/')||/query|preview|agent/.test(f.path)));assert.deepEqual(await readFile(join(root,'out','llms.txt')),await readFile(join(input,'llms.txt')));
 assert.ok(!(await readFile(join(root,'out','index.html'),'utf8')).includes('agents/'));
});
test('release manifests that list agent-style paths are rejected',async t=>{
 const root=await mkdtemp(join(tmpdir(),'lex-agent-path-'));t.after(()=>rm(root,{recursive:true,force:true}));const input=join(root,'input');await fixture(input,`agents/${'a'.repeat(64)}/catalog-1.html`);await assert.rejects(packageReader(input,join(root,'out')),/Non-public/);
});

test('explicit agent publication includes hashed HTML without modifying release input',async t=>{
 const root=await mkdtemp(join(tmpdir(),'lex-agents-opt-'));t.after(()=>rm(root,{recursive:true,force:true}));
 const input=join(root,'input');const {id,dir}=await fixture(input);const before=await readFile(join(dir,'manifest.json'));
 await packageReader(input,join(root,'out'),{includeAgentPages:true});
 const descriptor=JSON.parse(await readFile(join(root,'out','publication.json'),'utf8'));
 assert.ok(descriptor.files.some(f=>/^agents\/[a-f0-9]{64}\/catalog-1.html$/.test(f.path)));
 assert.deepEqual(await readFile(join(dir,'manifest.json')),before);assert.equal(descriptor.release_id,id);
 assert.ok((await readFile(join(root,'out','llms.txt'),'utf8')).includes('Acceso HTML'));
 await assert.rejects(stagePublication(join(root,'out'),join(root,'rejected')),/Non-public/);
 await stagePublication(join(root,'out'),join(root,'staged'),{includeAgentPages:true});
 assert.deepEqual(await readFile(join(root,'staged','agents/index.html')),await readFile(join(root,'out','agents/index.html')));
});

test('sitemap requires an explicit HTTPS origin and stages only on opt-in',async t=>{
 const root=await mkdtemp(join(tmpdir(),'lex-sitemap-'));t.after(()=>rm(root,{recursive:true,force:true}));const input=join(root,'input');await fixture(input);
 await assert.rejects(packageReader(input,join(root,'bad'),{includeAgentPages:true,publicOrigin:'https://example.org/private'}),/HTTPS origin/);
 await packageReader(input,join(root,'out'),{includeAgentPages:true,publicOrigin:'https://example.org'});
 const xml=await readFile(join(root,'out','agents/sitemap.xml'),'utf8');assert.ok(xml.includes('<urlset'));assert.ok(!xml.includes('<lastmod>'));
 await stagePublication(join(root,'out'),join(root,'staged'),{includeAgentPages:true});assert.equal(await readFile(join(root,'staged','agents/sitemap.xml'),'utf8'),xml);
});
