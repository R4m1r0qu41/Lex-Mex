export function normalize(value) {return String(value).normalize('NFD').replace(/[\u0300-\u036f]/g,'').toLowerCase();}
export function searchDocuments(documents, query) {
  const needle=normalize(query.trim());
  return documents.flatMap(document=>document.provisions.filter(p=>!needle||normalize(`${document.short_name} ${p.label} ${p.text}`).includes(needle)).map(provision=>({document,provision})));
}
export function safeRelative(path) {
  if(typeof path!=='string'||!path||path.startsWith('/')||path.includes('..')||path.includes('\\')||/[?#:]|%/.test(path))throw new Error('Ruta de publicación inválida');
  return path;
}
export function releaseId(value) {if(!/^[a-f0-9]{64}$/.test(value))throw new Error('Identificador de publicación inválido');return value;}
export function referencesFor(document, provision, documents) {
  const byId=new Map(documents.flatMap(d=>d.provisions.map(p=>[p.id,{document:d,provision:p}])));
  return document.references.filter(r=>r.source_provision_id===provision.id).map(reference=>({reference,target:reference.resolution_status==='resolved'?byId.get(reference.target_provision_id):undefined}));
}
export const statusLabels={unknown:'Estado por determinar',published_not_effective:'Publicado, sin vigencia',effective:'Vigencia registrada',future_effective:'Vigencia futura',partially_effective:'Vigencia parcial',conditionally_effective:'Vigencia condicionada',repealed:'Derogado',partially_repealed:'Derogación parcial',conditional_pending:'Condición pendiente',repealed_with_survival:'Derogado con efectos subsistentes',superseded:'Sustituido',temporarily_applicable:'Aplicación temporal',pending_consolidation:'Consolidación pendiente'};
export const reviewLabels={not_analyzed:'Sin análisis temporal',machine_accepted:'Clasificación automática',review_required:'Revisión pendiente',lawyer_verified:'Revisión jurídica registrada'};
export function navigation(document,id) {const index=document.provisions.findIndex(p=>p.id===id);return {previous:document.provisions[index-1],next:document.provisions[index+1]};}
export function selectionUrl(current,release,document,provision) {const url=new URL(current);url.search='';url.searchParams.set('release',releaseId(release));url.searchParams.set('instrument',document.slug);url.searchParams.set('provision',provision.id);return url;}

// Legal text remains literal UTF-8; presentation adds no escapes or links to it.
export function instrumentMarkdown(document) {
  return `# ${document.official_title}\n\n`+document.provisions.map(p=>`## ${p.label}\n\n${p.text}`).join('\n\n');
}
export function indexGroup(document,provision,searching=false) {
  if(searching)return document.short_name;
  const kind={article:'Artículos',transitory:'Transitorios',annex:'Anexos'}[provision.provision_type]||'Disposiciones';
  const headings=provision.heading_context||{};
  return [kind,headings.title,headings.chapter].filter(Boolean).join(' · ');
}

// Continuous-reading decisions are pure so they can be tested without a browser.
/** Chunk ranges of `size` provisions, nearest to the target chunk first, so a deep link renders before the rest. */
export function chunkOrder(count,target,size=40) {
  const chunks=[];for(let start=0;start<count;start+=size)chunks.push([start,Math.min(start+size,count)]);
  const first=Math.min(chunks.length-1,Math.max(0,Math.floor(target/size)));
  return chunks.map((range,index)=>({range,distance:Math.abs(index-first)})).sort((a,b)=>a.distance-b.distance||a.range[0]-b.range[0]).map(item=>item.range);
}
/** Active provision: the last visible article whose top has crossed the reading line, else the first visible one. */
export function pickActive(visible,line) {
  if(!visible.length)return -1;
  let best;for(const item of visible)if(item.top<=line&&(!best||item.top>best.top))best=item;
  return (best||visible.reduce((a,b)=>b.index<a.index?b:a)).index;
}
/** Passive scrolling must not add history entries; deliberate navigation does. */
export function historyMode(trigger) {return ['index','reference','instrument','search'].includes(trigger)?'push':trigger==='popstate'?'none':'replace';}
export function citation(document,provision) {const label=provision.label;return `${document.short_name}, ${label.charAt(0).toLowerCase()}${label.slice(1)}`;}
/** The complete canonical body, never a rendering of it. */
export function articleCopy(provision) {return provision.text;}
/** PDF following stays unavailable: no page anchor has been verified against a retained PDF hash. */
export function followState() {return {available:false,title:'Lectura independiente',note:'Sin anclaje verificado para este artículo. Navega por el PDF de forma independiente; conservamos su posición.'};}
export function matchSummary(count) {return count===1?'1 disposición coincidente':`${count} disposiciones coincidentes`;}
/** Placeholder height for a provision whose body is not yet laid out; real sizes replace it once rendered. */
export function estimateHeight(provision,{charsPerLine=52,lineHeight=34,chrome=260}={}) {
  const lines=String(provision.text).split('\n').reduce((sum,line)=>sum+Math.max(1,Math.ceil(line.length/charsPerLine)),0);
  return chrome+lines*lineHeight;
}
/** The open PDF keeps its page and zoom while the reader scrolls; only deliberate navigation swaps documents. */
export function sourceDecision(trigger,loaded,next) {
  if(!loaded)return 'swap';
  if(loaded.pdf===next)return 'keep';
  return trigger==='scroll'?'offer':'swap';
}
