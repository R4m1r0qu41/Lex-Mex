import {searchDocuments,safeRelative,releaseId,referencesFor,statusLabels,reviewLabels,selectionUrl,indexGroup,instrumentMarkdown,chunkOrder,activeIndex,historyMode,citation,articleCopy,followState,matchSummary,estimateHeight,sourceDecision} from './reader-core.mjs';
const $=id=>document.getElementById(id);
const base=new URL(document.querySelector('meta[name="lex-mex-base"]').content,location.origin);
const compact=matchMedia('(max-width: 1100px)');
let release,manifest,documents=[],selected,comparison,origin,observer,flowToken=0,lockUntil=0,replaceTimer,userScrolled=false,settleTarget=null,loading=false,frame=0,trailing,held=null,realigns=0,nudges=0,recoveries=0,settledTimer,expectedTop=0;
const flow={document:null,nodes:[],filled:new Set()};
const state={pdfUrl:null,source:null,offer:null};
function url(path){return new URL(safeRelative(path),base).href;}
function inRelease(path){return url(`releases/${release}/${safeRelative(path)}`);}
async function fetchJson(href){const response=await fetch(href);if(!response.ok)throw new Error(`No se pudo cargar el corpus (${response.status}).`);return response.json();}
function message(text){$('message').textContent=text;$('message').hidden=false;clearTimeout(message.timer);message.timer=setTimeout(()=>$('message').hidden=true,5500);}
function externalLink(id,href){const parsed=new URL(href);if(!['https:','http:'].includes(parsed.protocol))throw new Error('URL de fuente inválida');$(id).href=parsed.href;}
function pill(text,qualified=false){const node=document.createElement('span');node.textContent=text;if(qualified)node.className='qualified';return node;}
function element(tag,className,text){const node=document.createElement(tag);if(className)node.className=className;if(text!==undefined)node.textContent=text;return node;}
async function copy(text,done,failed){try{await navigator.clipboard.writeText(text);message(done);}catch{message(failed);}}

// Index: one button per provision of the selected instrument, or collection search hits. Updating the current item never rebuilds the list, so focus survives.
function renderIndex(){
  const query=$('search').value;
  const hits=searchDocuments(query?documents:[flow.document],query);
  $('count').textContent=String(hits.length);
  $('search-summary').hidden=!query;$('search-summary').textContent=query?`${matchSummary(hits.length)}. Las coincidencias se cuentan por disposición, no por apariciones en el PDF.`:'';
  const list=$('provision-list');list.replaceChildren();let group;
  for(const hit of hits.slice(0,query?100:hits.length)){
    const type=indexGroup(hit.document,hit.provision,Boolean(query));
    if(type!==group){list.append(element('p','group-heading',type));group=type;}
    const button=element('button','',hit.provision.label);button.type='button';button.tabIndex=-1;button.dataset.provision=hit.provision.id;button.dataset.slug=hit.document.slug;
    button.addEventListener('click',()=>{closeSheets();go(hit.document,hit.provision,query?'search':'index');});list.append(button);
  }
  if(query&&hits.length>100)list.append(element('p','muted small','Mostrando los primeros 100 resultados. Ajusta la búsqueda.'));
  if(!hits.length)list.append(element('p','muted small','Sin coincidencias en esta colección.'));
  markIndex();
}
function indexButtons(){return [...$('provision-list').querySelectorAll('button')];}
function markIndex(){
  let current;
  for(const button of indexButtons()){
    const on=selected&&button.dataset.slug===selected.document.slug&&button.dataset.provision===selected.provision.id;
    button.setAttribute('aria-current',String(Boolean(on)));button.tabIndex=on?0:-1;if(on)current=button;
  }
  if(!current&&indexButtons()[0])indexButtons()[0].tabIndex=0;
  if(current){const list=$('provision-list');const top=current.offsetTop-list.offsetTop;if(top<list.scrollTop||top+current.offsetHeight>list.scrollTop+list.clientHeight)list.scrollTop=top-list.clientHeight/3;}
}
function indexKeys(event){
  const buttons=indexButtons();const at=buttons.indexOf(document.activeElement);if(at<0)return;
  const next={ArrowDown:at+1,ArrowUp:at-1,Home:0,End:buttons.length-1}[event.key];
  if(next===undefined||!buttons[next])return;event.preventDefault();buttons[at].tabIndex=-1;buttons[next].tabIndex=0;buttons[next].focus();
}

