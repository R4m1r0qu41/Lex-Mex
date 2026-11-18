import {Buffer} from 'node:buffer';
import {hash} from './query-contract.mjs';
import {memoryAdapter} from './query-memory.mjs';
const allowed=['ifpe-dcg-2021','itf-dcg-2018','lic','lritf'];
// Filesystem-free candidate for the bounded pilot; deployment requires runtime measurements.
export function projectionAdapter(bytes,expectedSha256){
  if(!/^[a-f0-9]{64}$/.test(expectedSha256)||hash(bytes)!==expectedSha256)throw Error('Projection digest mismatch');
  const envelope=JSON.parse(new TextDecoder().decode(bytes));
  if(envelope.schema_version!=='lex-mex-serving-projection-v1'||!envelope.data||envelope.data.release.fixture!==false)throw Error('Invalid serving projection');
  const data=envelope.data;
  if(data.instruments.map(i=>i.slug).sort().join(',')!==allowed.join(','))throw Error('Unexpected serving scope');
  if(data.provisions.length>2000||bytes.byteLength>16000000)throw Error('Projection exceeds pilot budget');
  const adapter=memoryAdapter(data);adapter.notice='Colección parcial de cuatro instrumentos; fuentes y revisión limitadas a la publicación registrada. No acredita actualidad.';return adapter;
}
