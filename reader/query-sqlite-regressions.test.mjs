import test from 'node:test';
import assert from 'node:assert/strict';
import {DatabaseSync} from 'node:sqlite';
import {sqliteAdapter,MAX_DEPTH} from './query-sqlite.mjs';
import {handlePublicQuery} from './query-http.mjs';
import {checkResponse,VERSION,hash} from './query-contract.mjs';
import {miniProjection,RELEASE} from './query-sqlite-fixture.mjs';

// Independent review findings R-002, R-003 and R-004 (2026-10-26). The fixture tests are portable; the real-database
// tests reproduce the exact cases the reviewer ran and are gated on LEX_MEX_QUERY_DB.
const cursorOf=v=>Buffer.from(JSON.stringify(v)).toString('base64url');
const api=`/lex-mex/api/r/${RELEASE}/`;
const walk=async(adapter,query,limits)=>{const out=[];const pages=[];let cursor=null,i=0;
  do{const limit=limits[i++%limits.length];const r=await adapter.search({query,instrument:null,limit,cursor});pages.push(r.items.length);out.push(...r.items);cursor=r.next_cursor;}while(cursor);
  return {out,pages};};

test('R-002: a resolved reference whose target has no public route keeps its canonical id and status, with a null target',async()=>{
  const adapter=sqliteAdapter(miniProjection());
  const r=await adapter.references({id:'urn:lex-mx:federal:statute:alfa:article:1',limit:10,cursor:null});
  assert.equal(r.items.length,3);
  const [routable,ordinal,historical]=r.items;
  assert.equal(routable.resolution_status,'resolved');assert.deepEqual(routable.target,{slug:'beta',provision_type:'article',number:'1'});
  assert.deepEqual(routable.qualifiers,[{type:'fraction',text:'fracción II'}]);
  assert.equal(ordinal.target_id,'urn:lex-mx:federal:statute:alfa:transitory:9º');   // exact canonical id, not normalized
  assert.equal(ordinal.resolution_status,'resolved');assert.equal(ordinal.target,null);
  assert.equal(ordinal.source_span,'transitorio 9º');
  assert.equal(historical.resolution_status,'historical_target_unavailable');assert.equal(historical.target,null);
  checkResponse({schema_version:VERSION,release:adapter.release,result:{kind:'references',data:r}});
  const http=await handlePublicQuery(new Request(`https://t.invalid${api}mx/alfa/article/1/references`),adapter);
  assert.equal(http.status,200);assert.equal(JSON.parse(await http.text()).result.data.items.length,3);
  adapter.close();
});

test('R-002: a resolved target in an unknown instrument is still a service error, not silently nulled',async()=>{
  const path=miniProjection();const db=new DatabaseSync(path);
  db.prepare("INSERT INTO reference_edges VALUES('edge-9','urn:lex-mx:federal:statute:alfa:article:2','urn:lex-mx:federal:statute:nowhere','urn:lex-mx:federal:statute:nowhere:article:1','resolved',0,'x','/9','{}')").run();db.close();
  const adapter=sqliteAdapter(path);
  await assert.rejects(()=>adapter.references({id:'urn:lex-mx:federal:statute:alfa:article:2',limit:5,cursor:null}),/Unresolvable edge target/);
  adapter.close();
});

