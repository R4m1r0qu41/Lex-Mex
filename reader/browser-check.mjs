// Event-level browser regression for continuous reading. Drives a real headless Chrome over the DevTools
// protocol (no dependencies) against a packaged publication, and checks that the active label, index,
// exact copy body, share URL and replaced URL track the article at the reading line under real scroll
// events. Kept out of `node --test reader/` on purpose: it needs Chrome, and a silent skip would weaken
// that gate. Usage: node reader/browser-check.mjs --publication DIR [--chrome PATH] [--no-scroll-anchoring] [--legacy-release ID] [--out FILE]
import {spawn} from 'node:child_process';
import {mkdtemp,readFile,rm,writeFile} from 'node:fs/promises';
import {tmpdir} from 'node:os';
import {join,resolve} from 'node:path';
import {pathToFileURL} from 'node:url';
import {createReaderServer} from './serve.mjs';

const sleep=ms=>new Promise(r=>setTimeout(r,ms));
const defaultChrome='/Applications/Google Chrome.app/Contents/MacOS/Google Chrome';

// Page-side probes, installed before the reader runs. They read the DOM only; the shipped bytes are untouched.
const probes=`(()=>{
  window.__scrolls=0;{const scrollTo=Element.prototype.scrollTo;Element.prototype.scrollTo=function(...args){window.__scrolls++;return scrollTo.apply(this,args);};}
  window.__errors=[];addEventListener('error',e=>window.__errors.push(String(e.message||e)));addEventListener('unhandledrejection',e=>window.__errors.push(String(e.reason)));
  window.__pushes=[];const push=history.pushState.bind(history);history.pushState=(...args)=>{window.__pushes.push(String(args[2]));return push(...args);};
  Object.defineProperty(navigator,'clipboard',{configurable:true,value:{writeText:async text=>{window.__copied=text;}}});
  // Exports and the exact-text tab go through blob URLs; record them (and swallow window.open) so scenarios can read what the reader produced.
  window.__blobs=[];{const create=URL.createObjectURL.bind(URL);URL.createObjectURL=blob=>{const href=create(blob);window.__blobs.push({href,type:blob.type,blob});return href;};}
  window.__opened=[];window.open=(href,target)=>{window.__opened.push({href,target});return null;};
  const col=()=>document.getElementById('article');
  const nodes=()=>[...document.querySelectorAll('#flow > article.provision')];
  window.__probe={
    line:()=>document.getElementById('active-bar').offsetHeight+24,
    top:i=>nodes()[i].getBoundingClientRect().top-col().getBoundingClientRect().top,
    height:i=>nodes()[i].getBoundingClientRect().height,
    count:()=>nodes().length,
    // The article whose top last crossed the reading line, computed from layout alone.
    truth(){const line=this.line(),base=col().getBoundingClientRect().top;let best=0;for(const n of nodes()){if(n.getBoundingClientRect().top-base<=line)best=Number(n.dataset.index);else break;}return best;},
    // Which articles intersect the observer root (upper half of the reading column's padding box).
    membership(){const c=col(),r=c.getBoundingClientRect(),top=r.top+c.clientTop,bottom=top+c.clientHeight*0.5;return nodes().filter(n=>{const b=n.getBoundingClientRect();return b.bottom>top&&b.top<bottom;}).map(n=>Number(n.dataset.index));},
    ui(){const current=document.querySelector('#provision-list button[aria-current="true"]');const params=new URL(location.href).searchParams;return {label:document.getElementById('active-label').textContent,current:current?.dataset.provision??null,title:document.title,instrument:document.getElementById('instrument').value,pushes:window.__pushes.length,urlProvision:params.get('provision'),urlInstrument:params.get('instrument'),history:history.length};},
    scrollBy(dy){col().scrollTop+=dy;},
    ready:()=>performance.getEntriesByName('reader:flow-complete').length>0,
    firstChunk:()=>performance.getEntriesByName('reader:first-chunk').length>0,
    frames:()=>new Promise(r=>requestAnimationFrame(()=>requestAnimationFrame(r))),
    async copy(id){window.__copied=undefined;document.getElementById(id).click();for(let i=0;i<50&&window.__copied===undefined;i++)await new Promise(r=>setTimeout(r,10));return window.__copied;}
  };
})();`;

export class Page {
  constructor(ws){this.ws=ws;this.id=0;this.pending=new Map();ws.addEventListener('message',event=>{const msg=JSON.parse(event.data);if(msg.id&&this.pending.has(msg.id)){const {resolve,reject}=this.pending.get(msg.id);this.pending.delete(msg.id);msg.error?reject(new Error(msg.error.message)):resolve(msg.result);}});}
  send(method,params={}){const id=++this.id;this.ws.send(JSON.stringify({id,method,params}));return new Promise((resolve,reject)=>this.pending.set(id,{resolve,reject}));}
  async eval(expression){const result=await this.send('Runtime.evaluate',{expression,awaitPromise:true,returnByValue:true});if(result.exceptionDetails)throw new Error(result.exceptionDetails.exception?.description||result.exceptionDetails.text);return result.result.value;}
  async until(expression,timeout=15000){const end=Date.now()+timeout;while(Date.now()<end){if(await this.eval(expression))return;await sleep(25);}throw new Error(`Timed out waiting for ${expression}`);}
  wheel(x,y,deltaY){return this.send('Input.dispatchMouseEvent',{type:'mouseWheel',x,y,deltaX:0,deltaY});}
  viewport(width,height){return this.send('Emulation.setDeviceMetricsOverride',{width,height,deviceScaleFactor:1,mobile:false});}
}

export async function launch(chrome=defaultChrome,{anchoring=true,fonts='normal',smooth=false}={}){
  const profile=await mkdtemp(join(tmpdir(),'lex-reader-chrome-'));
  const child=spawn(chrome,['--headless=new','--remote-debugging-port=0',`--user-data-dir=${profile}`,...(smooth?[]:['--disable-smooth-scrolling']),'--no-first-run','--no-default-browser-check','--disable-background-timer-throttling','--disable-renderer-backgrounding','--window-size=1440,900','about:blank'],{stdio:'ignore'});
  let port;for(let i=0;i<200&&!port;i++){try{port=(await readFile(join(profile,'DevToolsActivePort'),'utf8')).split('\n')[0];}catch{await sleep(50);}}
  if(!port)throw new Error('Chrome did not expose a DevTools port.');
  const targets=await (await fetch(`http://127.0.0.1:${port}/json/list`)).json();const target=targets.find(t=>t.type==='page');
  const ws=new WebSocket(target.webSocketDebuggerUrl);await new Promise((ok,fail)=>{ws.onopen=ok;ws.onerror=fail;});
  const page=new Page(ws);await page.send('Page.enable');await page.send('Runtime.enable');await page.send('Page.addScriptToEvaluateOnNewDocument',{source:probes});
  // Without scroll anchoring (as in browsers that lack it) layout shifts above the view no longer emit scroll events.
  // Web fonts can be blocked or delayed to reproduce late layout shifts (the stylesheet imports Google Fonts).
  const fontUrls=['*fonts.googleapis.com*','*fonts.gstatic.com*'];
  if(fonts==='blocked'){await page.send('Network.enable');await page.send('Network.setBlockedURLs',{urls:fontUrls});}
  else if(typeof fonts==='number'){await page.send('Fetch.enable',{patterns:fontUrls.map(urlPattern=>({urlPattern}))});ws.addEventListener('message',event=>{const msg=JSON.parse(event.data);if(msg.method==='Fetch.requestPaused')setTimeout(()=>page.send('Fetch.continueRequest',{requestId:msg.params.requestId}).catch(()=>{}),fonts);});}
  if(!anchoring)await page.send('Page.addScriptToEvaluateOnNewDocument',{source:"document.addEventListener('DOMContentLoaded',()=>document.head.append(Object.assign(document.createElement('style'),{textContent:'#article{overflow-anchor:none}'})));"});
  return {page,async close(){ws.close();child.kill();await sleep(200);await rm(profile,{recursive:true,force:true});}};
}