// The loaded document, not the active provision, owns the source lines, digest and verification.
function loadSource(source,pdfUrl){
  state.source=source;state.pdfUrl=pdfUrl;state.offer=null;$('source-offer').hidden=true;
  $('source-publisher').textContent=source.publisher;
  $('retrieval-date').textContent=`Copia obtenida: ${source.retrieved_at.slice(0,10)}`;
  $('verification-result').hidden=true;$('verification-result').textContent='';
  $('source-digest').textContent=`SHA-256 · ${source.source_sha256}`;
  externalLink('upstream-link',source.official_url);
  $('pdf').hidden=!pdfUrl;$('pdf-unavailable').hidden=Boolean(pdfUrl);$('verify-pdf').disabled=!pdfUrl;
  $('pdf-open').hidden=!pdfUrl;
  if(pdfUrl){$('pdf-open').href=pdfUrl;$('pdf').src=pdfUrl+'#view=FitH';}
  else $('pdf').removeAttribute('src');
}
function renderSource(document_,provision,trigger){
  const source=provision.source,pdfUrl=source.pdf_path?inRelease(source.pdf_path):null;
  $('reform-date').textContent=`Última reforma registrada en la fuente: ${document_.latest_reform_date||'no disponible'}`;
  const follow=followState();$('follow-title').textContent=follow.title;$('follow-note').textContent=follow.note;
  const decision=sourceDecision(trigger,state.source?{pdf:state.pdfUrl}:null,pdfUrl);
  if(decision==='swap')loadSource(source,pdfUrl);
  else if(decision==='keep'){state.offer=null;$('source-offer').hidden=true;}
  else{
    state.offer={source,pdfUrl};$('source-offer').hidden=false;
    $('source-offer-text').textContent=pdfUrl?'Esta disposición pertenece a otro documento oficial. El PDF abierto conserva su página y zoom.':'Esta disposición no tiene copia PDF verificada. El PDF abierto se conserva.';
    $('source-offer-open').hidden=!pdfUrl;
  }
}

// Legal text stays literal; only express references that resolve inside the release become links.
function renderText(host,provision,relations){
  const chars=Array.from(provision.text);let cursor=0;
  for(const {reference,target} of relations.filter(r=>r.target).sort((a,b)=>a.reference.start_char-b.reference.start_char)){
    if(reference.start_char<cursor||reference.end_char>chars.length||chars.slice(reference.start_char,reference.end_char).join('')!==reference.source_span)continue;
    host.append(document.createTextNode(chars.slice(cursor,reference.start_char).join('')));
    const link=element('a','',reference.source_span);link.href=selectionUrl(location.href,release,target.document,target.provision).href;link.title=`Comparar: ${target.document.short_name} · ${target.provision.label}`;
    link.addEventListener('click',event=>{event.preventDefault();compare(target,link);});host.append(link);cursor=reference.end_char;
  }
  host.append(document.createTextNode(chars.slice(cursor).join('')));
}
function renderRelations(relations){
  const details=element('details','relations');details.append(element('summary','',`Referencias expresas (${relations.length})`),element('p','muted small','Relaciones ancladas al texto. Una referencia no determina por sí sola un efecto jurídico.'));
  if(!relations.length)details.append(element('p','muted small','No hay referencias expresas registradas para esta disposición.'));
  for(const {reference,target} of relations){
    const row=element('div','relation');const control=element(target?'button':'span','',target?`${target.document.short_name} · ${target.provision.label}`:reference.source_span);
    if(target){control.type='button';control.addEventListener('click',()=>compare(target,control));}
    row.append(control,element('span','muted',target?'Comparar ↗':reference.resolution_status==='historical_target_unavailable'?'Fuente histórica fuera de colección':'Destino fuera de colección o sin resolver'));details.append(row);
  }
  return details;
}

