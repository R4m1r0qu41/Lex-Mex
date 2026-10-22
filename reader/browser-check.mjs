// Event-level browser regression for continuous reading. Drives a real headless Chrome over the DevTools
// protocol (no dependencies) against a packaged publication, and checks that the active label, index,
// exact copy body, share URL and replaced URL track the article at the reading line under real scroll
// events. Kept out of `node --test reader/` on purpose: it needs Chrome, and a silent skip would weaken
// that gate. Usage: node reader/browser-check.mjs --publication DIR [--chrome PATH] [--no-scroll-anchoring] [--out FILE]
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
  window.__pushes=[];const push=history.pushState.bind(history);history.pushState=(...args)=>{window.__pushes.push(String(args[2]));return push(...args);};
  Object.defineProperty(navigator,'clipboard',{configurable:true,value:{writeText:async text=>{window.__copied=text;}}});
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

export async function launch(chrome=defaultChrome,{anchoring=true}={}){
  const profile=await mkdtemp(join(tmpdir(),'lex-reader-chrome-'));
  const child=spawn(chrome,['--headless=new','--remote-debugging-port=0',`--user-data-dir=${profile}`,'--disable-smooth-scrolling','--no-first-run','--no-default-browser-check','--disable-background-timer-throttling','--disable-renderer-backgrounding','--window-size=1440,900','about:blank'],{stdio:'ignore'});
  let port;for(let i=0;i<200&&!port;i++){try{port=(await readFile(join(profile,'DevToolsActivePort'),'utf8')).split('\n')[0];}catch{await sleep(50);}}
  if(!port)throw new Error('Chrome did not expose a DevTools port.');
  const targets=await (await fetch(`http://127.0.0.1:${port}/json/list`)).json();const target=targets.find(t=>t.type==='page');
  const ws=new WebSocket(target.webSocketDebuggerUrl);await new Promise((ok,fail)=>{ws.onopen=ok;ws.onerror=fail;});
  const page=new Page(ws);await page.send('Page.enable');await page.send('Runtime.enable');await page.send('Page.addScriptToEvaluateOnNewDocument',{source:probes});
  // Without scroll anchoring (as in browsers that lack it) layout shifts above the view no longer emit scroll events.
  if(!anchoring)await page.send('Page.addScriptToEvaluateOnNewDocument',{source:"document.addEventListener('DOMContentLoaded',()=>document.head.append(Object.assign(document.createElement('style'),{textContent:'#article{overflow-anchor:none}'})));"});
  return {page,async close(){ws.close();child.kill();await sleep(200);await rm(profile,{recursive:true,force:true});}};
}

export async function runChecks({publication,chrome=defaultChrome,anchoring=true}){
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
  async function open(href,{width=1440,height=900}={}){await page.viewport(width,height);await page.send('Page.navigate',{url:href});await page.until('window.__probe&&__probe.ready()');await page.send('Page.resetNavigationHistory');await sleep(700);}
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
      await open(at(slug,p.id),{width,height});await sleep(500);const ui=await page.eval('__probe.ui()');const truth=list[await page.eval('__probe.truth()')];
      record(`last provision deep link ${slug} at ${width}x${height}`,ui.label===p.label&&truth.id===p.id&&ui.urlProvision===p.id,{ui:ui.label,truth:truth.label,urlProvision:ui.urlProvision===p.id});
    }
    // Indicative LIC load times on these bytes (navigation to flow complete); no budget is asserted.
    for(let i=0;i<3;i++){await page.viewport(1440,900);const t0=Date.now();await page.send('Page.navigate',{url:at('lic',find('Artículo 193').id)});await page.until('window.__probe&&__probe.ready()');timings.push({wall_ms:Date.now()-t0,...(await page.eval("Object.fromEntries(['reader:first-chunk','reader:flow-complete'].map(n=>[n,Math.round(performance.getEntriesByName(n)[0].startTime)]))"))});}
  }finally{await browser.close();await new Promise(r=>server.close(r));}
  return {publication,releaseId,origin,scrollAnchoring:anchoring,results,notApplicable,licLoadTimings:timings,passed:results.filter(r=>r.ok).length,failed:results.filter(r=>!r.ok).length};
}

if(process.argv[1]&&import.meta.url===pathToFileURL(resolve(process.argv[1])).href){
  const args=process.argv.slice(2),value=key=>args.includes(key)?args[args.indexOf(key)+1]:undefined;
  if(!value('--publication')){console.error('Usage: node reader/browser-check.mjs --publication DIR [--chrome PATH] [--no-scroll-anchoring] [--out FILE]');process.exit(1);}
  const report=await runChecks({publication:value('--publication'),chrome:value('--chrome')||defaultChrome,anchoring:!args.includes('--no-scroll-anchoring')});
  if(value('--out'))await writeFile(value('--out'),JSON.stringify(report,null,2)+'\n');
  console.log(`${report.passed} passed, ${report.failed} failed, ${report.notApplicable.length} not applicable`);process.exitCode=report.failed?1:0;
}
