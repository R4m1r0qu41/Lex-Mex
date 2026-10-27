import test from 'node:test';
import assert from 'node:assert/strict';
import {DatabaseSync} from 'node:sqlite';
import {sqliteAdapter,MAX_DEPTH} from './query-sqlite.mjs';
import {handlePublicQuery} from './query-http.mjs';
import {hash,normalize,checkResponse,VERSION} from './query-contract.mjs';

// These tests need a projection built by `lex-mex db project`; the database is not committed.
const path=process.env.LEX_MEX_QUERY_DB;
const skip=path?false:'set LEX_MEX_QUERY_DB to a projection corpus.sqlite';
const db=path?new DatabaseSync(path,{readOnly:true}):null;
const adapter=path?sqliteAdapter(path):null;
const api=path?`/lex-mex/api/r/${adapter.release.id}/`:'';
const get=(p,method='GET')=>handlePublicQuery(new Request('https://example.test'+p,{method}),adapter);
const json=async r=>JSON.parse(await r.text());
const cursorOf=v=>Buffer.from(JSON.stringify(v)).toString('base64url');
async function all(query,instrument=null){const out=[];let cursor=null;do{const r=await adapter.search({query,instrument,limit:10,cursor});out.push(...r.items);cursor=r.next_cursor;}while(cursor);return out;}

test('lookup returns the stored text byte for byte, with its digest, for a sample of provisions',{skip},async()=>{
  const rows=db.prepare("SELECT p.id,p.text,p.label,p.provision_type,i.slug FROM provisions p JOIN instruments i ON i.id=p.instrument_id WHERE p.row_id%337=0").all().filter(r=>/:(article|transitory|annex):[a-z0-9-]{1,80}$/.test(r.id));
  assert.ok(rows.length>150);
  for(const r of rows){
    const m=/:(article|transitory|annex):([a-z0-9-]+)$/.exec(r.id);
    const p=await adapter.lookup({slug:r.slug,type:m[1],number:m[2]});
    assert.equal(p.id,r.id);assert.equal(p.text,r.text);assert.equal(p.text_sha256,hash(Buffer.from(r.text)));
    assert.equal(p.label,r.label);assert.equal(p.provision_type,r.provision_type);assert.equal(p.number,m[2]);
    checkResponse({schema_version:VERSION,release:adapter.release,result:{kind:'article',data:{provision:p}}});
  }
});

test('the public number is the URL slug even where the stored display number has spaces and capitals',{skip},async()=>{
  // golden G8: stored number "78 Terdecies"
  const stored=db.prepare("SELECT number FROM provisions WHERE id='urn:lex-mx:federal:statute:lac:article:78-terdecies'").get();
  assert.equal(stored.number,'78 Terdecies');
  const p=await adapter.lookup({slug:'lac',type:'article',number:'78-terdecies'});
  assert.equal(p.number,'78-terdecies');assert.equal(p.label,'Artículo 78 Terdecies');
  assert.equal(await adapter.lookup({slug:'lac',type:'article',number:'78 Terdecies'}),null);
});

test('missing provisions and unknown instruments are null, never substituted',{skip},async()=>{
  assert.equal(await adapter.lookup({slug:'lac',type:'article',number:'999999'}),null);
  assert.equal(await adapter.lookup({slug:'no-such-instrument',type:'article',number:'1'}),null);
  const r=await get(`${api}mx/lac/article/999999`);assert.equal(r.status,404);assert.equal((await json(r)).result.data.code,'not_found');
});

test('golden G6 separate article and the retired LFD heading are served as standalone provisions',{skip},async()=>{
  assert.ok(await adapter.lookup({slug:'lgtoc',type:'article',number:'228-b'}));
  const p=await adapter.lookup({slug:'lfd',type:'article',number:'257-sextus'});
  assert.ok(p);assert.match(p.text,/Se deroga/);
});

test('search membership equals an independent full-corpus tokenization (golden G7: 2,134 hits)',{skip},async()=>{
  const want=new Set();
  for(const r of db.prepare('SELECT id,label,heading_title,text FROM provisions').iterate()){
    const t=normalize(r.label+' '+r.heading_title+' '+r.text).split(/[^\p{L}\p{N}]+/u);
    if(t.includes('obligacion'))want.add(r.id);
  }
  assert.equal(want.size,2134);
  const fts=new Set(db.prepare("SELECT p.id FROM provisions_fts f JOIN provisions p ON p.row_id=f.rowid WHERE provisions_fts MATCH 'obligacion'").all().map(r=>r.id));
  assert.deepEqual([...fts].sort(),[...want].sort());
  // adapter pages reach MAX_DEPTH hits, all members of that set
  const got=await all('obligación');
  assert.equal(got.length,MAX_DEPTH);
  assert.ok(got.every(h=>want.has(h.id)));
  assert.equal(new Set(got.map(h=>h.id)).size,MAX_DEPTH);
});