// Every provision is real DOM inside one scroll container (native find, selection and anchors keep working);
// content-visibility defers painting. Bodies are filled in chunks, the chunk holding the target first.
function buildFlow(document_){
  flow.document=document_;flow.nodes=[];flow.filled=new Set();observer?.disconnect();cancelAnimationFrame(frame);frame=0;clearTimeout(trailing);$('flow').replaceChildren();
  observer=new IntersectionObserver(schedule,{root:$('article'),rootMargin:'0px 0px -50% 0px'});
  document_.provisions.forEach((provision,index)=>{
    const node=element('article','provision');node.dataset.index=String(index);node.style.containIntrinsicSize=`auto ${estimateHeight(provision)}px`;node.setAttribute('aria-labelledby',`h-${index}`);
    const heading=Object.values(provision.heading_context||{}).filter(Boolean).join(' · ');
    if(heading)node.append(element('p','heading-context',heading));
    const title=element('h2','',provision.label);title.id=`h-${index}`;node.append(title);
    flow.nodes.push(node);$('flow').append(node);observer.observe(node);
  });
}
function fill(index){
  if(flow.filled.has(index))return;flow.filled.add(index);
  const document_=flow.document,provision=document_.provisions[index],node=flow.nodes[index];
  const relations=referencesFor(document_,provision,documents);
  const classes=element('div','classification');classes.append(pill(statusLabels[provision.temporal_status]||provision.temporal_status,true),pill(reviewLabels[provision.review_status]||provision.review_status));
  const text=element('div','article-text');renderText(text,provision,relations);
  node.append(classes,element('p','muted small','La clasificación registrada no acredita por sí sola la vigencia actual o histórica del texto.'),text);
  if(provision.commencement_condition)node.append(element('div','notice',`Condición (${provision.commencement_condition.condition_status}): ${provision.commencement_condition.condition_text}`));
  if(provision.repeals.length)node.append(element('div','notice',provision.repeals.map(r=>`${r.cause==='judicial'?'Invalidez judicial':'Derogación legislativa'} · ${r.scope} ${r.ordinals.join(', ')}${r.dof_date?' · DOF '+r.dof_date:''}${r.renumbering?' · renumeración señalada por la fuente':''}`).join('\n')));
  node.append(renderRelations(relations));
}
function fillChunk([start,end]){for(let i=start;i<end;i++)fill(i);}
async function fillAll(target){
  const token=++flowToken,count=flow.document.provisions.length;loading=true;
  for(const [position,range] of chunkOrder(count,target).entries()){
    if(token!==flowToken)return;
    fillChunk(range);$('load-status').textContent=`Cargando disposiciones… ${flow.filled.size} de ${count}`;
    if(position===0){performance.mark('reader:first-chunk');scrollToIndex(target);}else schedule();
    await new Promise(resolve=>setTimeout(resolve,0));
  }
  if(token!==flowToken)return;
  loading=false;$('load-status').textContent='';performance.mark('reader:flow-complete');schedule();
  if(!userScrolled&&held===target){ // sizes above the target settle once everything is filled; a newer navigation owns its own target
    if(selected.index!==target)activate(flow.document,target,'settle');
    scrollToIndex(target);
  }
}

