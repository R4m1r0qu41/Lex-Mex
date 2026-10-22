import test from 'node:test';import assert from 'node:assert/strict';import {mkdtemp,writeFile,mkdir,rm} from 'node:fs/promises';import {tmpdir} from 'node:os';import {join} from 'node:path';
import {normalize,searchDocuments,safeRelative,releaseId,referencesFor,navigation,selectionUrl,indexGroup,instrumentMarkdown,chunkOrder,activeIndex,historyMode,citation,articleCopy,followState,matchSummary,estimateHeight,sourceDecision} from './reader-core.mjs';import {createReaderServer} from './serve.mjs';
const id='a'.repeat(64);const document={slug:'test',short_name:'LEY',provisions:[{id:'one',label:'Artículo 1',text:'Protección del crédito'},{id:'two',label:'Artículo 2 Bis',text:'Otra norma'}],references:[{source_provision_id:'one',target_provision_id:'two',resolution_status:'resolved'},{source_provision_id:'one',target_provision_id:'outside',resolution_status:'resolved'}]};
test('search, neighbors and pinned share URLs preserve suffix identities',()=>{
  assert.equal(normalize('CRÉDITO'),'credito');assert.equal(searchDocuments([document],'credito')[0].provision.id,'one');assert.equal(searchDocuments([document],'2 bis')[0].provision.id,'two');assert.equal(navigation(document,'one').next.id,'two');
  const url=selectionUrl('https://example.com/lex-mex/?old=1',id,document,document.provisions[1]);assert.equal(url.searchParams.get('release'),id);assert.equal(url.searchParams.get('provision'),'two');assert.equal(url.searchParams.has('old'),false);
});
test('relations distinguish published from excluded targets',()=>{const relations=referencesFor(document,document.provisions[0],[document]);assert.equal(relations[0].target.provision.id,'two');assert.equal(relations[1].target,undefined);});
test('Markdown preserves exact Unicode, whitespace and literal markup across article and annex boundaries',()=>{
 const text='Artículo único: ñ, á, ü.\n\n  Literal <texto> & \\u00e1 y [referencia].';
 const doc={official_title:'Norma de prueba',provisions:[{label:'Artículo 1',text},{label:'Anexo 1',text:'Última línea\n'}]};
 assert.equal(instrumentMarkdown(doc),'# Norma de prueba\n\n## Artículo 1\n\n'+text+'\n\n## Anexo 1\n\nÚltima línea\n');
 const p={provision_type:'article',heading_context:{title:'Título II',chapter:'Capítulo III'}};
 assert.equal(indexGroup({short_name:'LEY'},p),'Artículos · Título II · Capítulo III');
 assert.equal(indexGroup({short_name:'LEY'},p,true),'LEY');
 assert.equal(indexGroup({}, {provision_type:'annex',heading_context:{}}),'Anexos');
});
test('reject unsafe paths and unpinned release identities',()=>{for(const path of ['../secret','https://evil','/absolute','file%2fsecret','a\\b'])assert.throws(()=>safeRelative(path));assert.throws(()=>releaseId('latest'));assert.equal(safeRelative('mx/test/index.md'),'mx/test/index.md');});
test('preview serves text and PDF byte ranges and rejects wrong routes and methods',async t=>{
  const root=await mkdtemp(join(tmpdir(),'lex-reader-http-'));await writeFile(join(root,'index.html'),'<h1>Reader</h1>');await mkdir(join(root,'releases'));await writeFile(join(root,'releases','source.pdf'),'%PDF-1.7\nfixture');await writeFile(join(root,'releases','article.txt'),'Texto exacto');
  const server=createReaderServer(root);await new Promise(resolve=>server.listen(0,'127.0.0.1',resolve));t.after(async()=>{await new Promise(resolve=>server.close(resolve));await rm(root,{recursive:true,force:true});});const base=`http://127.0.0.1:${server.address().port}`;
  assert.equal((await fetch(base+'/lex-mex/')).status,200);assert.equal((await fetch(base+'/elsewhere')).status,404);assert.equal((await fetch(base+'/lex-mex/',{method:'POST'})).status,405);
  const pdf=await fetch(base+'/lex-mex/releases/source.pdf',{headers:{Range:'bytes=0-4'}});assert.equal(pdf.status,206);assert.equal(await pdf.text(),'%PDF-');assert.equal(pdf.headers.get('content-type'),'application/pdf');
  const text=await fetch(base+'/lex-mex/releases/article.txt');assert.equal(await text.text(),'Texto exacto');assert.match(text.headers.get('cache-control'),/immutable/);
  assert.equal((await fetch(base+'/lex-mex/releases/source.pdf',{headers:{Range:'bytes=1000-2000'}})).status,416);
});
test('chunks render the deep-link target first and then spread outward without gaps',()=>{
 assert.deepEqual(chunkOrder(100,95,40),[[80,100],[40,80],[0,40]]);assert.deepEqual(chunkOrder(100,0,40),[[0,40],[40,80],[80,100]]);assert.deepEqual(chunkOrder(0,0),[]);
 const covered=chunkOrder(511,300).flatMap(([start,end])=>Array.from({length:end-start},(_,i)=>start+i)).sort((a,b)=>a-b);assert.deepEqual(covered,Array.from({length:511},(_,i)=>i));
 assert.deepEqual(chunkOrder(10,999,4)[0],[8,10]);
});
test('active provision is the last article whose top crossed the reading line, found from positions alone',()=>{
 const at=tops=>[tops.length,i=>tops[i]];
 assert.equal(activeIndex(...at([]),100),-1);assert.equal(activeIndex(...at([-400,60,250]),100),1);
 assert.equal(activeIndex(...at([180,300]),100),0);assert.equal(activeIndex(...at([-900]),100),0);assert.equal(activeIndex(...at([-50,100,100,400]),100),2);
 // A heading crossing the line changes the answer even when no other article enters or leaves view.
 assert.equal(activeIndex(...at([-900,180]),124),0);assert.equal(activeIndex(...at([-1000,80]),124),1);
 const tops=Array.from({length:511},(_,i)=>i*300-20000);let reads=0;assert.equal(activeIndex(511,i=>{reads++;return tops[i];},124),67);assert.ok(reads<=10);
});
test('only deliberate navigation adds history entries',()=>{
 for(const trigger of ['index','reference','instrument','search'])assert.equal(historyMode(trigger),'push');
 for(const trigger of ['scroll','entry'])assert.equal(historyMode(trigger),'replace');assert.equal(historyMode('popstate'),'none');
});
test('article copy is the exact canonical body and citations name the consulted provision',()=>{
 const text='  Texto con\r\nsaltos\n\n y <literal> & espacios \u00a0finales  \n';assert.equal(articleCopy({text}),text);
 assert.equal(citation({short_name:'LRITF'},{label:'Artículo 4'}),'LRITF, artículo 4');assert.equal(citation({short_name:'LIC'},{label:'Artículo 112 Quintus'}),'LIC, artículo 112 Quintus');
 assert.equal(matchSummary(1),'1 disposición coincidente');assert.equal(matchSummary(3),'3 disposiciones coincidentes');
});
test('PDF following is reported unavailable and never infers a page',()=>{
 const follow=followState({pdf_page:7});assert.equal(follow.available,false);assert.match(follow.note,/Sin anclaje verificado/);
});
test('placeholder height grows with body length and counts every paragraph line',()=>{
 assert.ok(estimateHeight({text:'x'.repeat(5000)})>estimateHeight({text:'x'.repeat(100)}));assert.equal(estimateHeight({text:'a\n\nb'}),estimateHeight({text:'a\nb\nc'}));assert.ok(estimateHeight({text:''})>0);
});
test('passive scrolling never swaps the open PDF; deliberate navigation does',()=>{
 assert.equal(sourceDecision('entry',null,'a.pdf'),'swap');assert.equal(sourceDecision('scroll',{pdf:'a.pdf'},'a.pdf'),'keep');
 assert.equal(sourceDecision('scroll',{pdf:'a.pdf'},'b.pdf'),'offer');assert.equal(sourceDecision('scroll',{pdf:'a.pdf'},null),'offer');
 for(const trigger of ['index','search','reference','instrument','popstate','entry'])assert.equal(sourceDecision(trigger,{pdf:'a.pdf'},'b.pdf'),'swap');
 assert.equal(sourceDecision('index',{pdf:'a.pdf'},'a.pdf'),'keep');
});