test('accent and case variants return the same ordered hits (golden G7)',{skip},async()=>{
  const a=(await adapter.search({query:'obligación',instrument:null,limit:10,cursor:null})).items.map(h=>h.id);
  for(const v of ['obligacion','OBLIGACIÓN','Obligación ']){
    assert.deepEqual((await adapter.search({query:v,instrument:null,limit:10,cursor:null})).items.map(h=>h.id),a,v);
  }
});

test('a multi-word query is an adjacent phrase and partial words do not match',{skip},async()=>{
  const hits=(await all('contrato de seguro','lscs')).slice(0,30);assert.ok(hits.length>5);
  for(const h of hits.slice(0,10)){
    const p=await adapter.lookup({slug:h.instrument.slug,type:h.provision_type,number:h.number});
    const flat=normalize(p.label+' '+p.text).split(/[^\p{L}\p{N}]+/u).join(' ');
    assert.ok(flat.includes('contrato de seguro'),h.id);
  }
  const partial=await all('segur','lscs');
  for(const h of partial.slice(0,20)){
    const p=await adapter.lookup({slug:h.instrument.slug,type:h.provision_type,number:h.number});
    assert.ok(normalize(p.text+' '+p.label).split(/[^\p{L}\p{N}]+/u).includes('segur'));
  }
});

test('instrument filter restricts hits to that instrument and an unknown filter is empty',{skip},async()=>{
  const hits=await all('contrato','lscs');assert.ok(hits.length>0&&hits.every(h=>h.instrument.slug==='lscs'));
  const none=await adapter.search({query:'contrato',instrument:'no-such-instrument',limit:10,cursor:null});
  assert.deepEqual(none.items,[]);assert.equal(none.next_cursor,null);
});

test('pagination is stable, release- and query-bound, capped, and ends with a null cursor',{skip},async()=>{
  const first=await adapter.search({query:'ley',instrument:null,limit:10,cursor:null});
  const second=await adapter.search({query:'ley',instrument:null,limit:10,cursor:first.next_cursor});
  assert.equal(new Set([...first.items,...second.items].map(h=>h.id)).size,20);
  const big=await all('ley');assert.equal(big.length,MAX_DEPTH);
  assert.deepEqual(big.slice(0,10).map(h=>h.id),first.items.map(h=>h.id));
  await assert.rejects(()=>adapter.search({query:'otra',instrument:null,limit:10,cursor:first.next_cursor}),/Invalid cursor/);
  const scope=hash(JSON.stringify([adapter.release.id,'search','ley',null]));
  await assert.rejects(()=>adapter.search({query:'ley',instrument:null,limit:10,cursor:cursorOf({scope,offset:MAX_DEPTH})}),/Invalid cursor/);
  const rare=db.prepare("SELECT count(*) n FROM provisions_fts WHERE provisions_fts MATCH 'terdecies'").get().n;
  assert.ok(rare>0&&rare<MAX_DEPTH);assert.equal((await all('terdecies')).length,rare);
});

test('every page of the catalog together is the instrument table',{skip},async()=>{
  const out=[];let cursor=null;do{const r=await adapter.catalog({limit:10,cursor});out.push(...r.items);cursor=r.next_cursor;}while(cursor);
  const want=db.prepare('SELECT slug FROM instruments ORDER BY slug').all().map(r=>r.slug);
  assert.deepEqual(out.map(i=>i.slug),want);assert.equal(want.length,468);
});

test('malformed requests through the transport are client errors, not service failures',{skip},async()=>{
  const bad=[`${api}instruments?cursor=${cursorOf(null)}`,`${api}instruments?cursor=${cursorOf([1])}`,`${api}instruments?cursor=${cursorOf('x')}`,
    `${api}search?q=cuentas&cursor=${cursorOf({scope:'x',offset:1})}`,`${api}search?q=%CC%81`,`${api}search?q=%20`,`${api}search?q=${'a'.repeat(257)}`,
    `${api}search?q=${Array.from({length:13},(_,i)=>'palabra'+i).join('%20')}`,`${api}search?q=ley&limit=0`,`${api}search?q=ley&limit=11`,
    `${api}search?q=ley&unknown=1`,`${api}search?q=ley&q=otra`,`${api}instruments?cursor=!!!`];
  for(const p of bad){const r=await get(p);assert.equal(r.status,400,p);assert.equal((await json(r)).result.data.code,'bad_request',p);}
  assert.equal((await get(`${api}search?q=ley`,'POST')).status,405);
  assert.equal((await get(`${api}mx/..%2f..%2fetc/article/1`)).status,404);
});