function barHeight(){return $('active-bar').offsetHeight;}
function offsetOf(index){const column=$('article');return flow.nodes[index].getBoundingClientRect().top-column.getBoundingClientRect().top-barHeight()-8;}
// How far the column can still move toward aligning an article: the first article cannot rise above the top, nor the last past the end.
function drift(index){const column=$('article'),top=Math.min(Math.max(0,column.scrollTop+offsetOf(index)),column.scrollHeight-column.clientHeight);return top-column.scrollTop;}
// Skipped articles are laid out from estimates, so a jump is re-aligned until the layout stops moving or the reader scrolls.
// While it settles, passive tracking cannot reassign the selection; a pass runs once it ends.
function scrollToIndex(index,tries=6){
  const column=$('article');if(!flow.nodes[index]||readerMoved())return;
  settleTarget=index;lockUntil=performance.now()+300;
  column.scrollTo({top:column.scrollTop+drift(index),behavior:'instant'});expectedTop=column.scrollTop;
  requestAnimationFrame(()=>setTimeout(()=>{
    if(settleTarget!==index||readerMoved())return;
    if(!userScrolled&&tries>0&&Math.abs(drift(index))>2){scrollToIndex(index,tries-1);return;}
    settleTarget=null;schedule();keepHeld();
  },60));
}
// An explicit target stays at the reading line until the reader moves the view or navigates again: deferred article
// sizes and late web-font swaps can move it after the first jump. While a target is held this code does the anchoring
// (re-aligning the target as soon as the layout changes) and the browser's scroll anchoring is off, so the column only
// moves when this code moves it, unless the reader does: keyboard, wheel, touch, scrollbar, find in page, focus or
// assistive technology. That movement is the only sign of intent, and it is checked before every realign. Browser
// anchoring returns when the hold ends. A shift during a settle chain is corrected at once; otherwise a new chain starts.
// Budgets bound both; they reset once the target is aligned, a spent budget allows one more chain after 700ms of
// quiet (at most 30 per hold). A healthy aligned hold never expires. When those chains are spent and the target is still
// off the reading line, the hold ends: tracking resumes, so the label, index, URL and copy follow the visible text, and a
// notice says the requested position could not be kept. A later explicit navigation starts a fresh hold.
const layout=new ResizeObserver(()=>keepHeld());
function hold(index){held=index;realigns=0;nudges=0;recoveries=0;$('article').style.overflowAnchor='none';expectedTop=$('article').scrollTop;keepHeld();}
function unhold(){held=null;clearTimeout(settledTimer);$('article').style.overflowAnchor='';schedule();}
function readerMoved(){
  if(held===null)return false;
  const column=$('article'),top=column.scrollTop;
  if(Math.abs(top-expectedTop)<=2)return false;
  if(top<expectedTop&&top>=column.scrollHeight-column.clientHeight-1){expectedTop=top;return false;} // clamped by shrinking content
  intent();settleTarget=null;return true;
}
function keepHeld(){
  if(held===null||userScrolled||!flow.nodes[held]||readerMoved())return;
  const off=drift(held);
  if(Math.abs(off)>2){
    if(settleTarget!==null&&nudges<400){nudges++;const column=$('article');column.scrollTo({top:column.scrollTop+off,behavior:'instant'});expectedTop=column.scrollTop;}
    else if(settleTarget===null&&realigns<40){realigns++;scrollToIndex(held);}
    else if(realigns<=40&&nudges<=400){realigns=41;nudges=401;performance.mark('reader:hold-exhausted');}
  }
  clearTimeout(settledTimer);settledTimer=setTimeout(settled,700);
}
// The settled mark is informational: the held target is aligned with filling and fonts done. The hold continues.
function settled(){
  if(held===null||userScrolled)return;
  const aligned=settleTarget===null&&Math.abs(drift(held))<=2;
  if(aligned){realigns=0;nudges=0;}
  if(aligned&&!loading&&document.fonts.status==='loaded'){performance.mark('reader:target-settled');return;}
  if(!aligned&&settleTarget===null&&realigns>40){ // after a quiet spell, one more chain; at most 30 per hold
    if(recoveries<30){recoveries++;realigns=39;nudges=0;keepHeld();}
    else{performance.mark('reader:hold-recovery-exhausted');unhold();message('No se pudo mantener la posición solicitada; la selección sigue el texto visible.');}
    return;
  }
  settledTimer=setTimeout(settled,700);
}
function intent(){userScrolled=true;if(held!==null){unhold();performance.mark('reader:intent');}}
function scrolled(){readerMoved();schedule();}
// The selection follows the reading line, not observer membership: scrolling, resizing, filling and observer changes
// all schedule one pass per frame. A pass suppressed by a navigation lock retries when the lock expires.
function schedule(){if(!frame)frame=requestAnimationFrame(track);}
function track(){
  frame=0;clearTimeout(trailing);
  if(!flow.document||!selected||selected.document!==flow.document)return;
  const wait=lockUntil-performance.now();if(wait>0){trailing=setTimeout(schedule,wait);return;}
  if(settleTarget!==null||(loading&&!userScrolled)||(held!==null&&!userScrolled))return;
  const column=$('article').getBoundingClientRect().top;
  const index=activeIndex(flow.nodes.length,i=>flow.nodes[i].getBoundingClientRect().top-column,barHeight()+24);
  if(index>=0&&index!==selected.index)activate(flow.document,index,'scroll');
}

