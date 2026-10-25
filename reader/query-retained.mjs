// LOCAL reduced-corpus bridge over already-public Rust artifacts, not the private DB.
import {readFile,realpath} from 'node:fs/promises';
import {resolve,join,sep} from 'node:path';
import {hash} from './query-contract.mjs';
import {memoryAdapter} from './query-memory.mjs';
import {releaseId,safeRelative} from './reader-core.mjs';

export async function retainedAdapter(directory){
  const root=await realpath(resolve(directory));
  const latest=JSON.parse(await readFile(join(root,'latest.json'),'utf8'));
  const id=releaseId(latest.release_id),base=await realpath(join(root,'releases',id));
  if(!base.startsWith(root+sep))throw Error('Release path escapes input');
  async function read(path){const full=await realpath(join(base,safeRelative(path)));if(!full.startsWith(base+sep))throw Error('Retained path escapes release');return readFile(full);}
  const manifest=JSON.parse(await read('manifest.json'));
  const payload={...manifest};delete payload.release_id;
  if(manifest.schema_version!=='public-release-v1'||manifest.release_id!==id||hash(JSON.stringify(payload,null,2)+'\n')!==id)throw Error('Retained manifest identity mismatch');
  const expected=['ifpe-dcg-2021','itf-dcg-2018','lic','lritf'];
  if(!Array.isArray(manifest.instruments)||manifest.instruments.map(i=>i.slug).sort().join(',')!==expected.join(','))throw Error('Local bridge only supports the retained four-instrument selection');
  const entries=new Map();for(const f of manifest.files){const path=safeRelative(f.path);if(entries.has(path))throw Error('Duplicate manifest path');entries.set(path,f);}
  async function verified(path){const f=entries.get(path);if(!f)throw Error('Unlisted retained artifact');const bytes=await read(path);if(bytes.length!==f.bytes||hash(bytes)!==f.sha256)throw Error('Retained artifact integrity mismatch');return bytes;}
  const data={release:{id,corpus_commit:manifest.corpus_commit,fixture:false},instruments:[],provisions:[],references:{},coverage:{scope:'selected_partial',notice:'Publicación retenida: LRITF, LIC, DCG-ITF-2018 y DCG-IFPE-2021. No es el corpus federal actual ni una comprobación de vigencia. Las referencias contienen solo el detalle publicado en esa versión.'}};
  const originals=[];
  for(const entry of manifest.instruments){
    const path=`mx/${entry.slug}/instrument.json`;if(entry.document_path!==path)throw Error('Unexpected retained document path');
    const doc=JSON.parse(await verified(path));if(doc.id!==entry.id||doc.slug!==entry.slug)throw Error('Retained instrument identity mismatch');
    const instrument={slug:doc.slug,id:doc.id,title:doc.official_title,short_name:doc.short_name};data.instruments.push(instrument);originals.push(doc);
    for(const p of doc.provisions){
      // Do not silently discard new effects/conditions to fit this local bridge.
      if(p.commencement_condition!==null||!Array.isArray(p.repeals)||p.repeals.length)throw Error('Unsupported retained legal qualification; extend public contract first');
      if(!p.id.startsWith(`${doc.id}:${p.provision_type}:`))throw Error('Retained provision ownership mismatch');
      const text=await verified(`mx/${doc.slug}/${safeRelative(p.text_path)}`);
      if(!text.equals(Buffer.from(p.text))||hash(text)!==p.text_sha256)throw Error('Retained provision text mismatch');
      const number=p.id.slice(p.id.lastIndexOf(':')+1);
      data.provisions.push({id:p.id,instrument,provision_type:p.provision_type,number,label:p.label,text:p.text,text_sha256:p.text_sha256,source:{official_url:p.source.official_url,publisher:p.source.publisher,retrieved_at:p.source.retrieved_at,source_sha256:p.source.source_sha256,recorded_reform_date:p.provision_type==='annex'?null:doc.latest_reform_date??null},temporal_status:p.temporal_status,review_status:p.review_status});
    }
  }
  const byId=new Map(data.provisions.map(p=>[p.id,p]));
  for(const doc of originals){for(const edge of doc.references){
    if(!byId.has(edge.source_provision_id)&&edge.source_provision_id!==doc.id)throw Error('Unsupported retained reference anchor');
    const target=byId.get(edge.target_provision_id);
    if(!data.references[edge.source_provision_id])data.references[edge.source_provision_id]=[];
    data.references[edge.source_provision_id].push({target_id:edge.target_provision_id,target:target&&edge.resolution_status==='resolved'?{slug:target.instrument.slug,provision_type:target.provision_type,number:target.number}:null,source_span:edge.source_span,resolution_status:edge.resolution_status,resolution_rule:edge.resolution_rule??null,qualifiers:(edge.qualifiers??[]).map(q=>({type:q.type,text:q.text}))});
  }}
  const adapter=memoryAdapter(data);
  adapter.notice='PREVIEW LOCAL · Publicación retenida de 4 instrumentos. No consulta el corpus federal actual ni el DB en desarrollo.';
  return adapter;
}
