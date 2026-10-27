import test from 'node:test';
import assert from 'node:assert/strict';
import {mkdtemp,mkdir,writeFile,rm} from 'node:fs/promises';
import {tmpdir} from 'node:os';
import {join} from 'node:path';
import {hash} from './query-contract.mjs';
import {retainedAdapter} from './query-retained.mjs';
import {handlePublicQuery} from './query-http.mjs';
const encode=v=>JSON.stringify(v,null,2)+'\n';
async function setup(t,condition=null){
 const root=await mkdtemp(join(tmpdir(),'lex-retained-'));t.after(()=>rm(root,{recursive:true,force:true}));const files=new Map(),instruments=[];
 for(const slug of ['ifpe-dcg-2021','itf-dcg-2018','lic','lritf']){
  const id='urn:test:'+slug,text='Texto sintético de prueba: activos virtuales.',p={id:id+':article:30',number:'30',label:'Artículo 30',provision_type:'article',text,text_sha256:hash(text),text_path:'provision-'+hash(id)+'.txt',commencement_condition:condition,repeals:[],temporal_status:'unknown',review_status:'not_analyzed',source:{official_url:'https://example.invalid/source.pdf',publisher:'synthetic',retrieved_at:'2026-10-20',source_sha256:hash('source'),private_test_note:'NEVER_EXPOSE'}};
  const doc={id,slug,short_name:slug,official_title:'Norma de prueba '+slug,latest_reform_date:null,provisions:[p],references:[]};
  files.set(`mx/${slug}/instrument.json`,Buffer.from(encode(doc)));files.set(`mx/${slug}/${p.text_path}`,Buffer.from(text));instruments.push({id,slug,document_path:`mx/${slug}/instrument.json`});
 }
 const payload={schema_version:'public-release-v1',corpus_commit:'1'.repeat(40),instruments,files:[...files].map(([path,b])=>({path,bytes:b.length,sha256:hash(b)}))};const id=hash(encode(payload)),base=join(root,'releases',id);await mkdir(base,{recursive:true});
 for(const [path,b] of files){await mkdir(join(base,path,'..'),{recursive:true});await writeFile(join(base,path),b);}
 await writeFile(join(base,'manifest.json'),encode({...payload,release_id:id}));await writeFile(join(root,'latest.json'),encode({release_id:id}));return {root,base,id};
}
test('retained loader verifies public artifacts and returns real-mode bounded results without private extras',async t=>{
 const {root,id}=await setup(t);const a=await retainedAdapter(root);assert.equal(a.release.fixture,false);const response=await handlePublicQuery(new Request(`https://test/lex-mex/api/r/${id}/search?q=activos%20virtuales`),a);const result=await response.json();assert.equal(result.result.data.items.length,4);
 const p=await a.lookup({slug:'lritf',type:'article',number:'30'});assert.ok(!JSON.stringify(p).includes('NEVER_EXPOSE'));assert.match(a.notice,/Publicación retenida/);
 const page=await (await handlePublicQuery(new Request('https://test/lex-mex/agents/'),a)).text();assert.match(page,/PREVIEW LOCAL/);assert.ok(!page.includes('DEMOSTRACIÓN SINTÉTICA'));
});
test('tampered public artifact and unsupported effects stop the retained bridge',async t=>{
 const {root,base}=await setup(t);await writeFile(join(base,'mx/lritf/instrument.json'),'tampered');await assert.rejects(retainedAdapter(root),/integrity mismatch/);
 const other=await setup(t,{condition_status:'unverified'});await assert.rejects(retainedAdapter(other.root),/Unsupported retained legal qualification/);
});