function activate(document_,index,trigger){
  const provision=document_.provisions[index];
  selected={document:document_,provision,index};
  $('crumb-instrument').textContent=document_.short_name;$('active-label').textContent=provision.label;
  $('instrument').value=document_.slug;$('instrument-summary').textContent=`${document_.official_title} · ${document_.provisions.length} disposiciones`;$('instrument-summary').title=document_.official_title;
  renderSource(document_,provision,trigger);markIndex();
  const prefix=`mx/${document_.slug}/`;$('markdown-link').href=inRelease(prefix+provision.markdown_path);$('text-link').href=inRelease(prefix+provision.text_path);
  const alternate=window.document.querySelector('link[rel="alternate"][type="text/markdown"]')||window.document.head.appendChild(Object.assign(window.document.createElement('link'),{rel:'alternate',type:'text/markdown'}));alternate.href=$('markdown-link').href;
  window.document.title=`${document_.short_name} · ${provision.label} — Lex-Mex`;
  const mode=historyMode(trigger),target=selectionUrl(location.href,release,document_,provision);
  clearTimeout(replaceTimer);
  if(mode==='push')history.pushState({},'',target);
  else if(mode==='replace')replaceTimer=setTimeout(()=>history.replaceState({},'',target),250);
}
async function go(document_,provision,trigger){
  const index=document_.provisions.indexOf(provision);userScrolled=false;held=null; // the previous target's hold ends here
  if(flow.document!==document_){if(trigger!=='search')$('search').value='';buildFlow(document_);renderIndex();}
  activate(document_,index,trigger);
  fillChunk(chunkOrder(document_.provisions.length,index)[0]);scrollToIndex(index);hold(index);
  if(flow.filled.size<document_.provisions.length)fillAll(index);
}

// Secondary view: the consulted provision is shown whole while the origin article and scroll position stay put.
function compare(target,trigger){
  origin={scrollTop:$('article').scrollTop,trigger};comparison=target;
  $('comparison-title').textContent=`${target.document.short_name} · ${target.provision.label}`;$('comparison-text').textContent=target.provision.text;
  $('comparison').hidden=false;$('close-comparison').focus();
}
function closeComparison(){
  if(!comparison)return;$('comparison').hidden=true;comparison=null;
  if(held!==null&&!userScrolled)scrollToIndex(held);else $('article').scrollTop=origin.scrollTop; // a held target may have been re-aligned meanwhile
  expectedTop=$('article').scrollTop;origin.trigger?.focus?.({preventScroll:true});
}
function selectFromUrl(trigger){
  const params=new URL(location.href).searchParams;const document_=documents.find(d=>d.slug===params.get('instrument'))||documents[0];
  const provision=document_.provisions.find(p=>p.id===params.get('provision'))||document_.provisions[0];if(!provision)throw new Error('Instrumento sin disposiciones publicadas.');
  return go(document_,provision,trigger);
}

// Compact layouts show index and source as sheets over the reading column.
function openSheet(id,opener){const sheet=$(id);sheet.classList.add('open');$('sheet-backdrop').hidden=false;sheet.dataset.opener=opener.id;sheet.querySelector('.sheet-close').focus();}
function closeSheets(){
  for(const sheet of document.querySelectorAll('.open[data-opener]')){sheet.classList.remove('open');$(sheet.dataset.opener)?.focus({preventScroll:true});delete sheet.dataset.opener;}
  $('sheet-backdrop').hidden=true;
}

