import test from 'node:test';
import assert from 'node:assert/strict';
import {readFile} from 'node:fs/promises';
import {once} from 'node:events';
import {fixtureAdapter} from './query-fixture.mjs';
import {hash,checkResponse,checkSource} from './query-contract.mjs';
import {handlePublicQuery} from './query-http.mjs';
import {queryServer} from './query-preview.mjs';
const fixture=JSON.parse(await readFile(new URL('./fixtures/public-query-v1.json',import.meta.url),'utf8'));
const release=fixture.release.id,root=`/lex-mex/r/${release}/`,api=`/lex-mex/api/r/${release}/`;
const adapter=()=>fixtureAdapter(fixture);
const get=(path,a=adapter(),method='GET')=>handlePublicQuery(new Request('https://example.test'+path,{method}),a);
const text=p=>p.text();
function unescape(s){return s.replace(/&#13;/g,'\r').replace(/&#39;/g,"'").replace(/&quot;/g,'"').replace(/&gt;/g,'>').replace(/&lt;/g,'<').replace(/&amp;/g,'&');}

test('versioned transport fixtures stay in sync with the Rust/schema acceptance inputs',async()=>{
 const cases=JSON.parse(await readFile(new URL('./fixtures/public-query-responses-v1.json',import.meta.url),'utf8'));
 for(const item of cases){const result=await get(api+item.path);assert.equal(result.status,item.status);assert.deepEqual(await result.json(),item.body);}
});

test('homepage discovery and browse reach exact article through ordinary HTML links',async()=>{
 const entry=await get('/lex-mex/agents/');assert.equal(entry.status,200);assert.match(entry.headers.get('content-type'),/text\/html; charset=utf-8/);const body=await entry.text();assert.ok(Buffer.byteLength(body)<=4096);assert.match(body,/DEMOSTRACIÓN SINTÉTICA/);assert.ok(!body.includes('<script'));
 const catalog=await text(await get(root+'instruments'));assert.ok(catalog.includes(root+'mx/norma-demo'));
 const index=await text(await get(root+'mx/norma-demo'));assert.ok(index.includes(root+'mx/norma-demo/article/1'));assert.ok(index.includes(root+'mx/norma-demo/transitory/1'));
 for(const type of ['article','transitory','annex']){
  const response=await get(root+`mx/norma-demo/${type}/1`);assert.equal(response.status,200);const html=await response.text();const exact=unescape(html.match(/<pre><span>([\s\S]*?)<\/span><\/pre>/)[1]);assert.equal(exact,fixture.provisions.find(p=>p.provision_type===type&&p.number==='1').text);
 }
 const metadata=await (await get('/lex-mex/current.json')).json();assert.ok(Buffer.byteLength(JSON.stringify(metadata))<=2048);assert.equal(metadata.release.id,release);
});

test('empty search is explicit and preserves the query in a usable search form',async()=>{
 const response=await get(root+'search?q=activos%20virtuales');assert.equal(response.status,200);const html=await response.text();assert.match(html,/No se encontraron coincidencias/);assert.match(html,/solo contiene textos sintéticos/);assert.ok(html.includes('value="activos virtuales"'));assert.ok(html.includes('<form'));
});

test('HTML and JSON preserve Unicode, literal markup, digest and qualified source metadata',async()=>{
 const path='mx/norma-demo/article/2-bis';const response=await (await get(api+path)).json();checkResponse(response);const p=response.result.data.provision;assert.equal(p.text,fixture.provisions[1].text);assert.equal(p.text_sha256,hash(p.text));assert.equal(p.source.recorded_reform_date,null);assert.equal(p.temporal_status,'unknown');
 const html=await text(await get(root+path));assert.ok(html.includes('&lt;script&gt;'));assert.ok(!html.includes('<script>'));assert.equal(unescape(html.match(/<pre><span>([\s\S]*?)<\/span><\/pre>/)[1]),p.text);
});

test('literal accent-insensitive fixture search is scoped and pages bind to query and release',async()=>{
 const f=structuredClone(fixture);const p=f.provisions[0];
 for(let i=3;i<30;i++)f.provisions.push({...p,id:p.id+'-'+i,number:String(i),label:'Artículo '+i});
 const a=fixtureAdapter(f);const first=await (await get(api+'search?q=cuentas&limit=3',a)).json();assert.equal(first.result.data.items.length,3);assert.ok(first.result.data.next_cursor);
 const second=await (await get(api+'search?q=cuentas&limit=3&cursor='+first.result.data.next_cursor,a)).json();assert.equal(second.result.data.items.length,3);assert.notDeepEqual(first.result.data.items,second.result.data.items);
 const wrong=await get(api+'search?q=otra&cursor='+first.result.data.next_cursor,a);assert.equal(wrong.status,400);
 const equivalent=await (await get(api+'search?q=obligacion',a)).json();assert.equal(equivalent.result.data.items[0].number,'2-bis');
 const excluded=await (await get(api+'search?q=cuentas&instrument=outside',a)).json();assert.deepEqual(excluded.result.data.items,[]);
});

test('invalid input, missing material, unavailable backend and stale release stay distinct',async()=>{
 for(const suffix of ['search?q=x&limit=1000','search?q=x&limit=-1','search?q=x&limit=2&limit=3','search?q=','search?q=x&extra=1','instruments?cursor=bad'])assert.equal((await get(root+suffix)).status,400,suffix);
 const absent=await get(api+'mx/norma-demo/article/999');assert.equal(absent.status,404);assert.match((await absent.json()).result.data.message,/cobertura no establecida/);
 assert.equal((await get('/lex-mex/r/'+'b'.repeat(64)+'/mx/norma-demo/article/1')).status,404);
 const broken=adapter();broken.lookup=async()=>{throw Error('private SQL and token=secret');};const failed=await get(api+'mx/norma-demo/article/1',broken);assert.equal(failed.status,503);assert.ok(!(await failed.text()).includes('secret'));
 assert.equal((await get(root+'coverage',adapter(),'POST')).status,405);
 const head=await get(root+'coverage',adapter(),'HEAD');assert.equal(await head.text(),'');assert.ok(Number(head.headers.get('content-length'))>0);
});

test('private fields, tampered article text and identity substitutions fail closed',async()=>{
 for(const mutate of [p=>({...p,legal_operator_review:{reviewer:'PRIVATE'}}),p=>({...p,text:'tampered'}),p=>({...p,number:'999'})]){
  const a=adapter();a.lookup=async()=>mutate(structuredClone(fixture.provisions[0]));const result=await get(api+'mx/norma-demo/article/1',a);assert.equal(result.status,503);assert.ok(!(await result.text()).includes('PRIVATE'));
 }
 const f=structuredClone(fixture);f.provisions[0].source.private_note='secret';assert.throws(()=>fixtureAdapter(f),/Invalid public/);
});

test('historical references remain unlinked and do not turn into live targets',async()=>{
 const json=await (await get(api+'mx/norma-demo/article/1/references')).json();checkResponse(json);assert.equal(json.result.data.items[1].target,null);
 const html=await text(await get(root+'mx/norma-demo/article/1/references'));assert.match(html,/historical_target_unavailable/);assert.ok(!html.includes('href="'+root+'mx/outside'));
 const a=adapter();a.references=async()=>({items:[{...fixture.references[fixture.provisions[0].id][1],target:{slug:'outside',provision_type:'article',number:'1'}}],next_cursor:null});assert.equal((await get(api+'mx/norma-demo/article/1/references',a)).status,503);
});

test('long bodies are never silently truncated and oversized responses fail explicitly',async()=>{
 const f=structuredClone(fixture),p=f.provisions[0];p.text='á & texto\n'.repeat(15000);p.text_sha256=hash(p.text);const a=fixtureAdapter(f);const complete=await (await get(api+'mx/norma-demo/article/1',a)).json();assert.equal(complete.result.data.provision.text,p.text);
 p.text='<'.repeat(600000);p.text_sha256=hash(p.text);const tooLarge=await get(root+'mx/norma-demo/article/1',fixtureAdapter(f));assert.equal(tooLarge.status,413);assert.match(await tooLarge.text(),/No se truncó/);
});

test('HTML protects carriage returns from parser newline normalization',async()=>{
 const f=structuredClone(fixture);f.provisions[0].text='\n  Primera\r\nSegunda\rLiteral &#13;';f.provisions[0].text_sha256=hash(f.provisions[0].text);
 const html=await text(await get(root+'mx/norma-demo/article/1',fixtureAdapter(f)));const body=html.match(/<pre><span>([\s\S]*?)<\/span><\/pre>/)[1];assert.ok(!body.includes('\r'));assert.equal(unescape(body),f.provisions[0].text);
});

test('actual local HTTP preserves HTML MIME, bytes, HEAD and real missing status',async t=>{
 const server=queryServer(adapter());server.listen(0,'127.0.0.1');await once(server,'listening');t.after(()=>new Promise(resolve=>server.close(resolve)));
 const origin='http://127.0.0.1:'+server.address().port;
 for(const path of ['/lex-mex/agents/',root+'search?q=cuentas',root+'mx/norma-demo/article/1']){const response=await fetch(origin+path);assert.equal(response.status,200);assert.match(response.headers.get('content-type'),/text\/html/);const bytes=await response.arrayBuffer();assert.equal(bytes.byteLength,Number(response.headers.get('content-length')));}
 assert.equal((await fetch(origin+root+'mx/norma-demo/article/999')).status,404);
});

const cursorOf=value=>Buffer.from(JSON.stringify(value)).toString('base64url');
test('a decoded cursor that is not an object is a client error, not a service failure',async()=>{
 for(const value of [null,1,'x',true,[],[1,2]]){
  for(const path of [`${api}instruments?cursor=${cursorOf(value)}`,`${api}search?q=cuentas&cursor=${cursorOf(value)}`,`${root}instruments?cursor=${cursorOf(value)}`]){
   const response=await get(path);assert.equal(response.status,400,path);
   if(path.startsWith(api))assert.equal(JSON.parse(await response.text()).result.data.code,'bad_request');
  }
 }
 assert.equal((await get(`${api}instruments?cursor=${cursorOf({scope:'x',offset:'1'})}`)).status,400);
});
test('a search that normalizes to nothing is rejected rather than ranked as an unrestricted query',async()=>{
 for(const q of ['%CC%81','%20%CC%81%20','%CC%81%CC%81']){
  const response=await get(`${api}search?q=${q}`);assert.equal(response.status,400,q);
  assert.equal(JSON.parse(await response.text()).result.data.code,'bad_request');
 }
 await assert.rejects(()=>adapter().search({query:'\u0301',instrument:null,limit:5,cursor:null}),/Invalid search/);
});
test('the source URL boundary accepts recorded http and https links and rejects credentials and other schemes',()=>{
 const source=url=>({official_url:url,publisher:'Oficial',retrieved_at:'2026-07-12',source_sha256:'a'.repeat(64),recorded_reform_date:null});
 assert.doesNotThrow(()=>checkSource(source('http://www.oag.salud.gob.mx/descargas/LV/56-31052021.pdf')));
 assert.doesNotThrow(()=>checkSource(source('https://www.diputados.gob.mx/LeyesBiblio/pdf/LBOGM.pdf')));
 for(const bad of ['https://user:pw@example.org/a.pdf','ftp://example.org/a.pdf','javascript:alert(1)','//example.org/a.pdf','not a url'])assert.throws(()=>checkSource(source(bad)),/Invalid public retrieval DTO/,bad);
});
