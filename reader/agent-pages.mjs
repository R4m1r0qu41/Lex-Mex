import {createHash} from 'node:crypto';
import {safeRelative,releaseId} from './reader-core.mjs';
import {isOfficialSourceUrl} from './query-contract.mjs';

const sha=b=>createHash('sha256').update(b).digest('hex');
const escape=s=>String(s??'').replace(/[&<>"'\r]/g,c=>({'&':'&amp;','<':'&lt;','>':'&gt;','"':'&quot;',"'":'&#39;','\r':'&#13;'}[c]));
const limit=25;
const candidateLimit=20;
// Corpus-wide budget for static term pages: one file per distinct word is unbounded, so only the most
// frequent words get a page (count, then alphabetical, for a deterministic selection). Other words are
// answered by the search endpoint, never by a missing page being read as absence.
export const MAX_TERM_PAGES=5000;
const stop=new Set('para como este esta estos estas entre sobre desde hasta cuando donde mismo misma todos todas cada sera seran sido sean sean debe deben debera deberan podra podran dichos dichas dicho dicha mediante conforme articulo articulos fraccion fracciones parrafo parrafos primero segundo tercero anterior anteriores siguiente siguientes caso casos efectos dispuesto presente presentes previsto previstas previstos quienes cualquier cuales otros otras otra otro tanto forma menos parte partes dentro fuera antes despues mismo mexico mexicanos general disposiciones disposicion institucion instituciones'.split(' '));
const termOf=s=>s.toLowerCase().normalize('NFD').replace(/\p{M}/gu,'');
const warning='Colección parcial. La fecha de reforma es la registrada en la fuente recuperada; no acredita vigencia actual ni aplicabilidad histórica. No hay registro de última comprobación con la fuente oficial. No es asesoría jurídica.';
const chunks=items=>Array.from({length:Math.max(1,Math.ceil(items.length/limit))},(_,i)=>items.slice(i*limit,(i+1)*limit));
const link=(href,label)=>`<a href="${escape(href)}">${escape(label)}</a>`;
const safeUrl=url=>{if(!isOfficialSourceUrl(url))throw Error('Invalid official source URL');return url;};
const page=(title,body)=>Buffer.from(`<!doctype html><html lang="es"><head><meta charset="utf-8"><meta name="viewport" content="width=device-width"><title>${escape(title)} · Lex-Mex</title></head><body><header><p>Lex-Mex · acceso estático para agentes</p></header><main><h1>${escape(title)}</h1>${body}</main><footer><p>${warning}</p></footer></body></html>\n`);
const sourceHtml=source=>`<p>Fuente oficial: ${link(safeUrl(source.official_url),source.publisher||source.official_url)}. Recuperada: ${escape(source.retrieved_at||'no disponible')}. SHA-256: <code>${escape(source.source_sha256)}</code>. Parser: ${escape(source.parser_version||'no disponible')}; esquema de fuente: ${escape(source.schema_version||'no disponible')}.</p>`;
const navigation=(paths,current)=>`<nav>${current>0?link(paths[current-1],'Anterior')+' · ':''}${current+1<paths.length?link(paths[current+1],'Siguiente'):''} <span>Página ${current+1} de ${paths.length}</span></nav>`;

// A presentation-only view over public documents already verified by the packager.
// Its independent content ID avoids changing an existing canonical release.
export function agentPages(manifest,content,{maxTermPages=MAX_TERM_PAGES}={}){
  const release=releaseId(manifest.release_id),base=manifest.base_path;
  const releasePath=`releases/${release}/`;
  const docs=manifest.instruments.map(item=>{
    if(!/^[a-z0-9-]+$/.test(item.slug))throw Error('Invalid agent instrument slug');
    const expected=`mx/${item.slug}/instrument.json`;
    if(item.document_path!==expected)throw Error('Unexpected agent instrument document path');
    const bytes=content.get(releasePath+expected);if(!bytes)throw Error('Missing verified agent document');
    const doc=JSON.parse(bytes);
    if(doc.id!==item.id||doc.slug!==item.slug||!Array.isArray(doc.provisions))throw Error('Agent instrument identity mismatch');
    return {item,doc};
  });
  const files=new Map();
  const targets=new Map(docs.flatMap(({item,doc})=>doc.provisions.map(p=>[p.id,{slug:item.slug,path:sha(p.id)+'.html',label:p.label}])));
  const terms=new Map();
  for(const {doc,item} of docs){for(const p of doc.provisions){
    const words=[...p.text.matchAll(/\p{L}+/gu)],counts=new Map();
    for(const match of words){const term=termOf(match[0]);if(!/^[a-z]{4,40}$/.test(term)||stop.has(term))continue;const prior=counts.get(term);if(prior)prior.count++;else counts.set(term,{count:1,index:match.index});}
    for(const [term,hit] of counts){
      if(!terms.has(term))terms.set(term,{count:0,hits:[]});const entry=terms.get(term);entry.count++;
      entry.hits.push({id:p.id,label:p.label,slug:item.slug,short:doc.short_name,text:p.text,index:hit.index,score:hit.count/Math.sqrt(Math.max(1,words.length))});
      // Bound retained postings per term, even as the corpus grows.
      entry.hits.sort((a,b)=>b.score-a.score||a.id.localeCompare(b.id,'en'));if(entry.hits.length>candidateLimit)entry.hits.length=candidateLimit;
    }
  }}
  if(!Number.isSafeInteger(maxTermPages)||maxTermPages<0)throw Error('Invalid term page budget');
  const totalTerms=terms.size;
  const kept=new Map([...terms].sort(([a,x],[b,y])=>y.count-x.count||a.localeCompare(b,'en')).slice(0,maxTermPages));
  for(const [term,{count,hits}] of [...kept].sort(([a],[b])=>a.localeCompare(b,'en'))){
    files.set(`terms/${term}.html`,page(`Coincidencias literales: ${term}`,`${link('../catalog-1.html','Instrumentos')}<p>Hasta ${candidateLimit} candidatos de ${count} disposiciones con esta palabra (mayúsculas y acentos normalizados; sin sinónimos ni búsqueda semántica). Orden mecánico por frecuencia dividida por raíz de longitud. Resultados no exhaustivos: una ausencia aquí no demuestra ausencia normativa. Lea el artículo completo y sus referencias; otras normas fuera de la selección pueden aplicar.</p><ul>${hits.map(hit=>{const start=Math.max(0,hit.index-100),end=Math.min(hit.text.length,hit.index+300);return `<li>${link(`../${hit.slug}/${sha(hit.id)}.html`,`${hit.short} — ${hit.label}`)}<blockquote>${start?'[…] ':''}${escape(hit.text.slice(start,end))}${end<hit.text.length?' […]':''}</blockquote></li>`;}).join('')}</ul>`));
  }
  const catalogPages=chunks(docs).map((_,i)=>`catalog-${i+1}.html`);
  for(const [i,group] of chunks(docs).entries()){
    const examples=[...new Set(group.flatMap(({doc})=>(doc.official_title.match(/\p{L}+/gu)||[]).map(termOf)))].filter(term=>kept.has(term)).slice(0,5);
    files.set(catalogPages[i],page('Instrumentos publicados',`<p>Publicación canónica: <code>${release}</code>. ${warning}</p><p>Para preguntas temáticas: solicite <code>terms/palabra.html</code> relativo a este directorio (una palabra española, minúsculas, sin acentos; mínimo 4 letras). Ejemplos: ${examples.map(term=>link(`terms/${term}.html`,term)).join(' · ')}. Hasta 20 candidatos con fragmentos exactos; no exhaustivos. Pruebe varias palabras específicas, sin asumir que una coincidencia resuelve la cuestión jurídica. Hay páginas de término para las ${kept.size} palabras más frecuentes de ${totalTerms}; una ruta de término ausente indica que no existe ese índice literal, no que no exista una norma.</p><p>Para leer una norma: consulte su guía por capítulos y fragmentos exactos antes de abrir artículos. No descargue la colección completa ni el manifiesto de integridad para responder una consulta.</p><ul>${group.map(({doc,item})=>`<li>${link(`${item.slug}/guide-1.html`,`${doc.short_name} — ${doc.official_title}`)}. Reforma registrada: ${escape(doc.latest_reform_date||'no disponible')}; ${doc.provisions.length} disposiciones.</li>`).join('')}</ul>${navigation(catalogPages,i)}<p>${link(base+releasePath+'manifest.json',`Manifiesto de integridad (${content.get(releasePath+'manifest.json').length} bytes; solo para verificar archivos)`)}</p>`));
  }
  const seen=new Set();
  for(const {item,doc} of docs){
    const referencesBySource=new Map();
    for(const edge of doc.references||[]){if(!referencesBySource.has(edge.source_provision_id))referencesBySource.set(edge.source_provision_id,[]);referencesBySource.get(edge.source_provision_id).push(edge);}
    const guidePages=chunks(doc.provisions).map((_,i)=>`guide-${i+1}.html`);
    const articlePath=p=>`${sha(p.id)}.html`;
    for(const [i,group] of chunks(doc.provisions).entries()){
      files.set(`${item.slug}/${guidePages[i]}`,page(`${doc.short_name} — guía de disposiciones`,`${link('../catalog-1.html','Instrumentos')}<h2>${escape(doc.official_title)}</h2><p>Reforma registrada en esta fuente: ${escape(doc.latest_reform_date||'no disponible')}. Fragmentos iniciales exactos para orientar la lectura; no son resúmenes ni el texto completo.</p>${sourceHtml(doc.source)}<ol start="${i*limit+1}">${group.map(p=>`<li>${link(articlePath(p),p.label)} <span>${escape(Object.values(p.heading_context||{}).filter(Boolean).join(' · '))}</span><blockquote>${escape(Array.from(p.text).slice(0,400).join(''))}${Array.from(p.text).length>400?' […]':''}</blockquote></li>`).join('')}</ol>${navigation(guidePages,i)}`));
    }
    for(const [i,p] of doc.provisions.entries()){
      if(typeof p.id!=='string'||typeof p.text!=='string'||seen.has(p.id))throw Error('Invalid or duplicate agent provision');seen.add(p.id);
      const textPath=`mx/${item.slug}/${safeRelative(p.text_path)}`;
      const text=content.get(releasePath+textPath);
      if(!text||!text.equals(Buffer.from(p.text))||sha(text)!==p.text_sha256)throw Error('Agent exact text mismatch');
      const mdPath=`mx/${item.slug}/${safeRelative(p.markdown_path)}`;
      if(!content.has(releasePath+mdPath))throw Error('Missing agent Markdown representation');
      // Preserve paragraph bytes in a preformatted text node; never inject source markup.
      const siblings=[i>0?link(articlePath(doc.provisions[i-1]),'Anterior'):'',i+1<doc.provisions.length?link(articlePath(doc.provisions[i+1]),'Siguiente'):''].filter(Boolean).join(' · ');
      const refs=referencesBySource.get(p.id)||[];
      const references=refs.length?`<h2>Referencias expresas registradas</h2><ul>${refs.map(edge=>{const target=targets.get(edge.target_provision_id);return `<li>${escape(edge.source_span)}: ${target&&edge.resolution_status==='resolved'?link(`../${target.slug}/${target.path}`,`${target.label} — ${edge.target_provision_id}`):`${escape(edge.target_provision_id)} (sin resolución, histórica o fuera de esta selección)`}</li>`;}).join('')}</ul>`:'';
      files.set(`${item.slug}/${articlePath(p)}`,page(`${doc.short_name} — ${p.label}`,`<nav>${link(guidePages[Math.floor(i/limit)],'Guía')} · ${link('../catalog-1.html','Instrumentos')} · ${siblings}</nav><h2>${escape(doc.official_title)}</h2><p>Cita estable: <code>${escape(p.id)}</code>. Release: <code>${release}</code>.</p><p>${escape(Object.values(p.heading_context||{}).filter(Boolean).join(' · '))}. Estado: ${escape(p.temporal_status)} (clasificación cualificada del repositorio).</p><p>Reforma registrada de la norma: ${escape(doc.latest_reform_date||'no disponible')}; puede no describir la versión de anexos con fuente separada.</p>${sourceHtml(p.source)}<h2>Texto íntegro de la disposición</h2><pre style="white-space:pre-wrap"><span>${escape(p.text)}</span></pre>${references}<p>SHA-256 del texto UTF-8: <code>${escape(p.text_sha256)}</code>. ${link(base+releasePath+mdPath,'Markdown con referencias expresas')} · ${link(base+releasePath+textPath,'Texto exacto sin metadatos')}</p>`));
    }
  }
  // Hash generated bytes, not a manually maintained generator version.
  const id=sha(JSON.stringify([...files].map(([path,bytes])=>[path,sha(bytes)])));
  const prefix=`agents/${id}/`;
  return {id,entry:base+prefix+'catalog-1.html',files:new Map([...files].map(([path,bytes])=>[prefix+path,bytes]))};
}
