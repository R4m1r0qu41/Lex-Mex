// FROZEN COPY of reader/query-memory.mjs at 4fb0b5579, kept only as a differential-test oracle for search.
// Do not import from production code. Delete once the optimized search has shipped and been accepted.
import {Buffer} from 'node:buffer';
// Bounded response adapter for explicit local in-memory datasets, not federal-scale serving.
import {hash,normalize,checkArticle,checkInstrument,checkRelease,checkEdge} from '../query-contract.mjs';
export function memoryAdapter(input){
  const data=structuredClone(input);checkRelease(data.release);data.instruments.forEach(checkInstrument);data.provisions.forEach(checkArticle);
  for(const [id,edges] of Object.entries(data.references)){if(!data.provisions.some(p=>p.id===id)&&!data.instruments.some(i=>i.id===id))throw Error('Unknown reference source');edges.forEach(checkEdge);}
  const identities=new Set();for(const p of data.provisions){const key=[p.instrument.slug,p.provision_type,p.number].join('/');if(identities.has(key)||!data.instruments.some(i=>JSON.stringify(i)===JSON.stringify(p.instrument)))throw Error('Ambiguous fixture identity');identities.add(key);}
  // Normalized text is computed once per provision, not inside sort comparisons.
  const norm=new Map(data.provisions.map(p=>[p.id,{text:normalize(p.text),both:normalize(p.text+' '+p.label)}]));
  const hit=(p,query='')=>{const start=query?Math.max(0,norm.get(p.id).text.indexOf(normalize(query))-50):0;return {id:p.id,instrument:p.instrument,provision_type:p.provision_type,number:p.number,label:p.label,excerpt:Array.from(p.text.slice(start)).slice(0,160).join('')};};
  function paginate(items,limit,token,binding,map=x=>x){
    const scope=hash(binding);let offset=0;
    if(token){let decoded;try{decoded=JSON.parse(Buffer.from(token,'base64url').toString());}catch{throw Error('Invalid cursor');}if(!decoded||typeof decoded!=='object'||Array.isArray(decoded)||decoded.scope!==scope||!Number.isSafeInteger(decoded.offset)||decoded.offset<0||decoded.offset>items.length)throw Error('Invalid cursor');offset=decoded.offset;}
    const next=offset+limit;
    return {items:structuredClone(items.slice(offset,next).map(map)),next_cursor:next<items.length?Buffer.from(JSON.stringify({scope,offset:next})).toString('base64url'):null};
  }
  return {
    release:structuredClone(data.release),
    async catalog({limit,cursor}){return paginate(data.instruments,limit,cursor,JSON.stringify([data.release.id,'catalog']));},
    async search({query,instrument,limit,cursor}){
      if(!normalize(query).trim())throw Error('Invalid search');
      const needle=normalize(query);
      const pattern=new RegExp('(?<![\\p{L}\\p{N}_])'+needle.replace(/[.*+?^${}()|[\]\\]/g,'\\$&')+'(?![\\p{L}\\p{N}_])','u');
      const rank=p=>(norm.get(p.id).text.split(needle).length-1)/Math.sqrt(Math.max(1,p.text.length));
      const order={article:0,transitory:1,annex:2};
      const matches=data.provisions.filter(p=>(!instrument||p.instrument.slug===instrument)&&pattern.test(norm.get(p.id).both)).sort((a,b)=>order[a.provision_type]-order[b.provision_type]||rank(b)-rank(a)||a.id.localeCompare(b.id,'en'));
      return {...paginate(matches,limit,cursor,JSON.stringify([data.release.id,'search',query,instrument]),p=>hit(p,query)),query};
    },
    async index({slug,limit,cursor}){return {...paginate(data.provisions.filter(p=>p.instrument.slug===slug).map(p=>hit(p)),limit,cursor,JSON.stringify([data.release.id,'index',slug])),query:`Índice de ${slug}`};},
    async lookup({slug,type,number}){return structuredClone(data.provisions.find(p=>p.instrument.slug===slug&&p.provision_type===type&&p.number===number)||null);},
    async references({id,limit,cursor}){return paginate(data.references[id]||[],limit,cursor,JSON.stringify([data.release.id,'references',id]));},
    async coverage(){return structuredClone(data.coverage);},
  };
}
