import test from 'node:test';
import assert from 'node:assert/strict';
import {createHash} from 'node:crypto';
import {agentPages} from './agent-pages.mjs';
const sha=b=>createHash('sha256').update(b).digest('hex');
function fixture(count=26){
  const id='a'.repeat(64),prefix=`releases/${id}/`,source={official_url:'https://example.org/source.pdf',publisher:'Official',source_sha256:'b'.repeat(64),retrieved_at:'2026-07-12'};
  const provisions=Array.from({length:count},(_,i)=>{const text=`Artículo ${i+1}: á & <script>alert("x")</script>\n  línea dos\n`;return {id:`urn:test:article:${i+1}`,label:`Artículo ${i+1}`,text,text_sha256:sha(text),text_path:`provision-${sha(''+i)}.txt`,markdown_path:`provision-${sha(''+i)}.md`,source,heading_context:{title:'Título I',chapter:'Capítulo II'},temporal_status:'effective'};});
  const doc={id:'urn:test',slug:'test',short_name:'TEST',official_title:'Test instrument',latest_reform_date:'2025-11-14',source,provisions};
  const manifest={release_id:id,base_path:'/lex-mex/',instruments:[{id:doc.id,slug:'test',document_path:'mx/test/instrument.json'}]};
  const content=new Map([[prefix+'mx/test/instrument.json',Buffer.from(JSON.stringify(doc))],[prefix+'manifest.json',Buffer.from('{}')]]);
  for(const p of provisions){content.set(prefix+'mx/test/'+p.text_path,Buffer.from(p.text));content.set(prefix+'mx/test/'+p.markdown_path,Buffer.from('Markdown'));}
  return {manifest,content,doc,prefix};
}
test('bounded guides preserve exact escaped article text, citations, sources and immutable release bytes',()=>{
 const {manifest,content,doc}=fixture();const before=[...content].map(([p,b])=>[p,sha(b)]);const out=agentPages(manifest,content);assert.deepEqual([...content].map(([p,b])=>[p,sha(b)]),before);assert.deepEqual(agentPages(manifest,content),out);
 const guides=[...out.files].filter(([p])=>p.includes('/guide-'));assert.equal(guides.length,2);assert.equal((guides[0][1].toString().match(/<li>/g)||[]).length,25);assert.equal((guides[1][1].toString().match(/<li>/g)||[]).length,1);
 const article=[...out.files].find(([p])=>p.endsWith('/'+sha(doc.provisions[0].id)+'.html'))[1].toString();assert.ok(article.includes('á &amp; &lt;script&gt;alert(&quot;x&quot;)&lt;/script&gt;\n  línea dos\n'));assert.ok(article.includes('urn:test:article:1'));assert.ok(article.includes('2025-11-14'));assert.ok(article.includes('2026-07-12'));assert.ok(!article.includes('<script>'));
 for(const [path,b] of out.files){for(const [,href] of b.toString().matchAll(/href="([^"]+)"/g)){if(href.startsWith('/')||href.startsWith('http'))continue;const resolved=new URL(href,'https://test/'+path).pathname.slice(1);assert.ok(out.files.has(resolved),`missing ${resolved}`);}}
});
test('inconsistent exact text and unverified document paths fail closed',()=>{
 const {manifest,content,doc,prefix}=fixture(1);content.set(prefix+'mx/test/'+doc.provisions[0].text_path,Buffer.from('wrong'));assert.throws(()=>agentPages(manifest,content),/text mismatch/);manifest.instruments[0].document_path='../private.json';assert.throws(()=>agentPages(manifest,content),/document path/);
});
test('large provision counts remain paginated instead of creating one unbounded guide',()=>{
 const {manifest,content}=fixture(10001);const out=agentPages(manifest,content);const guides=[...out.files].filter(([p])=>p.includes('/guide-'));assert.equal(guides.length,401);assert.ok(Math.max(...guides.map(([,b])=>b.length))<22000);
 const terms=[...out.files].filter(([p])=>p.includes('/terms/'));assert.ok(terms.length>0);for(const [,b] of terms)assert.ok((b.toString().match(/<li>/g)||[]).length<=20);
});

// Parsed-DOM text of the first <pre> under the HTML preprocessing and tree-construction
// rules that change bytes: raw CR/CRLF become LF, a newline directly after the <pre>
// start tag is dropped, and character references decode.
function parsedPreText(html){
 const raw=html.replace(/\r\n?/g,'\n');const m=/<pre[^>]*>([\s\S]*?)<\/pre>/.exec(raw);assert.ok(m,'pre element');
 let inner=m[1];if(inner.startsWith('\n'))inner=inner.slice(1);
 inner=inner.replace(/<\/?span[^>]*>/g,'');
 return inner.replace(/&(#\d+|amp|lt|gt|quot|#39);/g,(_,e)=>e==='amp'?'&':e==='lt'?'<':e==='gt'?'>':e==='quot'?'"':String.fromCodePoint(Number(e.slice(1))));
}
function exactFixture(text,url){
 const f=fixture(1);const p=f.doc.provisions[0];p.text=text;p.text_sha256=sha(Buffer.from(text));
 if(url)f.doc.source.official_url=url;
 f.content.set(f.prefix+'mx/test/instrument.json',Buffer.from(JSON.stringify(f.doc)));f.content.set(f.prefix+'mx/test/'+p.text_path,Buffer.from(text));return f;
}
test('parsed article text equals the canonical bytes for a leading newline, CRLF and bare CR',()=>{
 for(const text of ['\námbito\r\nsegunda línea','\n\nsolo saltos','uno\rdos\rtres','\r\ncrlf inicial','sin saltos especiales & <b>']){
  const {manifest,content,doc}=exactFixture(text);const out=agentPages(manifest,content);
  const article=[...out.files].find(([path])=>path.endsWith('/'+sha(doc.provisions[0].id)+'.html'))[1].toString();
  assert.equal(parsedPreText(article),text,JSON.stringify(text));
 }
});
test('official source URLs follow the shared policy: recorded http and https links, no credentials or other schemes',()=>{
 const render=url=>{const {manifest,content}=exactFixture('Texto',url);return agentPages(manifest,content);};
 assert.doesNotThrow(()=>render('http://www.example.org/recorded.pdf'));
 assert.doesNotThrow(()=>render('https://www.example.org/recorded.pdf'));
 for(const bad of ['https://user:secret@example.org/a.pdf','ftp://example.org/a.pdf','javascript:alert(1)','//example.org/a.pdf'])assert.throws(()=>render(bad),/Invalid official source URL/,bad);
});

function wordFixture(n,textOf){
 const f=fixture(n);
 f.doc.provisions.forEach((p,i)=>{p.text=textOf(i);p.text_sha256=sha(Buffer.from(p.text));f.content.set(f.prefix+'mx/test/'+p.text_path,Buffer.from(p.text));});
 f.content.set(f.prefix+'mx/test/instrument.json',Buffer.from(JSON.stringify(f.doc)));return f;
}
test('static term pages obey a corpus-wide budget, keep the most frequent words, and disclose it',()=>{
 // comun in all 12 provisions, medio in the even 6, tercio in every third (4), raroa/raroc once each
 const {manifest,content}=wordFixture(12,i=>`comun ${i%2===0?'medio':''} ${i%3===0?'tercio':''} ${i===1?'raroa':''} ${i===2?'raroc':''}`);
 const termFiles=out=>[...out.files].filter(([p])=>p.includes('/terms/')).map(([p])=>p.split('/terms/')[1]);
 const all=termFiles(agentPages(manifest,content,{maxTermPages:1_000_000}));
 assert.deepEqual(all.sort(),['comun.html','medio.html','raroa.html','raroc.html','tercio.html']);
 const small=agentPages(manifest,content,{maxTermPages:3});
 // most frequent first: comun (12), medio (6), tercio (4)
 assert.deepEqual(termFiles(small).sort(),['comun.html','medio.html','tercio.html']);
 // deterministic selection and file set
 assert.deepEqual([...agentPages(manifest,content,{maxTermPages:3}).files].map(([p])=>p),[...small.files].map(([p])=>p));
 // a tie is broken alphabetically: budget 4 keeps raroa, not raroc
 assert.deepEqual(termFiles(agentPages(manifest,content,{maxTermPages:4})).sort(),['comun.html','medio.html','raroa.html','tercio.html']);
 const catalog=[...small.files].find(([p])=>p.endsWith('/catalog-1.html'))[1].toString();
 assert.match(catalog,/las 3 palabras más frecuentes de 5/);
 assert.equal(termFiles(agentPages(manifest,content,{maxTermPages:0})).length,0);
 assert.throws(()=>agentPages(manifest,content,{maxTermPages:-1}),/Invalid term page budget/);
 assert.throws(()=>agentPages(manifest,content,{maxTermPages:1.5}),/Invalid term page budget/);
});
test('the default term budget bounds a large corpus',()=>{
 const {manifest,content}=fixture(10001);
 const out=agentPages(manifest,content);
 assert.ok([...out.files].filter(([p])=>p.includes('/terms/')).length<=5000);
});
