import test from 'node:test';import assert from 'node:assert/strict';import {loadFixture} from './query-fixture.mjs';import {callRetrievalTool,retrievalTools} from './mcp-tools.mjs';
test('read-only tools preserve public retrieval data and reject release substitution',async()=>{
 const a=await loadFixture(new URL('./fixtures/public-query-v1.json',import.meta.url)),release_id=a.release.id;
 assert.ok(retrievalTools.find(t=>t.name==='get_provision').inputSchema.required.includes('instrument'));
 assert.equal(retrievalTools.length,4);assert.ok(retrievalTools.every(t=>t.annotations.readOnlyHint));
 const r=await callRetrievalTool(a,'get_provision',{release_id,instrument:'norma-demo',type:'article',number:'1'});assert.equal(r.isError,false);assert.equal(r.structuredContent.release.id,release_id);assert.ok(r.structuredContent.result.data.provision.text);
 await assert.rejects(callRetrievalTool(a,'get_coverage',{release_id:'f'.repeat(64)}),/Release unavailable/);
 await assert.rejects(callRetrievalTool(a,'get_coverage',{release_id,private:true}),/arguments/);
});
