import test from 'node:test';import assert from 'node:assert/strict';import {handleRag} from './rag-http.mjs';import {loadFixture} from './query-fixture.mjs';
const token='a'.repeat(32);const request=(auth=true)=>new Request('https://example.test/lex-mex/api/answer',{method:'POST',headers:{'content-type':'application/json',...(auth?{authorization:'Bearer '+token}:{})},body:JSON.stringify({question:'obligación',queries:['obligación']})});
test('answer transport denies unauthenticated calls and requires model budget approval',async()=>{
 let calls=0;const options={adapter:await loadFixture(new URL('./fixtures/public-query-v1.json',import.meta.url)),bearerToken:token,authorizeBudget:async()=>false,generate:async()=>{calls++;}};
 assert.equal((await handleRag(request(false),options)).status,401);assert.equal((await handleRag(request(),options)).status,429);assert.equal(calls,0);
});
test('transport returns validated answer and evidence; invalid model citations fail closed',async()=>{
 const options={adapter:await loadFixture(new URL('./fixtures/public-query-v1.json',import.meta.url)),bearerToken:token,authorizeBudget:async()=>true,generate:async({evidence:b})=>({release_id:b.release.id,status:'insufficient_evidence',claims:[],limitations:['Synthetic test']})};
 const response=await handleRag(request(),options);assert.equal(response.status,200);const result=await response.json();assert.equal(result.answer.release_id,result.evidence.release.id);
 options.generate=async()=>({});assert.equal((await handleRag(request(),options)).status,502);
});

test('budget service failure returns unavailable without executing model',async()=>{
 let called=false;const options={adapter:await loadFixture(new URL('./fixtures/public-query-v1.json',import.meta.url)),bearerToken:token,authorizeBudget:async()=>{throw Error('offline');},generate:async()=>{called=true;}};
 assert.equal((await handleRag(request(),options)).status,503);assert.equal(called,false);
});
test('local HTTP server preserves anonymous reading and gates model execution',async t=>{
 const {queryServer}=await import('./query-preview.mjs');const {once}=await import('node:events');
 const a=await loadFixture(new URL('./fixtures/public-query-v1.json',import.meta.url));let calls=0;
 const server=queryServer(a,{rag:{bearerToken:token,authorizeBudget:async()=>true,generate:async({evidence:b})=>{calls++;return {release_id:b.release.id,status:'insufficient_evidence',claims:[],limitations:['Synthetic fixture only']};}}});
 server.listen(0,'127.0.0.1');await once(server,'listening');t.after(()=>new Promise(resolve=>server.close(resolve)));
 const base=`http://127.0.0.1:${server.address().port}`;
 assert.equal((await fetch(base+'/lex-mex/agents/')).status,200);
 const init={method:'POST',headers:{'content-type':'application/json'},body:JSON.stringify({question:'obligación',queries:['obligación']})};
 assert.equal((await fetch(base+'/lex-mex/api/answer',init)).status,401);assert.equal(calls,0);
 init.headers.authorization='Bearer '+token;const response=await fetch(base+'/lex-mex/api/answer',init);assert.equal(response.status,200);assert.equal(calls,1);
 const result=await response.json();assert.equal(result.answer.release_id,result.evidence.release.id);
});
