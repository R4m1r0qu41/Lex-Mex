// Explicit synthetic fixtures only. Real retained releases use query-retained.mjs.
import {readFile} from 'node:fs/promises';
import {memoryAdapter} from './query-memory.mjs';
export async function loadFixture(path){return fixtureAdapter(JSON.parse(await readFile(path,'utf8')));}
export function fixtureAdapter(input){
  if(Object.keys(input).sort().join(',')!=='coverage,instruments,provisions,references,release,schema_version'||input.schema_version!=='public-query-fixture-v1'||input.release.fixture!==true)throw Error('Expected explicit synthetic fixture');
  return memoryAdapter(input);
}
