import {writeFile,readFile} from 'node:fs/promises';
import {join} from 'node:path';
import assert from 'node:assert/strict';
import {Client} from '@modelcontextprotocol/sdk/client/index.js';
import {StreamableHTTPClientTransport} from '@modelcontextprotocol/sdk/client/streamableHttp.js';
const [origin,output,host]=process.argv.slice(2),results=[];
if(!origin||!output||!host)throw Error('Expected preview origin, output JSON and candidate host directory');
const pin=JSON.parse(await readFile(join(host,'lex-mex-release.json')));
const client=new Client({name:'lex-remote-runtime-proof',version:'1'});
try {
 await client.connect(new StreamableHTTPClientTransport(new URL('/lex-mex/mcp',origin)));
 const tools=(await client.listTools()).tools.map(t=>t.name);assert.equal(tools.length,4);
 const article=await client.callTool({name:'get_provision',arguments:{release_id:pin.release_id,instrument:'lritf',type:'article',number:'46'}});
 assert.ok(!article.isError);assert.ok(article.structuredContent.result.data.provision.text.includes('segregados'));
 for(const path of ['/','/en','/lex-mex/','/lex-mex/current.json','/lex-mex/agents/index.html','/sitemap.xml','/lex-mex/agents/sitemap.xml',`/lex-mex/r/${pin.release_id}/mx/lritf/article/46`]) {
  const start=performance.now(),r=await fetch(new URL(path,origin)),body=await r.text();assert.equal(r.status,200,path);
  if(path.endsWith('sitemap.xml'))assert.ok((body.match(/<url>/g)||[]).length>=870);
  results.push({path,status:r.status,bytes:Buffer.byteLength(body),elapsed_ms:Math.round(performance.now()-start)});
 }
 const hostile=await fetch(new URL('/lex-mex/mcp',origin),{method:'POST',headers:{Origin:'https://hostile.example','Content-Type':'application/json'},body:'{}'});assert.equal(hostile.status,403);await hostile.arrayBuffer();
 await writeFile(output,JSON.stringify({origin,version:'834b5d37-284d-4550-8b52-90be61a980ed',publication_id:pin.publication_id,tools,article46_retrieved:true,hostile_origin_status:hostile.status,results},null,2)+'\n');
 console.log(JSON.stringify({passed:true,tools,routes:results.length,output}));
} finally {await client.close();}
