import test from 'node:test';import assert from 'node:assert/strict';import {projectionAdapter} from './query-projection.mjs';import {hash} from './query-contract.mjs';
test('projection refuses tampering, synthetic data and unexpected scope',()=>{
 const bytes=Buffer.from('{}');assert.throws(()=>projectionAdapter(bytes,'0'.repeat(64)),/digest/);
 assert.throws(()=>projectionAdapter(bytes,hash(bytes)),/Invalid serving/);
 const invalid=Buffer.from(JSON.stringify({schema_version:'lex-mex-serving-projection-v1',data:{release:{fixture:false},instruments:[{slug:'other'}]}}));assert.throws(()=>projectionAdapter(invalid,hash(invalid)),/scope/);
});
