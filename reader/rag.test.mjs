import test from 'node:test';import assert from 'node:assert/strict';import {loadFixture} from './query-fixture.mjs';import {retrieveEvidence} from './rag-evidence.mjs';import {validateAnswer,generateAnswer} from './rag-answer.mjs';
const adapter=()=>loadFixture(new URL('./fixtures/public-query-v1.json',import.meta.url));
const request={question:'Obligaciones',queries:['obligación']};
test('retrieval reads complete provisions, deduplicates and exposes budget exclusion',async()=>{
 const a=await adapter();const b=await retrieveEvidence(a,{...request,queries:['obligación','obligación']});assert.ok(b.evidence.length);assert.equal(new Set(b.evidence.map(p=>p.id)).size,b.evidence.length);
 const excluded=await retrieveEvidence(a,{...request,maxBytes:1});assert.equal(excluded.evidence.length,0);assert.equal(excluded.budget_exceeded,true);
});
test('text tampering and release drift fail closed',async()=>{
 const a=await adapter(),lookup=a.lookup;a.lookup=async q=>({...await lookup(q),text:'tampered'});await assert.rejects(retrieveEvidence(a,request),/integrity/);
 const b=await adapter(),coverage=b.coverage;b.coverage=async()=>{b.release.corpus_commit='changed';return coverage();};await assert.rejects(retrieveEvidence(b,request),/Release changed/);
});
test('answers reject invented sources and altered quotations; abstention is explicit',async()=>{
 const bundle=await retrieveEvidence(await adapter(),request),p=bundle.evidence[0];
 const answer={release_id:bundle.release.id,status:'answered',claims:[{kind:'quotation',text:p.text,citations:[{provision_id:p.id,quote:p.text}]}],limitations:['Synthetic fixture']};
 assert.deepEqual(await generateAnswer(bundle,async()=>answer),answer);
 const wrong=structuredClone(answer);wrong.claims[0].citations[0].provision_id='invented';assert.throws(()=>validateAnswer(wrong,bundle),/citation/);
 const altered=structuredClone(answer);altered.claims[0].text+=' altered';assert.throws(()=>validateAnswer(altered,bundle),/quotation/);
 assert.throws(()=>validateAnswer({...answer,release_id:'other'},bundle),/envelope/);
 assert.ok(validateAnswer({release_id:bundle.release.id,status:'insufficient_evidence',claims:[],limitations:['No evidence retrieved']},bundle));
});