export async function runChecks({publication,chrome=defaultChrome,anchoring=true,legacyRelease=null}){
  publication=resolve(publication);
  const releaseId=JSON.parse(await readFile(join(publication,'latest.json'),'utf8')).release_id;
  const manifest=JSON.parse(await readFile(join(publication,'releases',releaseId,'manifest.json'),'utf8'));
  const instruments={};for(const item of manifest.instruments)instruments[item.slug]=JSON.parse(await readFile(join(publication,'releases',releaseId,item.document_path),'utf8'));
  const server=createReaderServer(publication);await new Promise(r=>server.listen(0,'127.0.0.1',r));
  const origin=`http://127.0.0.1:${server.address().port}/lex-mex/`;
  const browser=await launch(chrome,{anchoring});const page=browser.page;
  const results=[],notApplicable=[],timings=[];
  const record=(scenario,ok,detail)=>{results.push({scenario,ok,...detail});console.log(`${ok?'PASS':'FAIL'} ${scenario}${ok?'':' '+JSON.stringify(detail)}`);};
  const at=(slug,provisionId='')=>`${origin}?release=${releaseId}&instrument=${slug}${provisionId?`&provision=${encodeURIComponent(provisionId)}`:''}`;
  // Push counts come from a pushState wrapper; resetting history keeps back() inside each scenario.
  // Scenarios start once an explicit target has settled (older builds have no such mark and get a bounded wait).
  const mark=name=>`performance.getEntriesByName('${name}').length`;
  const pageErrors=[],unsettled=[];
  async function errors(p,where){const list=await p.eval('window.__errors||[]');if(list.length)pageErrors.push({where,list});}
  async function settle(p=page,timeout=10000){await p.until('window.__probe&&__probe.ready()',20000);return p.until(`${mark('reader:target-settled')}>0`,timeout).then(()=>true,()=>false);}
  async function open(href,{width=1440,height=900}={}){await page.viewport(width,height);await page.send('Page.navigate',{url:href});if(!await settle(page,3000))unsettled.push(href);await page.send('Page.resetNavigationHistory');await sleep(700);await errors(page,href);}
  // Compares every UI surface with the article at the reading line.
  async function agree(slug,{url=true}={}){
    const truth=await page.eval('__probe.truth()'),ui=await page.eval('__probe.ui()'),p=instruments[slug].provisions[truth];
    const mismatches=[];if(ui.label!==p.label)mismatches.push('label');if(ui.current!==p.id)mismatches.push('index');if(!ui.title.includes(p.label))mismatches.push('title');if(url&&ui.urlProvision!==p.id)mismatches.push('url');
    return {truth:p.label,ui:ui.label,mismatches};
  }
  async function copies(slug){const truth=await page.eval('__probe.truth()'),p=instruments[slug].provisions[truth];const body=await page.eval("__probe.copy('copy-text')");const share=await page.eval("__probe.copy('share')");return {copyExact:body===p.text,shareProvision:new URL(share).searchParams.get('provision')===p.id};}
  const centre=async()=>page.eval("(()=>{const r=document.getElementById('article').getBoundingClientRect();return {x:Math.round(r.left+r.width/2),y:Math.round(r.top+r.height*0.6)};})()");
  const lic=instruments.lic.provisions,find=label=>lic.find(p=>p.label===label);
  try{
    // 1. Reviewer geometry: the next heading crosses the reading line while observer membership is constant.
    for(const mode of ['scrollTop','wheel']){
      const ordered=lic.map((p,i)=>({i,len:p.text.length})).filter(x=>x.i>0&&x.i<lic.length-1);
      let done=false;
      for(const {i:b} of ordered.sort((x,y)=>y.len-x.len).slice(0,25)){
        // Start from a deep link to A (a legitimate selection), then bring B's heading to line+56 and wait out the lock.
        await open(at('lic',lic[b-1].id));
        const line=await page.eval('__probe.line()');const offset=await page.eval(`__probe.top(${b})`);
        await page.eval(`__probe.scrollBy(${offset-(line+56)})`);await sleep(500);await page.eval('__probe.frames()');
        const before=await page.eval('__probe.membership()');
        if(!(before.length===2&&before[0]===b-1&&before[1]===b))continue;
        const start=await agree('lic');
        const trace=[];const {x,y}=await centre();
        for(let step=0;step<40&&(await page.eval(`__probe.top(${b})`))>line-44;step++){
          if(mode==='wheel')await page.wheel(x,y,8);else await page.eval('__probe.scrollBy(8)');
          await page.eval('__probe.frames()');
          trace.push({top:Math.round(await page.eval(`__probe.top(${b})`)),members:(await page.eval('__probe.membership()')).join(','),...(await agree('lic',{url:false}))});
        }
        const constant=trace.every(t=>t.members===before.join(','));const crossed=trace.some(t=>t.top<=line)&&trace[0].top>line;
        // The replaced URL lags by design (250ms); the copy and share actions read the selection immediately.
        const immediate=await copies('lic');await sleep(400);const end=await agree('lic');const copy=await copies('lic');
        const lagging=trace.filter(t=>t.mismatches.length);
        record(`reading-line crossing without observer change (${mode})`,constant&&crossed&&start.mismatches.length===0&&lagging.length===0&&end.mismatches.length===0&&immediate.copyExact&&immediate.shareProvision&&copy.copyExact&&copy.shareProvision,{pair:[lic[b-1].label,lic[b].label],line,constantMembership:constant,crossed,start,end,immediateCopy:immediate,copy,laggingSteps:lagging.length,trace});
        done=true;break;
      }
      if(!done)record(`reading-line crossing without observer change (${mode})`,false,{error:'no LIC pair satisfied the constant-membership precondition'});
    }
    // 2. Short adjacent articles: per-step agreement while many headings pass quickly.
    {
      let run=0,startAt=-1;for(let i=0;i<lic.length;i++){if(lic[i].text.length<120){run++;if(run===6){startAt=i-5;break;}}else run=0;}
      await open(at('lic',lic[Math.max(0,startAt-1)].id));const {x,y}=await centre();const bad=[];
      for(let step=0;step<30;step++){await page.wheel(x,y,40);await page.eval('__probe.frames()');const a=await agree('lic',{url:false});if(a.mismatches.length)bad.push({step,...a});}
      await sleep(400);const end=await agree('lic');
      record('short adjacent articles',bad.length===0&&end.mismatches.length===0,{from:lic[startAt]?.label,badSteps:bad,end});
    }
    // 3. Long article tail: scroll through the longest LIC body until the next heading passes the line.
    {
      const longest=lic.reduce((a,p,i)=>p.text.length>lic[a].text.length&&i<lic.length-1?i:a,0);
      await open(at('lic',lic[longest].id));const {x,y}=await centre();const bad=[];const line=await page.eval('__probe.line()');
      for(let step=0;step<400&&(await page.eval(`__probe.top(${longest+1})`))>line-60;step++){await page.wheel(x,y,60);await page.eval('__probe.frames()');const a=await agree('lic',{url:false});if(a.mismatches.length)bad.push({step,...a});}
      await sleep(400);const end=await agree('lic');const copy=await copies('lic');
      record('long article tail',bad.length===0&&end.mismatches.length===0&&end.truth===lic[longest+1].label&&copy.copyExact&&copy.shareProvision,{article:lic[longest].label,badSteps:bad.slice(0,5),badCount:bad.length,end,copy});
    }
    // 4. Resize across the 1100px breakpoint changes the bar height and the reading line without a scroll.
    {
      await open(at('lic',find('Artículo 193').id));const {x,y}=await centre();for(let i=0;i<9;i++)await page.wheel(x,y,40);await sleep(500);
      const wide=await agree('lic');await page.viewport(900,900);await sleep(600);const narrow=await agree('lic');await page.viewport(1440,900);await sleep(600);const back=await agree('lic');
      record('resize across breakpoint',[wide,narrow,back].every(a=>a.mismatches.length===0),{wide,narrow,back});
    }
    // 5. Deep links settle on the explicit target, including the recovered 112 articles and the last transitory.
    for(const [slug,label] of [['lic','Artículo 112 Quintus'],['lic','Artículo 112 Sextus'],['lic','Artículo 112 Séptimus'],['lic','Artículo 193'],['lic',lic[lic.length-1].label],['lritf','Artículo 4']]){
      const p=instruments[slug].provisions.find(q=>q.label===label);
      if(!p){notApplicable.push(`deep link ${slug} ${label}: provision not in this release`);console.log(`N/A  deep link ${slug} ${label} (not in this release)`);continue;}
      await open(at(slug,p.id),{width:1440,height:900});await sleep(500);
      const ui=await page.eval('__probe.ui()'),truth=instruments[slug].provisions[await page.eval('__probe.truth()')];const index=instruments[slug].provisions.indexOf(p);
      const offset=Math.round(await page.eval(`__probe.top(${index})-document.getElementById('active-bar').offsetHeight`));
      record(`deep link ${slug} ${label}`,ui.label===label&&truth.id===p.id&&ui.urlProvision===p.id&&Math.abs(offset-8)<=3,{ui:ui.label,truth:truth.label,urlProvision:ui.urlProvision===p.id,offsetBelowBar:offset});
    }
    // 6. A wheel during the post-navigation lock ends on the article at the line, with one history entry for the click.
    {
      await open(at('lic',find('Artículo 10').id));const before=(await page.eval("__probe.ui()")).pushes;const {x,y}=await centre();
      await page.eval(`document.querySelector('#provision-list button[data-provision="${find('Artículo 50').id}"]').click()`);
      for(let i=0;i<6;i++){await page.wheel(x,y,60);await sleep(15);}
      await sleep(1200);const end=await agree('lic');const ui=await page.eval('__probe.ui()');
      record('wheel during navigation lock',end.mismatches.length===0&&ui.pushes===before+1,{end,pushes:ui.pushes-before});
    }
    // 7. Index click pushes one entry, is not overwritten by a trailing pass, and back restores the previous article.
    {
      await open(at('lic',find('Artículo 2o').id));const before=(await page.eval("__probe.ui()")).pushes;const target=find('Artículo 46');
      await page.eval(`document.querySelector('#provision-list button[data-provision="${target.id}"]').click()`);await sleep(1200);
      const ui=await page.eval('__probe.ui()');const a=await agree('lic');
      await page.eval('history.back()');await sleep(1200);const backUi=await page.eval('__probe.ui()');const b=await agree('lic');
      record('index push, no trailing overwrite, back',ui.label===target.label&&ui.urlProvision===target.id&&ui.pushes===before+1&&a.mismatches.length===0&&backUi.label==='Artículo 2o'&&b.mismatches.length===0,{afterClick:ui.label,pushes:ui.pushes-before,url:ui.urlProvision===target.id,back:backUi.label});
    }
    // 8. Interrupted settle: a wheel while a deep link is still filling ends on the article at the line.
    {
      const p=lic[lic.length-40];await page.viewport(1440,900);await page.send('Page.navigate',{url:at('lic',p.id)});await page.until('window.__probe&&__probe.firstChunk()');
      const {x,y}=await centre();for(let i=0;i<5;i++){await page.wheel(x,y,-80);await sleep(10);}
      await page.until('__probe.ready()');await sleep(900);const end=await agree('lic');
      record('interrupted deep-link settle',end.mismatches.length===0,{target:p.label,end});
    }
    // 9. Two quick instrument switches end on the last instrument's first provision, with no stray selection.
    {
      await open(at('lic',find('Artículo 193').id));const before=(await page.eval("__probe.ui()")).pushes;
      await page.eval(`(()=>{const s=document.getElementById('instrument');s.value='ifpe-dcg-2021';s.dispatchEvent(new Event('change'));s.value='itf-dcg-2018';s.dispatchEvent(new Event('change'));})()`);
      await page.until('__probe.ready()');await sleep(1200);const ui=await page.eval('__probe.ui()');const a=await agree('itf-dcg-2018');const first=instruments['itf-dcg-2018'].provisions[0];
      record('quick instrument switches',ui.instrument==='itf-dcg-2018'&&ui.urlInstrument==='itf-dcg-2018'&&ui.label===first.label&&a.mismatches.length===0&&ui.pushes===before+2,{ui:ui.label,instrument:ui.instrument,pushes:ui.pushes-before,pushes:await page.eval('window.__pushes'),agree:a});
    }
    // 10. Narrow layout: a deep link lands under the compact bar and the label matches the line.
    {
      const p=find('Artículo 193');await open(at('lic',p.id),{width:390,height:844});await sleep(500);const a=await agree('lic');
      const overflow=await page.eval('document.documentElement.scrollWidth>document.documentElement.clientWidth');
      record('narrow 390px deep link',a.mismatches.length===0&&a.truth===p.label&&!overflow,{agree:a,horizontalOverflow:overflow});
    }
    // 11. Search result: one push, the chosen provision becomes active and stays active.
    {
      await open(at('lic',find('Artículo 2o').id));
      await page.eval(`(()=>{const s=document.getElementById('search');s.value='Artículo 112';s.dispatchEvent(new Event('input'));})()`);await page.eval('__probe.frames()');
      const hit=await page.eval(`(()=>{const b=document.querySelector('#provision-list button[data-slug="lic"]');b.click();return b.dataset.provision;})()`);
      await sleep(1200);const ui=await page.eval('__probe.ui()');const a=await agree('lic');
      record('search result navigation',ui.urlProvision===hit&&ui.pushes===1&&a.mismatches.length===0,{label:ui.label,pushes:ui.pushes,agree:a});
    }
    // 12. Reference view keeps the origin article and scroll; Volver restores them.
    {
      const source=lic.find(p=>instruments.lic.references.some(r=>r.source_provision_id===p.id&&r.resolution_status==='resolved'&&lic.some(q=>q.id===r.target_provision_id)));
      await open(at('lic',source.id));const before=await page.eval("({top:document.getElementById('article').scrollTop,ui:__probe.ui()})");
      const opened=await page.eval(`(()=>{const a=document.querySelector('#flow article[data-index="${lic.indexOf(source)}"] .article-text a');if(!a)return false;a.click();return !document.getElementById('comparison').hidden;})()`);
      await sleep(300);const during=await page.eval("({top:document.getElementById('article').scrollTop,ui:__probe.ui()})");
      await page.eval("document.getElementById('close-comparison').click()");await sleep(300);const after=await page.eval("({top:document.getElementById('article').scrollTop,ui:__probe.ui()})");
      record('reference view and Volver',opened&&during.ui.label===source.label&&Math.abs(during.top-before.top)<2&&Math.abs(after.top-before.top)<2&&after.ui.label===source.label&&during.ui.pushes===0,{source:source.label,opened,scroll:[before.top,during.top,after.top]});
    }
    // 13. PDF following stays unavailable; passive scrolling into another source offers, never swaps, the open PDF.
    {
      const ifpe=instruments['ifpe-dcg-2021'].provisions;const annex=ifpe.findIndex((p,i)=>i>0&&p.source.source_sha256!==ifpe[i-1].source.source_sha256);
      await open(at('ifpe-dcg-2021',ifpe[annex-1].id));const pdfBefore=await page.eval("document.getElementById('pdf').getAttribute('src')");
      const disabled=await page.eval("document.getElementById('pdf-follow').disabled");const {x,y}=await centre();
      for(let i=0;i<200&&(await page.eval('__probe.truth()'))<annex;i++){await page.wheel(x,y,60);await page.eval('__probe.frames()');}
      await sleep(400);const pdfAfter=await page.eval("document.getElementById('pdf').getAttribute('src')");const offer=await page.eval("!document.getElementById('source-offer').hidden");const a=await agree('ifpe-dcg-2021');
      record('PDF offer without auto-follow',disabled&&pdfBefore===pdfAfter&&offer&&a.mismatches.length===0,{followDisabled:disabled,pdfKept:pdfBefore===pdfAfter,offer,agree:a});
    }
    // 14. A wheel that ends inside the navigation lock, followed by no further events, still updates once the lock expires.
    {
      const pair=lic.findIndex((p,i)=>i>0&&i<lic.length-1&&p.text.length>150&&p.text.length<420&&lic[i+1].text.length>3000);
      await open(at('lic',lic[pair>0?pair-1:0].id));const {x,y}=await centre();
      await page.eval(`document.querySelector('#provision-list button[data-provision="${lic[pair].id}"]').click()`);
      const line=await page.eval('__probe.line()');let delta=Math.round(await page.eval(`__probe.top(${pair+1})`)-line+30);
      while(delta>0){const step=Math.min(120,delta);await page.wheel(x,y,step);delta-=step;}
      await sleep(900);const end=await agree('lic');const copy=await copies('lic');
      record('wheel ending inside the navigation lock',end.mismatches.length===0&&end.truth===lic[pair+1].label&&copy.copyExact&&copy.shareProvision,{clicked:lic[pair].label,end,copy});
    }
    // 15. Explicit targets at the end of each instrument reach the reading line and keep the selection, even on tall viewports.
    for(const [width,height] of [[1440,3000],[390,844]])for(const slug of Object.keys(instruments)){
      const list=instruments[slug].provisions,p=list[list.length-1];
      await open(at(slug,p.id),{width,height});await sleep(500);const ui=await page.eval('__probe.ui()');const truth=list[await page.eval('__probe.truth()')];const intents=await page.eval("performance.getEntriesByName('reader:intent').length");
      record(`last provision deep link ${slug} at ${width}x${height}`,ui.label===p.label&&truth.id===p.id&&ui.urlProvision===p.id&&intents===0,{ui:ui.label,truth:truth.label,urlProvision:ui.urlProvision===p.id,intentMarks:intents});
    }
    // 16. Requested-target stability with no reader input: deferred sizes and late web fonts move the target after the
    // jump. It must end on the requested article, settle (mark asserted), and no intent may be inferred from the browser's
    // own scroll-anchoring adjustments. Fresh browsers per condition.
    const fontsLoaded=async p=>p.until("document.fonts.status==='loaded'",16000).catch(()=>{});
    const stability=async(label,rid,list,slug)=>{
      const matrix=[];for(const fonts of ['normal',1500,'blocked'])for(let rep=0;rep<3;rep++)matrix.push([390,844,fonts,rep]);
      matrix.push([390,844,10000,0]);for(const fonts of ['normal',1500,'blocked',10000])matrix.push([1440,900,fonts,0]);
      for(const [width,height,fonts,rep] of matrix){
        const b=await launch(chrome,{anchoring,fonts});const p=list.find(q=>q.label===label);const index=list.indexOf(p);
        try{
          await b.page.viewport(width,height);const t0=Date.now();await b.page.send('Page.navigate',{url:`${origin}?release=${rid}&instrument=${slug}&provision=${encodeURIComponent(p.id)}`});
          await b.page.until('window.__probe&&__probe.ready()',20000);
          await b.page.eval(`(()=>{window.__drift={frames:0,over50:0,max:0};const bar=document.getElementById('active-bar');const tick=()=>{const off=Math.abs(__probe.top(${index})-bar.offsetHeight-8);__drift.frames++;if(off>50)__drift.over50++;__drift.max=Math.max(__drift.max,Math.round(off));if(!performance.getEntriesByName('reader:target-settled').length)requestAnimationFrame(tick);};requestAnimationFrame(tick);})()`);
          await fontsLoaded(b.page);const isSettled=await settle(b.page,8000);const settleMs=Date.now()-t0;await sleep(600);
          const ui=await b.page.eval('__probe.ui()'),truth=list[await b.page.eval('__probe.truth()')];
          const offset=Math.round(await b.page.eval(`__probe.top(${index})-document.getElementById('active-bar').offsetHeight`));
          const copy=await b.page.eval("__probe.copy('copy-text')");const intents=await b.page.eval(mark('reader:intent'));const exhausted=await b.page.eval(mark('reader:hold-exhausted'));const drift=await b.page.eval('window.__drift');
          await errors(b.page,`stability ${label} ${width} ${fonts}`);
          record(`requested target stable ${slug} ${label} ${width}px fonts=${fonts} #${rep+1}${rid===releaseId?'':' (legacy release)'}`,ui.label===label&&truth.id===p.id&&ui.urlProvision===p.id&&copy===p.text&&Math.abs(offset-8)<=3&&isSettled&&intents===0&&exhausted===0,{ui:ui.label,truth:truth.label,urlProvision:ui.urlProvision===p.id,copyExact:copy===p.text,offsetBelowBar:offset,settled:isSettled,settleMs,intentMarks:intents,exhausted,paintedDriftAfterReady:drift});
        }finally{await b.close();}
      }
    };
    await stability('Artículo 193',releaseId,lic,'lic');
    // 17. Regression guard (also passes on builds without a hold): a wheel during a held target wins over later font swaps.
    // 19. A scroll the page cannot attribute to input (find in page, focus, assistive technology; simulated by setting the
    //     scroll position) also ends the hold: no snap-back when fonts land, and tracking resumes.
    // 20. Pressing "Copiar enlace" during a held target is not scroll intent: the target, label, URL and copy survive the swap.
    {
      const p=find('Artículo 193');
      const during=async(name,act,check,p=find('Artículo 193'),smooth=false)=>{const b=await launch(chrome,{anchoring,fonts:1500,smooth});try{
        await b.page.viewport(390,844);await b.page.send('Page.navigate',{url:at('lic',p.id)});await b.page.until('window.__probe&&__probe.ready()',20000);
        const loadingAtAct=await b.page.eval("document.fonts.status");await act(b.page);await fontsLoaded(b.page);await sleep(1500);
        const ui=await b.page.eval('__probe.ui()'),truth=lic[await b.page.eval('__probe.truth()')];const intents=await b.page.eval(mark('reader:intent'));const copy=await b.page.eval("__probe.copy('copy-text')");
        const offset=Math.round(await b.page.eval(`__probe.top(${lic.indexOf(p)})-document.getElementById('active-bar').offsetHeight`));
        await errors(b.page,name);record(name,check({ui,truth,intents,copy,offset})&&loadingAtAct==='loading',{ui:ui.label,truth:truth.label,url:ui.urlProvision===truth.id,intentMarks:intents,offsetBelowBar:offset,fontsAtAction:loadingAtAct});
      }finally{await b.close();}};
      const box="(()=>{const r=document.getElementById('article').getBoundingClientRect();return {x:Math.round(r.left+r.width/2),y:Math.round(r.top+r.height*0.6)};})()";
      await during('reader scroll during a held target is preserved',async b=>{const c=await b.eval(box);for(let i=0;i<8;i++){await b.wheel(c.x,c.y,-120);await sleep(15);}},({ui,truth})=>ui.label===truth.label&&ui.urlProvision===truth.id&&truth.id!==p.id);
      await during('unattributed scroll during a held target ends the hold',async b=>{await b.eval('__probe.scrollBy(-900)');},({ui,truth,intents,copy})=>ui.label===truth.label&&ui.urlProvision===truth.id&&truth.id!==p.id&&intents>0&&copy===truth.text);
      await during('pressing Copiar enlace during a held target keeps the target',async b=>{const r=await b.eval("(()=>{const r=document.getElementById('share').getBoundingClientRect();return {x:Math.round(r.left+r.width/2),y:Math.round(r.top+r.height/2)};})()");for(const type of ['mousePressed','mouseReleased'])await b.send('Input.dispatchMouseEvent',{type,x:r.x,y:r.y,button:'left',clickCount:1});},({ui,truth,intents,copy})=>ui.label===p.label&&truth.id===p.id&&ui.urlProvision===p.id&&intents===0&&copy===p.text);
    // 22. Real input during a held target: PageDown (keyboard scroll) is intent and is not undone; a 2px touch tap is not.
    // 23. Volver from a reference opened during the hold returns to the held target, not to a stale scroll position.
      await during('PageDown during a held target is intent and is kept',async b=>{await b.eval("document.getElementById('article').focus({preventScroll:true})");for(const type of ['rawKeyDown','keyUp'])await b.send('Input.dispatchKeyEvent',{type,key:'PageDown',code:'PageDown',windowsVirtualKeyCode:34,nativeVirtualKeyCode:34});await sleep(200);},({ui,truth,intents,offset})=>ui.label===truth.label&&ui.urlProvision===truth.id&&intents>0&&Math.abs(offset-8)>50);
      await during('touch tap during a held target keeps the target',async b=>{await b.send('Emulation.setTouchEmulationEnabled',{enabled:true});const c=await b.eval(box);await b.send('Input.dispatchTouchEvent',{type:'touchStart',touchPoints:[{x:c.x,y:c.y}]});await b.send('Input.dispatchTouchEvent',{type:'touchMove',touchPoints:[{x:c.x,y:c.y+2}]});await b.send('Input.dispatchTouchEvent',{type:'touchEnd',touchPoints:[]});},({ui,truth,intents,copy})=>ui.label===p.label&&truth.id===p.id&&ui.urlProvision===p.id&&intents===0&&copy===p.text);
      await during('smooth-scrolling PageDown during a held target is intent and is kept',async b=>{await b.eval("document.getElementById('article').focus({preventScroll:true})");for(const type of ['rawKeyDown','keyUp'])await b.send('Input.dispatchKeyEvent',{type,key:'PageDown',code:'PageDown',windowsVirtualKeyCode:34,nativeVirtualKeyCode:34});await sleep(600);},({ui,truth,intents,offset})=>ui.label===truth.label&&ui.urlProvision===truth.id&&intents>0&&Math.abs(offset-8)>50,find('Artículo 193'),true);
      const linked=lic.find((q,i)=>i>150&&instruments.lic.references.some(r=>r.source_provision_id===q.id&&r.resolution_status==='resolved'&&lic.some(t=>t.id===r.target_provision_id)&&Array.from(q.text).slice(r.start_char,r.end_char).join('')===r.source_span));
      await during('Volver after a reference opened during a held target returns to the target',async b=>{const opened=await b.eval(`(()=>{const a=document.querySelector('#flow article[data-index="${lic.indexOf(linked)}"] .article-text a');if(!a)return false;a.click();return !document.getElementById('comparison').hidden;})()`);if(!opened)throw new Error('reference link did not open');await b.until("document.fonts.status==='loaded'",8000).catch(()=>{});await sleep(800);await b.eval("document.getElementById('close-comparison').click()");await sleep(500);},({ui,truth,intents})=>ui.label===linked.label&&truth.id===linked.id&&ui.urlProvision===linked.id&&intents===0,linked);
    }
    // 24. A resize drag across the breakpoint during a held target keeps it (budget never exhausted, no intent inferred).
    {
      const b=await launch(chrome,{anchoring});const p=find('Artículo 193');try{
        await b.page.viewport(1440,900);await b.page.send('Page.navigate',{url:at('lic',p.id)});await settle(b.page);
        for(const w of [1300,1200,1100,1000,900,800,900,1050,1200,1440]){await b.page.viewport(w,900);await sleep(300);}
        await sleep(1200);const ui=await b.page.eval('__probe.ui()'),truth=lic[await b.page.eval('__probe.truth()')];const intents=await b.page.eval(mark('reader:intent'));const exhausted=await b.page.eval(mark('reader:hold-exhausted'));
        await errors(b.page,'resize drag');record('resize drag during a held target keeps it',ui.label===p.label&&truth.id===p.id&&ui.urlProvision===p.id&&intents===0&&exhausted===0,{ui:ui.label,truth:truth.label,intentMarks:intents,exhausted});
      }finally{await b.close();}
    }
    // 26. A long resize storm exhausts the realign budget (exhaustion lands ~5.5s in; 800 steps run ~18s); once it stops, the held target is realigned within a quiet spell.
    {
      const b=await launch(chrome,{anchoring});const p=find('Artículo 193');try{
        await b.page.viewport(1440,900);await b.page.send('Page.navigate',{url:at('lic',p.id)});await settle(b.page);
        const t0=Date.now();for(let i=0;i<800;i++){await b.page.viewport(i%2?1440:1380+(i%7)*5,900);await sleep(18);}const stormMs=Date.now()-t0;
        await b.page.viewport(1200,900);await sleep(2500);const before=await b.page.eval('window.__scrolls');await sleep(3000);const idle=await b.page.eval('window.__scrolls')-before;
        const ui=await b.page.eval('__probe.ui()'),truth=lic[await b.page.eval('__probe.truth()')];const exhausted=await b.page.eval(mark('reader:hold-exhausted'));const intents=await b.page.eval(mark('reader:intent'));
        const offset=Math.round(await b.page.eval(`__probe.top(${lic.indexOf(p)})-document.getElementById('active-bar').offsetHeight`));
        await errors(b.page,'resize storm');record('long resize storm: budget bounded, target realigned after quiet',exhausted>=1&&idle===0&&ui.label===p.label&&truth.id===p.id&&ui.urlProvision===p.id&&intents===0&&Math.abs(offset-8)<=3,{ui:ui.label,truth:truth.label,offsetBelowBar:offset,exhaustedMarks:exhausted,scrollsWhileIdleAfterRecovery:idle,stormMs,intentMarks:intents});
      }finally{await b.close();}
    }
    // 27. Terminal recovery-cap boundary (fault injection, NOT natural-storm coverage; scenario 26 is that). reader.js is a
    // module, so a CDP breakpoint in keepHeld sets the lifetime counters to their spent values while a synthetic layout
    // change leaves the held target off the reading line. No test hook is added to the shipped bytes. The hold must end:
    // one exhaustion mark, a visible notice, label/index/URL/copy agreeing with the reading line, no correction loop,
    // reader movement still tracked without intent, and a fresh explicit navigation holding and settling again.
    for(const pendingFill of [false,true]){
      const b=await launch(chrome,{anchoring,fonts:'blocked'});const p=find('Artículo 193'),index=lic.indexOf(p);try{
        const sources=new Map();b.page.ws.addEventListener('message',event=>{const msg=JSON.parse(event.data);if(msg.method==='Debugger.scriptParsed'&&/\/reader\.js$/.test(msg.params.url))sources.set(msg.params.scriptId,msg.params.url);});
        // Pause the real fillAll zero-delay yield after its first chunk; leave all other timers running.
        if(pendingFill)await b.page.send('Page.addScriptToEvaluateOnNewDocument',{source:`(()=>{
          const original=setTimeout;let paused=true;const pending=[];
          window.__resumeFill=()=>{paused=false;for(const run of pending.splice(0))run();};
          window.setTimeout=(fn,delay,...args)=>paused&&delay===0?original(()=>pending.push(()=>fn(...args)),0):original(fn,delay,...args);
        })();`});
        await b.page.viewport(390,844);await b.page.send('Page.navigate',{url:at('lic',p.id)});
        let settledFirst;
        if(pendingFill){await b.page.until('window.__probe&&__probe.firstChunk()');await sleep(1000);settledFirst=await b.page.eval('!__probe.ready()');}
        else settledFirst=await settle(b.page,15000);
        const before=await b.page.eval('__probe.ui()');
        await b.page.send('Debugger.enable');const [scriptId]=[...sources.keys()];
        const lines=(await b.page.send('Debugger.getScriptSource',{scriptId})).scriptSource.split('\n');const bodyLine=lines.findIndex(line=>line.startsWith('function keepHeld(){'))+1;
        const breakpoint=await b.page.send('Debugger.setBreakpointByUrl',{urlRegex:'/reader\\.js$',lineNumber:bodyLine});
        const paused=new Promise((resolve,reject)=>{const handler=async event=>{const msg=JSON.parse(event.data);if(msg.method!=='Debugger.paused')return;b.page.ws.removeEventListener('message',handler);
          try{await b.page.send('Debugger.evaluateOnCallFrame',{callFrameId:msg.params.callFrames[0].callFrameId,expression:'realigns=41;nudges=401;recoveries=30;settleTarget=null;0'});await b.page.send('Debugger.removeBreakpoint',{breakpointId:breakpoint.breakpointId});await b.page.send('Debugger.resume');resolve();}catch(error){reject(error);}};
          b.page.ws.addEventListener('message',handler);});
        await b.page.eval("document.querySelector('#flow article').style.paddingBottom='1400px';0");
        await Promise.race([paused,sleep(10000).then(()=>{throw new Error('keepHeld breakpoint never paused');})]);
        await sleep(2000); // 700ms settled timer, then the URL replace delay
        const notice=await b.page.eval("(()=>{const m=document.getElementById('message');return {visible:!m.hidden,text:m.textContent};})()"); // read before Copiar replaces the message
        const truth=lic[await b.page.eval('__probe.truth()')],ui=await b.page.eval('__probe.ui()'),copy=await b.page.eval("__probe.copy('copy-text')");
        const exhausted=await b.page.eval(mark('reader:hold-recovery-exhausted')),intents=await b.page.eval(mark('reader:intent'));
        const idleStart=await b.page.eval('window.__scrolls');await sleep(1800);const idle=await b.page.eval('window.__scrolls')-idleStart;
        await b.page.eval('__probe.scrollBy(100)');await sleep(700);
        const movedTruth=lic[await b.page.eval('__probe.truth()')],moved=await b.page.eval('__probe.ui()'),movedCopy=await b.page.eval("__probe.copy('copy-text')"),intentsAfterMove=await b.page.eval(mark('reader:intent'));
        const stillPending=pendingFill?await b.page.eval('!__probe.ready()'):true;
        let completion={ok:true};
        if(pendingFill){
          const scrolls=await b.page.eval('window.__scrolls');await b.page.eval('__resumeFill();0');await b.page.until('__probe.ready()');await sleep(1000);
          const completedTruth=lic[await b.page.eval('__probe.truth()')],completed=await b.page.eval('__probe.ui()'),completedCopy=await b.page.eval("__probe.copy('copy-text')");
          const corrections=await b.page.eval('window.__scrolls')-scrolls;
          completion={ok:completed.current===completedTruth.id&&completed.label===completedTruth.label&&completed.urlProvision===completedTruth.id&&completedCopy===completedTruth.text&&corrections===0,truth:completedTruth.label,ui:completed.label,copyExact:completedCopy===completedTruth.text,corrections};
        }
        const settledMarks=await b.page.eval(mark('reader:target-settled'));
        await b.page.eval(`document.querySelector('#provision-list button[data-provision="${p.id}"]').click();0`);
        await b.page.until(`${mark('reader:target-settled')}>${settledMarks}`,10000);await sleep(700);
        const renav=await b.page.eval('__probe.ui()'),renavTruth=lic[await b.page.eval('__probe.truth()')],renavCopy=await b.page.eval("__probe.copy('copy-text')");
        const renavOffset=Math.round(await b.page.eval(`__probe.top(${index})-document.getElementById('active-bar').offsetHeight`));
        const renavStart=await b.page.eval('window.__scrolls');await sleep(1800);const renavIdle=await b.page.eval('window.__scrolls')-renavStart;
        await errors(b.page,'terminal recovery cap');
        record(`terminal recovery cap (fault injection, ${pendingFill?'fill pending then resumed':'fill complete'}): unaligned hold ends with a notice, tracking resumes, new navigation re-holds`,
          stillPending&&completion.ok&&settledFirst&&before.label===p.label&&truth.id!==p.id&&ui.label===truth.label&&ui.current===truth.id&&ui.urlProvision===truth.id&&copy===truth.text&&notice.visible&&notice.text.includes('No se pudo mantener')&&exhausted===1&&intents===0&&idle===0
          &&movedTruth.id===moved.current&&moved.label===movedTruth.label&&moved.urlProvision===movedTruth.id&&movedCopy===movedTruth.text&&intentsAfterMove===0
          &&renav.label===p.label&&renavTruth.id===p.id&&renav.current===p.id&&renav.urlProvision===p.id&&renavCopy===p.text&&Math.abs(renavOffset-8)<=3&&renavIdle===0&&await b.page.eval(mark('reader:hold-recovery-exhausted'))===1,
          {pendingFill,stillPending,completion,requested:p.label,readingLine:truth.label,ui:ui.label,index:ui.current===truth.id,urlProvision:ui.urlProvision===truth.id,copyExact:copy===truth.text,notice,exhaustedMarks:exhausted,intentMarks:intents,scrollsWhileIdle:idle,afterReaderScroll:{truth:movedTruth.label,ui:moved.label,copyExact:movedCopy===movedTruth.text,intentMarks:intentsAfterMove},afterNewNavigation:{ui:renav.label,truth:renavTruth.label,copyExact:renavCopy===p.text,offsetBelowBar:renavOffset,scrollsWhileIdle:renavIdle}});
      }finally{await b.close();}
    }
    // 28. View options and exports (R7): citation name toggle (remembered, shared with the hover window), exact article copy
    // unchanged by it, heading path in the middle column, index group and both Markdown exports, the exact-text tab with its
    // encoding stated, and the right-panel structure.
    {
      const b=await launch(chrome,{anchoring});const p=find('Artículo 104'),index=lic.indexOf(p),path=`${p.heading_context.title} · ${p.heading_context.chapter}`,title=instruments.lic.official_title;
      const first=lic.find(q=>q.heading_context.title===p.heading_context.title&&q.heading_context.chapter===p.heading_context.chapter);try{ // the instrument export prints a heading once per path, before its first article
        await b.page.viewport(1440,900);await b.page.send('Page.navigate',{url:at('lic',p.id)});await settle(b.page);
        const short=await b.page.eval("__probe.copy('copy-citation')");await b.page.eval("document.getElementById('name-toggle').click()");
        const full=await b.page.eval("__probe.copy('copy-citation')"),exact=await b.page.eval("__probe.copy('copy-text')");
        await b.page.send('Page.navigate',{url:at('lic',p.id)});await settle(b.page);const remembered=await b.page.eval("__probe.copy('copy-citation')");
        const source=lic.find(q=>instruments.lic.references.some(r=>r.source_provision_id===q.id&&r.resolution_status==='resolved'&&lic.some(t=>t.id===r.target_provision_id)));
        await b.page.send('Page.navigate',{url:at('lic',source.id)});await settle(b.page);
        const opened=await b.page.eval(`(()=>{const a=document.querySelector('#flow article[data-index="${lic.indexOf(source)}"] .article-text a');if(!a)return false;a.click();return !document.getElementById('comparison').hidden;})()`);
        const heading=await b.page.eval("document.getElementById('comparison-title').textContent"),targetLabel=heading.split(' · ').slice(1).join(' · ');
        const target=Object.values(instruments).flatMap(d=>d.provisions.map(q=>({q,d}))).find(({q,d})=>`${d.short_name} · ${q.label}`===heading);
        const hoverCopy=await b.page.eval("__probe.copy('comparison-copy')"),hoverCite=await b.page.eval("__probe.copy('comparison-citation')");
        await b.page.eval("document.getElementById('close-comparison').click()");await b.page.send('Page.navigate',{url:at('lic',p.id)});await settle(b.page);
        await b.page.eval("document.getElementById('markdown-link').click()");const md=await b.page.eval("(async()=>window.__blobs.at(-1).blob.text())()");
        await b.page.eval("document.getElementById('download-instrument').click()");const whole=await b.page.eval("(async()=>window.__blobs.at(-1).blob.text())()");
        await b.page.eval("document.getElementById('text-link').click()");
        const tab=await b.page.eval("(async()=>{const o=window.__opened.at(-1);const blob=window.__blobs.find(x=>x.href===o.href);return {type:blob.type,text:await blob.blob.text(),target:o.target,count:window.__opened.length};})()");
        const group=await b.page.eval(`(()=>{let n=document.querySelector('#provision-list button[aria-current="true"]');while(n&&!n.classList.contains('group-heading'))n=n.previousElementSibling;return n?n.textContent:null;})()`);
        const middle=await b.page.eval(`document.querySelector('#flow article[data-index="${index}"] .heading-context').textContent`);
        const panel=await b.page.eval(`({verifyInside:Boolean(document.querySelector('.source-bottom details #verify-pdf')),reformVisible:!document.getElementById('reform-date').closest('details'),reform:document.getElementById('reform-date').textContent,agent:(a=>({target:a.target,rel:a.rel,text:a.textContent}))(document.getElementById('agent-index')),textLink:document.getElementById('text-link').target})`);
        await b.page.eval("document.getElementById('name-toggle').click()");const back=await b.page.eval("__probe.copy('copy-citation')");
        await errors(b.page,'view options');
        record('view options and exports: citation toggle, hover copy, headings in index/middle/Markdown, exact-text tab, right panel',
          short==='LIC, artículo 104'&&full===`${title}, artículo 104`&&exact===p.text&&remembered===full&&back===short&&opened&&target&&hoverCopy===target.q.text&&hoverCite===`${target.d.official_title}, ${target.q.label.charAt(0).toLowerCase()}${target.q.label.slice(1)}`
          &&md===`## ${path}\n\n### ${p.label}\n\n${p.text}`&&whole.startsWith(`# ${title}\n\n`)&&whole.includes(`## ${path}\n\n### ${first.label}\n\n${first.text}`)&&whole.includes(`\n\n### ${p.label}\n\n${p.text}`)&&(whole.match(new RegExp(`\n## ${path.replace(/[.*+?^${}()|[\]\\]/g,'\\$&')}\n`,'g'))||[]).length===1
          &&tab.type==='text/plain;charset=utf-8'&&tab.text===p.text&&tab.target==='_blank'&&group===path&&middle===path
          &&panel.verifyInside&&panel.reformVisible&&panel.reform.includes(instruments.lic.latest_reform_date)&&panel.agent.target==='_blank'&&panel.agent.text.includes('llms.txt')&&panel.textLink==='_blank',
          {short,full,exact:exact===p.text,remembered:remembered===full,hover:{heading,copyExact:target?hoverCopy===target.q.text:null,cite:hoverCite},markdownStart:md.slice(0,80),wholeHasHeading:whole.includes(`## ${path}\n\n### ${first.label}`),firstInPath:first.label,tab:{type:tab.type,exact:tab.text===p.text,target:tab.target},group,middle,panel});
      }finally{await b.close();}
    }
    // 29. Text size toggle (R7) at 390px with fonts blocked: the article at the reading line stays there (label, index, URL and
    // exact copy agree with the physical reading line), no hold is exhausted, no correction loop runs, the preference
    // survives a reload, and toggling back works the same.
    {
      const b=await launch(chrome,{anchoring,fonts:'blocked'});const p=find('Artículo 193'),index=lic.indexOf(p);try{
        const size=()=>b.page.eval(`parseFloat(getComputedStyle(document.querySelector('#flow article[data-index="${index}"] .article-text')).fontSize)`);
        const state=async(label,expectRatio,base)=>{
          await sleep(1800);const ui=await b.page.eval('__probe.ui()'),truth=lic[await b.page.eval('__probe.truth()')],copy=await b.page.eval("__probe.copy('copy-text')");
          const offset=Math.round(await b.page.eval(`__probe.top(${index})-document.getElementById('active-bar').offsetHeight`)),px=await size();
          const exhausted=await b.page.eval(mark('reader:hold-exhausted'))+await b.page.eval(mark('reader:hold-recovery-exhausted')),intents=await b.page.eval(mark('reader:intent'));
          const idleStart=await b.page.eval('window.__scrolls');await sleep(1500);const idle=await b.page.eval('window.__scrolls')-idleStart;
          const ok=ui.label===p.label&&truth.id===p.id&&ui.current===p.id&&ui.urlProvision===p.id&&copy===p.text&&Math.abs(offset-8)<=3&&exhausted===0&&intents===0&&idle===0&&Math.abs(px/base-expectRatio)<0.02;
          return {label,ok,ui:ui.label,truth:truth.label,copyExact:copy===p.text,offsetBelowBar:offset,fontPx:px,ratio:Math.round(px/base*100)/100,exhaustedMarks:exhausted,intentMarks:intents,scrollsWhileIdle:idle};
        };
        await b.page.viewport(390,844);await b.page.send('Page.navigate',{url:at('lic',p.id)});await settle(b.page);const base=await size();
        const steps=[await state('large (start)',1,base)];
        let settled=await b.page.eval(mark('reader:target-settled'));await b.page.eval("document.getElementById('size-toggle').click()");
        await b.page.until(`${mark('reader:target-settled')}>${settled}`,10000);steps.push(await state('small',0.65,base));
        await b.page.send('Page.navigate',{url:at('lic',p.id)});await settle(b.page);steps.push(await state('small after reload (remembered)',0.65,base));
        settled=await b.page.eval(mark('reader:target-settled'));await b.page.eval("document.getElementById('size-toggle').click()");
        await b.page.until(`${mark('reader:target-settled')}>${settled}`,10000);steps.push(await state('large again',1,base));
        await errors(b.page,'text size toggle');
        record('text size toggle at 390px keeps the reading-line article, is remembered, and reverses',steps.every(step=>step.ok),{steps});
      }finally{await b.close();}
    }
    // 25. Default entry (first provision, no provision in the URL) and a selector switch to another instrument's first
    // provision settle, never exhaust the realign budget, and leave no realigning running afterwards.
    for(const [width,height] of [[1440,900],[390,844]]){
      const b=await launch(chrome,{anchoring});try{
        await b.page.viewport(width,height);await b.page.send('Page.navigate',{url:`${origin}?release=${releaseId}&instrument=lic`});const isSettled=await settle(b.page,10000);
        await sleep(1000);const before=await b.page.eval('window.__scrolls');await sleep(3000);const after=await b.page.eval('window.__scrolls');
        const ui=await b.page.eval('__probe.ui()');const exhausted=await b.page.eval(mark('reader:hold-exhausted'));
        await b.page.eval(`(()=>{const s=document.getElementById('instrument');s.value='lritf';s.dispatchEvent(new Event('change'));})()`);
        await b.page.until(`${mark('reader:target-settled')}>1`,10000).catch(()=>{});await sleep(1000);const before2=await b.page.eval('window.__scrolls');await sleep(3000);const after2=await b.page.eval('window.__scrolls');
        const ui2=await b.page.eval('__probe.ui()'),settledCount=await b.page.eval(mark('reader:target-settled')),exhausted2=await b.page.eval(mark('reader:hold-exhausted')),total=await b.page.eval('window.__scrolls');
        await errors(b.page,`first provision ${width}`);
        record(`first-provision entry and instrument switch settle at ${width}px`,isSettled&&exhausted===0&&after===before&&ui.label===lic[0].label&&ui2.instrument==='lritf'&&ui2.label===instruments.lritf.provisions[0].label&&settledCount>=2&&exhausted2===0&&after2===before2,{entry:ui.label,settled:isSettled,scrollsWhileIdle:after-before,switched:ui2.label,settledMarks:settledCount,scrollsWhileIdleAfterSwitch:after2-before2,totalScrollCalls:total,exhausted:exhausted+exhausted2});
      }finally{await b.close();}
    }
    // 21. Explicit navigation other than entry at 390px while fonts are late: an index choice pushes once and holds; back restores.
    {
      const b=await launch(chrome,{anchoring,fonts:1500});const target=find('Artículo 193');try{
        await b.page.viewport(390,844);await b.page.send('Page.navigate',{url:at('lic',find('Artículo 2o').id)});await b.page.until('window.__probe&&__probe.ready()',20000);
        const centreOf=async sel=>b.page.eval(`(()=>{const r=document.querySelector('${sel}').getBoundingClientRect();return {x:Math.round(r.left+r.width/2),y:Math.round(r.top+r.height/2)};})()`);
        const click=async sel=>{const c=await centreOf(sel);for(const type of ['mousePressed','mouseReleased'])await b.page.send('Input.dispatchMouseEvent',{type,x:c.x,y:c.y,button:'left',clickCount:1});};
        await click('#index-open');await sleep(300);await b.page.eval(`document.querySelector('#provision-list button[data-provision="${target.id}"]').scrollIntoView({block:'center'})`);await sleep(100);
        await click(`#provision-list button[data-provision="${target.id}"]`);await fontsLoaded(b.page);await sleep(1500);
        const ui=await b.page.eval('__probe.ui()'),truth=lic[await b.page.eval('__probe.truth()')];
        await b.page.eval('history.back()');await sleep(1500);const back=await b.page.eval('__probe.ui()'),backTruth=lic[await b.page.eval('__probe.truth()')];
        await errors(b.page,'index at 390');
        record('index choice at 390px with late fonts holds, pushes once, back restores',ui.label===target.label&&truth.id===target.id&&ui.urlProvision===target.id&&ui.pushes===1&&back.label==='Artículo 2o'&&backTruth.label==='Artículo 2o'&&back.urlProvision===find('Artículo 2o').id,{ui:ui.label,truth:truth.label,pushes:ui.pushes,back:back.label,backTruth:backTruth.label});
      }finally{await b.close();}
    }
    // 18. Legacy pinned release served next to the current one (preferred staging keeps the older release published).
    if(legacyRelease){
      const legacy=JSON.parse(await readFile(join(publication,'releases',legacyRelease,'manifest.json'),'utf8'));const doc=legacy.instruments.find(i=>i.slug==='lic');
      const list=JSON.parse(await readFile(join(publication,'releases',legacyRelease,doc.document_path),'utf8')).provisions;
      await stability('Artículo 193',legacyRelease,list,'lic');
      const b=await launch(chrome,{anchoring});try{await b.page.viewport(390,844);const p=list.find(q=>q.label==='Artículo 193');await b.page.send('Page.navigate',{url:`${origin}?release=${legacyRelease}&instrument=lic&provision=${encodeURIComponent(p.id)}`});await settle(b.page);const tag=await b.page.eval("document.getElementById('release-tag').textContent");const share=await b.page.eval("__probe.copy('share')");
        record('legacy release link keeps its pinned release',tag.includes(legacyRelease.slice(0,12))&&new URL(share).searchParams.get('release')===legacyRelease,{tag,shareRelease:new URL(share).searchParams.get('release')});}finally{await b.close();}
    }
    // Indicative LIC load times on these bytes (navigation to flow complete); no budget is asserted.
    for(let i=0;i<3;i++){await page.viewport(1440,900);const t0=Date.now();await page.send('Page.navigate',{url:at('lic',find('Artículo 193').id)});await page.until('window.__probe&&__probe.ready()');timings.push({wall_ms:Date.now()-t0,...(await page.eval("Object.fromEntries(['reader:first-chunk','reader:flow-complete'].map(n=>[n,Math.round(performance.getEntriesByName(n)[0].startTime)]))"))});}
    record('no page errors (exceptions, ResizeObserver loops)',pageErrors.length===0,{pageErrors});
    record('every opened page settles its explicit target',unsettled.length===0,{unsettled});
  }finally{await browser.close();await new Promise(r=>server.close(r));}
  return {publication,releaseId,origin,scrollAnchoring:anchoring,results,notApplicable,licLoadTimings:timings,passed:results.filter(r=>r.ok).length,failed:results.filter(r=>!r.ok).length};
}

if(process.argv[1]&&import.meta.url===pathToFileURL(resolve(process.argv[1])).href){
  const args=process.argv.slice(2),value=key=>args.includes(key)?args[args.indexOf(key)+1]:undefined;
  if(!value('--publication')){console.error('Usage: node reader/browser-check.mjs --publication DIR [--chrome PATH] [--no-scroll-anchoring] [--legacy-release ID] [--out FILE]');process.exit(1);}
  const report=await runChecks({publication:value('--publication'),chrome:value('--chrome')||defaultChrome,anchoring:!args.includes('--no-scroll-anchoring'),legacyRelease:value('--legacy-release')||null});
  if(value('--out'))await writeFile(value('--out'),JSON.stringify(report,null,2)+'\n');
  console.log(`${report.passed} passed, ${report.failed} failed, ${report.notApplicable.length} not applicable`);process.exitCode=report.failed?1:0;
}