test('responses stay within the transport budgets and never expose stored extras',{skip},async()=>{
  for(const p of [`${api}instruments?limit=10`,`${api}search?q=ley&limit=10`,`${api}search?q=obligaci%C3%B3n&limit=10`,`${api}mx/lscs?limit=10`]){
    const r=await get(p);assert.equal(r.status,200,p);const body=await r.text();assert.ok(Buffer.byteLength(body)<=16384,p);
    assert.doesNotMatch(body,/extras|operator_note|payload|input_path/);
  }
  const big=db.prepare("SELECT p.id,i.slug,p.provision_type t,length(CAST(p.text AS BLOB)) n FROM provisions p JOIN instruments i ON i.id=p.instrument_id ORDER BY n DESC LIMIT 1").get();
  const m=/:([a-z0-9-]+)$/.exec(big.id);const r=await get(`${api}mx/${big.slug}/${big.t}/${m[1]}`);
  const body=await json(r);
  if(big.n>2*1024*1024-4096)assert.equal(body.result.data.code,'response_too_large');else{assert.equal(r.status,200);assert.equal(body.result.data.provision.text.length>0,true);}
});

test('historical and unavailable citations keep their status and have no live target (golden G3)',{skip},async()=>{
  const r=await adapter.references({id:'urn:lex-mx:federal:statute:lrsic:transitory:quinto',limit:10,cursor:null});
  const gone=r.items.filter(e=>e.resolution_status==='historical_target_unavailable');
  assert.equal(gone.length,4);assert.ok(gone.every(e=>e.target===null));
  checkResponse({schema_version:VERSION,release:adapter.release,result:{kind:'references',data:r}});
  const live=db.prepare("SELECT source_provision_id,target_provision_id FROM reference_edges WHERE resolution_status='resolved' AND rowid%401=0").all();
  assert.ok(live.length>50);
  for(const e of live){
    const refs=await adapter.references({id:e.source_provision_id,limit:10,cursor:null});
    const hit=refs.items.find(x=>x.target_id===e.target_provision_id);
    if(hit){assert.ok(hit.target);assert.ok(await adapter.lookup({slug:hit.target.slug,type:hit.target.provision_type,number:hit.target.number}),e.target_provision_id);}
  }
});

test('edge counts per source equal the stored edges for a sample',{skip},async()=>{
  const rows=db.prepare("SELECT s.id,(SELECT count(*) FROM reference_edges e WHERE e.source_provision_id=s.id) n FROM (SELECT DISTINCT source_provision_id id FROM reference_edges WHERE rowid%977=0) s").all();
  assert.ok(rows.length>20);
  for(const r of rows){
    const out=[];let cursor=null;do{const x=await adapter.references({id:r.id,limit:10,cursor});out.push(...x.items);cursor=x.next_cursor;}while(cursor);
    assert.equal(out.length,r.n,r.id);
  }
});

test('provisions whose identifiers are not URL-safe are counted in the coverage notice and never break a page',{skip},async()=>{
  const want=db.prepare('SELECT id FROM provisions').all().map(r=>r.id).filter(id=>!/:(article|transitory|annex):[a-z0-9-]{1,80}$/.test(id));
  assert.equal(want.length,68);
  const cov=await adapter.coverage();assert.match(cov.notice,/68 disposiciones/);
  // an instrument that has such rows pages through its index without error and without them
  const slug=db.prepare("SELECT i.slug FROM provisions p JOIN instruments i ON i.id=p.instrument_id WHERE p.id=?").get(want[0]).slug;
  const out=[];let cursor=null;do{const r=await adapter.index({slug,limit:10,cursor});out.push(...r.items);cursor=r.next_cursor;}while(cursor&&out.length<MAX_DEPTH);
  assert.ok(out.length>0);assert.ok(out.every(h=>!want.includes(h.id)));
  const total=db.prepare("SELECT count(*) n FROM provisions p JOIN instruments i ON i.id=p.instrument_id WHERE i.slug=?").get(slug).n;
  assert.equal(out.length+want.filter(id=>id.startsWith(db.prepare('SELECT id FROM instruments WHERE slug=?').get(slug).id+':')).length,total);
  checkResponse({schema_version:VERSION,release:adapter.release,result:{kind:'coverage',data:cov}});
});
