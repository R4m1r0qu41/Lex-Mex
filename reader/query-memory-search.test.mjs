import test from 'node:test';
import assert from 'node:assert/strict';
import {readFile} from 'node:fs/promises';
import {memoryAdapter} from './query-memory.mjs';
import {memoryAdapter as prior} from './fixtures/query-memory-prior.mjs';
import {hash} from './query-contract.mjs';

// Differential regression: the optimized search must return exactly what the frozen prior implementation returned.
const fixture=JSON.parse(await readFile(new URL('./fixtures/public-query-v1.json',import.meta.url),'utf8'));
const base=fixture.provisions[0];
const texts=[
  'segregados',                                   // exact
  'Los fondos segregados se identifican. Segregados otra vez; SEGREGADOS.',
  'desegregados y segregadosx no son palabras completas',
  'xsegregados segregados',                        // first occurrence fails the boundary, second passes
  'segregados_ segregados1 1segregados _segregados', // underscore and digits are word characters
  '(segregados) [segregados] "segregados"',        // punctuation is a boundary
  '𝒜segregados segregados𝒜',                       // astral letters adjacent
  '😀segregados😀',                                 // astral non-letter adjacent
  '٣segregados segregados٣',                        // non-ASCII digits adjacent
  'ÁRBOL árbol Arbol ÉLECTRÓNICA electronica cañón canon',
  'a.b a+b a|b (x) [y] $ ^ \\ ? * {1,2} a.b',
  'cuentas de prueba, cuentas  de prueba, cuentas de prueba',
  'ﬁnal Ⅻ ǅ İstanbul ǈ',                           // normalization edge cases
  'x',
  '',
];
const queries=['segregados','Segregados','segregado','desegregados','xsegregados','arbol','ÁRBOL','electronica','Electrónica','canon','cañón',
  'a.b','a+b','a|b','(x)','[y]','$','^','\\','?','*','{1,2}','x','cuentas de prueba','cuentas  de prueba','prueba','de','𝒜','😀','٣','final','ǅ','i̇stanbul','no existe'];
const types=['article','transitory','annex'];
function dataset(){
  const provisions=structuredClone(fixture.provisions);
  texts.forEach((text,i)=>types.forEach((type,t)=>{
    const p=structuredClone(base);p.number=`synthetic-${i}-${t}`;p.provision_type=type;p.id=`${base.id}:synthetic:${i}:${t}`;
    p.text=text||'texto vacío';p.text_sha256=hash(Buffer.from(p.text));p.label=`Sintético ${i}`;provisions.push(p);
  }));
  return {...structuredClone(fixture),provisions};
}
async function pages(adapter,params){
  const out=[];let cursor=null;
  for(let n=0;n<50;n++){const page=await adapter.search({...params,cursor});out.push(page);if(!page.next_cursor)break;cursor=page.next_cursor;}
  return out;
}
const slug=base.instrument.slug;
test('optimized search is byte-identical to the prior implementation across ranking, boundaries, filters and cursors',async()=>{
  const data=dataset(),a=memoryAdapter(data),b=prior(data);
  for(const query of queries)for(const instrument of [undefined,slug,'otro'])for(const limit of [1,3,10]){
    const got=await pages(a,{query,instrument,limit}),want=await pages(b,{query,instrument,limit});
    assert.equal(JSON.stringify(got),JSON.stringify(want),`${JSON.stringify(query)} ${instrument} limit ${limit}`);
  }
});
test('search matches only whole words and keeps the empty-query error',async()=>{
  const a=memoryAdapter(dataset());
  const hits=await a.search({query:'segregados',limit:10});
  assert.ok(hits.items.length>0);
  await assert.rejects(()=>a.search({query:'   ',limit:10}),/Invalid search/);
  await assert.rejects(()=>b_search_cursor(a),/Invalid cursor/);
});
async function b_search_cursor(a){return a.search({query:'segregados',limit:1,cursor:'not-a-cursor'});}
// Optional: also compare against a real serving projection (not committed). LEX_MEX_PROJECTION=/path/projection.json
test('optimized search matches the prior implementation on a real projection when supplied',{skip:!process.env.LEX_MEX_PROJECTION},async()=>{
  const envelope=JSON.parse(await readFile(process.env.LEX_MEX_PROJECTION,'utf8')),data=envelope.data||envelope;
  const a=memoryAdapter(data),b=prior(data);
  const real=['segregados','cuentas','fondos de pago electrónico','Artículo 46','instituciones de fondos de pago electrónico','límites','obligaciones','plataforma','de','a','prueba','cliente','recursos','sociedades','x'];
  for(const query of real)for(const instrument of [undefined,'lritf','lic','itf-dcg-2018','ifpe-dcg-2021'])for(const limit of [10]){
    const got=await pages(a,{query,instrument,limit}),want=await pages(b,{query,instrument,limit});
    assert.equal(JSON.stringify(got),JSON.stringify(want),`${query} ${instrument}`);
  }
});
