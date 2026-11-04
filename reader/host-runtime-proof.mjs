// Exact built-module and asset proof, local only. Args: HOST_DIR TOOL_NODE_MODULES OUTPUT_JSON
import {readdir,readFile,writeFile} from 'node:fs/promises';import {resolve,join} from 'node:path';import {pathToFileURL} from 'node:url';
import {Client} from '@modelcontextprotocol/sdk/client/index.js';import {StreamableHTTPClientTransport} from '@modelcontextprotocol/sdk/client/streamableHttp.js';
const [hostArg,toolsArg,output]=process.argv.slice(2);if(!hostArg||!toolsArg||!output)throw Error('Expected host, tooling node_modules and proof output');
const host=resolve(hostArg),root=join(host,'dist/server'),files=[];
async function walk(dir){for(const f of await readdir(dir,{withFileTypes:true})){const path=join(dir,f.name);if(f.isDirectory())await walk(path);else if(f.name.endsWith('.js'))files.push(path);}}
await walk(root);files.sort((a,b)=>a===join(root,'index.js')?-1:b===join(root,'index.js')?1:a.localeCompare(b));
const config=JSON.parse(await readFile(join(root,'wrangler.json'))),{Miniflare}=await import(pathToFileURL(join(resolve(toolsArg),'miniflare/dist/src/index.js')));
const mf=new Miniflare({modules:files.map(path=>({type:'ESModule',path})),modulesRoot:root,ratelimits:Object.fromEntries((config.ratelimits||[]).map(({name,namespace_id,simple})=>[name,{namespace_id,simple}])),compatibilityDate:config.compatibility_date,compatibilityFlags:config.compatibility_flags,assets:{directory:join(host,'dist/client'),binding:'ASSETS',routerConfig:{has_user_worker:true,static_routing:{user_worker:config.assets.run_worker_first}}}});
const client=new Client({name:'lex-built-host-proof',version:'1'}),results=[];
try{
 await client.connect(new StreamableHTTPClientTransport(new URL('https://example.test/lex-mex/mcp'),{fetch:(url,init)=>mf.dispatchFetch(url,init)}));
 const tools=(await client.listTools()).tools.map(t=>t.name),pin=JSON.parse(await readFile(join(host,'lex-mex-release.json')));
 const result=await client.callTool({name:'get_provision',arguments:{release_id:pin.release_id,instrument:'lritf',type:'article',number:'46'}});
 if(result.isError||!result.structuredContent.result.data.provision.text.includes('segregados'))throw Error('Wrong MCP evidence');
 for(const path of ['/','/en','/lex-mex/agents/index.html','/lex-mex/agents/sitemap.xml','/sitemap.xml',`/lex-mex/r/${pin.release_id}/mx/lritf/article/46`]){const r=await mf.dispatchFetch('https://example.test'+path),body=await r.text();if(r.status!==200)throw Error(path+': '+r.status);if(path.endsWith('sitemap.xml')&&(body.match(/<url>/g)||[]).length<870)throw Error('Incomplete sitemap');results.push({path,status:r.status,bytes:Buffer.byteLength(body)});}
 let rateLimitObserved=false;
 if(config.ratelimits?.length){
  for(let i=0;i<650;i++){const response=await mf.dispatchFetch('https://example.test/lex-mex/current.json');await response.arrayBuffer();if(response.status===429){if(response.headers.get('Retry-After')!=='60')throw Error('Missing retry interval');rateLimitObserved=true;break;}}
  if(!rateLimitObserved)throw Error('Configured retrieval budget did not reject excess requests');
  const home=await mf.dispatchFetch('https://example.test/');if(home.status!==200)throw Error('Retrieval budget affected authored home');await home.arrayBuffer();
 }
 const proof={rateLimitObserved,runtime:'exact built host modules and native local asset binding',tools,article46_retrieved:true,publication_id:pin.publication_id,results};await writeFile(output,JSON.stringify(proof,null,2)+'\n');console.log(JSON.stringify(proof));
}finally{await client.close();await mf.dispose();}
