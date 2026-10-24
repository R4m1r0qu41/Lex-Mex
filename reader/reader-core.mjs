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
/** Heading levels in hierarchy order; the JSON key order of `heading_context` is not meaningful. */
export const headingLevels=['libro','title','chapter','section','apartado'];
/** The index groups articles by Libro, Título and Capítulo, so two Libros that repeat a Título name stay apart. */
export const indexLevels=['libro','title','chapter'];
export function headingPath(context,levels=headingLevels) {return levels.map(level=>(context||{})[level]).filter(Boolean).join(' · ');}
/** One provision as Markdown: its heading path (when it differs from `previous`) above the label and the exact text. */
export function provisionMarkdown(provision,previous='') {
  const path=headingPath(provision.heading_context);
  return `${path&&path!==previous?`## ${path}\n\n`:''}${path?'###':'##'} ${provision.label}\n\n${provision.text}`;
}
export function instrumentMarkdown(document) {
  let previous='';
  return `# ${document.official_title}\n\n`+document.provisions.map(p=>{const markdown=provisionMarkdown(p,previous);previous=headingPath(p.heading_context);return markdown;}).join('\n\n');
}
/** Articles under a heading are grouped by that heading alone; other kinds keep their name. */
export function indexGroup(document,provision,searching=false) {
  if(searching)return document.short_name;
  const path=headingPath(provision.heading_context,indexLevels);
  if(provision.provision_type==='article'&&path)return path;
  const kind={article:'Artículos',transitory:'Transitorios',annex:'Anexos'}[provision.provision_type]||'Disposiciones';
  return [kind,path].filter(Boolean).join(' · ');
}

// Continuous-reading decisions are pure so they can be tested without a browser.
/** Chunk ranges of `size` provisions, nearest to the target chunk first, so a deep link renders before the rest. */
export function chunkOrder(count,target,size=40) {
  const chunks=[];for(let start=0;start<count;start+=size)chunks.push([start,Math.min(start+size,count)]);
  const first=Math.min(chunks.length-1,Math.max(0,Math.floor(target/size)));
  return chunks.map((range,index)=>({range,distance:Math.abs(index-first)})).sort((a,b)=>a.distance-b.distance||a.range[0]-b.range[0]).map(item=>item.range);
}
/** Active provision: the last article whose top has crossed the reading line, else the first. Article tops increase
 * with their index, so a binary search reads only a few positions per pass, independent of observer membership. */
export function activeIndex(count,topOf,line) {
  if(!count)return -1;
  let low=0,high=count-1,best=0;
  while(low<=high){const middle=(low+high)>>1;if(topOf(middle)<=line){best=middle;low=middle+1;}else high=middle-1;}
  return best;
}
/** Passive scrolling must not add history entries; deliberate navigation does. */
export function historyMode(trigger) {return ['index','reference','instrument','search'].includes(trigger)?'push':trigger==='popstate'?'none':'replace';}
/** `full` names the law by its official title instead of its abbreviation. */
export function citation(document,provision,full=false) {const label=provision.label;return `${full?document.official_title:document.short_name}, ${label.charAt(0).toLowerCase()}${label.slice(1)}`;}
/** The complete canonical body, never a rendering of it. */
export function articleCopy(provision) {return provision.text;}
/** PDF following stays unavailable: no page anchor has been verified against a retained PDF hash. */
export function followState() {return {available:false,title:'Lectura independiente',note:'Sin anclaje verificado para este artículo. Navega por el PDF de forma independiente; conservamos su posición.'};}
export function matchSummary(count) {return count===1?'1 disposición coincidente':`${count} disposiciones coincidentes`;}
/** Placeholder height for a provision whose body is not yet laid out; real sizes replace it once rendered. */
export const textScales={large:1,small:0.65};
/** Estimate parameters for a text scale: a smaller type fits more characters per line and each line is shorter. */
export function estimateMetrics(scale=1) {return {charsPerLine:Math.round(52/scale),lineHeight:34*scale};}
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