test('R-003: pagination ends exactly at the declared depth for any page size, with changing limits and at the last offset',async()=>{
  const adapter=sqliteAdapter(miniProjection({matches:40}),{maxDepth:25});
  for(const limits of [[3],[7],[9],[10],[3,7,9],[10,1,9,2]]){
    const {out,pages}=await walk(adapter,'comun',limits);
    assert.equal(out.length,25,JSON.stringify(limits));
    assert.equal(new Set(out.map(h=>h.id)).size,25);
    assert.ok(pages.every(n=>n>=1&&n<=10));
  }
  // limit 7: pages 7,7,7,4 (the last page is cut to the remaining depth)
  assert.deepEqual((await walk(adapter,'comun',[7])).pages,[7,7,7,4]);
  // the last permitted offset returns one item and no cursor, whatever the requested limit
  const scope=hash(JSON.stringify([RELEASE,'search','comun',null]));
  const last=await adapter.search({query:'comun',instrument:null,limit:10,cursor:cursorOf({scope,offset:24})});
  assert.equal(last.items.length,1);assert.equal(last.next_cursor,null);
  await assert.rejects(()=>adapter.search({query:'comun',instrument:null,limit:10,cursor:cursorOf({scope,offset:25})}),/Invalid cursor/);
  // results fewer than the depth still end with a null cursor
  const few=await walk(adapter,'beta',[3]);assert.ok(few.out.length<25);
  // the cap is the same ordered prefix whatever the page size
  const a=(await walk(adapter,'comun',[3])).out.map(h=>h.id),b=(await walk(adapter,'comun',[10])).out.map(h=>h.id);
  assert.deepEqual(a,b);
  adapter.close();
});

test('R-003: the depth option is validated and defaults to MAX_DEPTH',()=>{
  const path=miniProjection();
  for(const bad of [0,-1,1.5,MAX_DEPTH+1,'9'])assert.throws(()=>sqliteAdapter(path,{maxDepth:bad}),/Invalid depth/,String(bad));
  sqliteAdapter(path).close();
});

test('R-004: a malformed cursor is a client error even when the instrument filter is unknown',async()=>{
  const adapter=sqliteAdapter(miniProjection());
  for(const value of [null,1,'x',[],{scope:'x',offset:0}]){
    await assert.rejects(()=>adapter.search({query:'comun',instrument:'no-such-instrument',limit:5,cursor:cursorOf(value)}),/Invalid cursor/,JSON.stringify(value));
  }
  await assert.rejects(()=>adapter.search({query:'comun',instrument:'no-such-instrument',limit:0,cursor:null}),/Invalid limit/);
  const empty=await adapter.search({query:'comun',instrument:'no-such-instrument',limit:5,cursor:null});
  assert.deepEqual(empty.items,[]);assert.equal(empty.next_cursor,null);
  const http=await handlePublicQuery(new Request(`https://t.invalid${api}search?q=comun&instrument=no-such-instrument&cursor=${cursorOf(null)}`),adapter);
  assert.equal(http.status,400);
  adapter.close();
});

// ---- real database (the cases the reviewer reproduced)
const path=process.env.LEX_MEX_QUERY_DB;const skip=path?false:'set LEX_MEX_QUERY_DB to a projection corpus.sqlite';
test('R-002 on the real database: mx/lft/transitory/10/references answers 200 with the canonical 9º target and a null route',{skip},async()=>{
  const adapter=sqliteAdapter(path);const real=`/lex-mex/api/r/${adapter.release.id}/`;
  const r=await handlePublicQuery(new Request(`https://t.invalid${real}mx/lft/transitory/10/references`),adapter);
  assert.equal(r.status,200);
  const body=JSON.parse(await r.text());const unrouted=body.result.data.items.filter(e=>e.target===null&&e.resolution_status==='resolved');
  assert.ok(unrouted.some(e=>e.target_id==='urn:lex-mx:federal:statute:lft:transitory:9º'),JSON.stringify(body.result.data.items.map(e=>e.target_id)));
  adapter.close();
});

test('R-003 on the real database: limit 7 over "obligación" returns exactly 1,000 results',{skip},async()=>{
  const adapter=sqliteAdapter(path);
  const {out,pages}=await walk(adapter,'obligación',[7]);
  assert.equal(out.length,MAX_DEPTH);assert.equal(pages.length,143);assert.equal(pages.at(-1),6);
  assert.equal(new Set(out.map(h=>h.id)).size,MAX_DEPTH);adapter.close();
});

test('R-004 on the real database: search for an unknown instrument with a null cursor is 400',{skip},async()=>{
  const adapter=sqliteAdapter(path);const real=`/lex-mex/api/r/${adapter.release.id}/`;
  const r=await handlePublicQuery(new Request(`https://t.invalid${real}search?q=ley&instrument=no-such-instrument&cursor=bnVsbA`),adapter);
  assert.equal(r.status,400);adapter.close();
});