async function verifyPdf(){const source=state.source,href=state.pdfUrl;$('verification-result').hidden=true;$('verification-result').textContent='';try{const response=await fetch(href);if(!response.ok)throw new Error('No se pudo leer el PDF.');const bytes=await response.arrayBuffer();const digest=Array.from(new Uint8Array(await crypto.subtle.digest('SHA-256',bytes)),b=>b.toString(16).padStart(2,'0')).join('');if(digest!==source.source_sha256)throw new Error('La copia PDF no coincide con el manifiesto.');const result=`PDF verificado. La huella criptográfica (SHA-256) de la copia PDF de la norma recuperada de ${source.official_url} coincide con la registrada para la fuente de esta publicación.`;if(state.pdfUrl===href){$('verification-result').textContent=result;$('verification-result').hidden=false;message('PDF verificado: coincide con la huella registrada de la fuente.');}}catch(error){if(state.pdfUrl===href)message(error.message);}}
function downloadMarkdown(text,name){const blob=new Blob([text],{type:'text/markdown;charset=utf-8'});const href=URL.createObjectURL(blob);const a=document.createElement('a');a.href=href;a.download=name;document.body.append(a);a.click();a.remove();setTimeout(()=>URL.revokeObjectURL(href),60000);}
async function init(){
  const params=new URL(location.href).searchParams;release=releaseId(params.get('release')||(await fetchJson(url('latest.json'))).release_id);
  manifest=await fetchJson(inRelease('manifest.json'));if(manifest.release_id!==release)throw new Error('El manifiesto no corresponde a la publicación solicitada.');
  documents=await Promise.all(manifest.instruments.map(i=>fetchJson(inRelease(i.document_path))));
  for(const item of documents){const option=document.createElement('option');option.value=item.slug;option.textContent=item.short_name;$('instrument').append(option);}
  $('release-tag').textContent=`Publicación ${release.slice(0,12)}`;$('manifest-link').href=inRelease('manifest.json');
  $('instrument').addEventListener('change',()=>{const document_=documents.find(d=>d.slug===$('instrument').value);$('search').value='';go(document_,document_.provisions[0],'instrument');});
  $('search').addEventListener('input',renderIndex);$('provision-list').addEventListener('keydown',indexKeys);
  $('close-comparison').onclick=closeComparison;$('comparison-citation').onclick=()=>copy(citation(comparison.document,comparison.provision),'Cita copiada.','No se pudo copiar la cita.');
  $('show-comparison').onclick=()=>{const target=comparison;closeComparison();go(target.document,target.provision,'reference');};
  $('share').onclick=()=>{const href=selectionUrl(location.href,release,selected.document,selected.provision).href;copy(href,'Enlace copiado con la publicación seleccionada.',href);};
  $('copy-text').onclick=()=>copy(articleCopy(selected.provision),'Artículo copiado: texto exacto completo.','No se pudo copiar. Selecciona el texto del artículo o descarga Texto exacto.');
  $('copy-citation').onclick=()=>copy(citation(selected.document,selected.provision),'Cita copiada.','No se pudo copiar la cita.');
  $('markdown-link').addEventListener('click',event=>{event.preventDefault();downloadMarkdown(selected.provision.text,`${selected.document.slug}-${selected.provision.markdown_path}`);});
  $('download-instrument').onclick=()=>downloadMarkdown(instrumentMarkdown(selected.document),`${selected.document.slug}.md`);
  $('about-open').onclick=()=>$('about-dialog').showModal();$('about-close').onclick=()=>$('about-dialog').close();
  $('index-open').onclick=event=>openSheet('index-sheet',event.currentTarget);$('source-open').onclick=event=>openSheet('source-sheet',event.currentTarget);
  for(const button of document.querySelectorAll('.sheet-close'))button.onclick=closeSheets;$('sheet-backdrop').onclick=closeSheets;
  compact.addEventListener('change',()=>{closeSheets();schedule();});window.addEventListener('resize',schedule);
  for(const id of ['flow','active-bar','load-status'])layout.observe($(id));document.fonts.addEventListener('loadingdone',()=>keepHeld());
  $('article').addEventListener('scroll',scrolled,{passive:true});
  $('verify-pdf').onclick=verifyPdf;$('source-offer-open').onclick=()=>state.offer&&loadSource(state.offer.source,state.offer.pdfUrl);
  window.addEventListener('popstate',()=>{const requested=new URL(location.href).searchParams.get('release');if(requested&&requested!==release){location.reload();return;}closeComparison();selectFromUrl('popstate');});
  window.addEventListener('keydown',event=>{
    if(event.key==='Escape'){if(comparison)closeComparison();else closeSheets();return;}
    if(!$('about-dialog').open&&event.key==='/'&&!['INPUT','TEXTAREA','SELECT'].includes(event.target.tagName)){event.preventDefault();$('search').focus();}
  });
  await selectFromUrl('entry');history.replaceState({},'',selectionUrl(location.href,release,selected.document,selected.provision));performance.mark('reader:ready');
}
init().catch(error=>{$('active-label').textContent='No se pudo abrir la colección';$('flow').textContent=error.message;message(error.message);});
