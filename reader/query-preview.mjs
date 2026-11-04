import {createServer} from 'node:http';
import {Readable} from 'node:stream';
import {handleRag} from './rag-http.mjs';
import {resolve} from 'node:path';
import {pathToFileURL} from 'node:url';
import {loadFixture} from './query-fixture.mjs';
import {retainedAdapter} from './query-retained.mjs';
import {handlePublicQuery} from './query-http.mjs';
export function queryServer(adapter,{rag}={}){return createServer(async(req,res)=>{
  try{
    const url=new URL(req.url,'http://127.0.0.1');
    const init={method:req.method,headers:req.headers};
    if(req.method!=='GET'&&req.method!=='HEAD'){init.body=Readable.toWeb(req);init.duplex='half';}
    const request=new Request(url,init);
    const response=url.pathname==='/lex-mex/api/answer'
      ? await handleRag(request,{...rag,adapter})
      : await handlePublicQuery(request,adapter);
    res.writeHead(response.status,Object.fromEntries(response.headers));res.end(Buffer.from(await response.arrayBuffer()));
  }catch{res.writeHead(503,{'Content-Type':'text/plain; charset=utf-8','Cache-Control':'no-store'});res.end('Consulta no disponible.');}
});}

if(process.argv[1]&&import.meta.url===pathToFileURL(resolve(process.argv[1])).href){
  const args=process.argv.slice(2);
  if(args.some((arg,i)=>arg!=='--fixture'&&arg!=='--release'&&args[i-1]!=='--release')||(args.includes('--fixture')&&args.includes('--release'))||args.filter(arg=>arg==='--release').length>1||(args.includes('--release')&&!args[args.indexOf('--release')+1]))throw Error('Usage: query-preview.mjs [--fixture | --release DIR]');
  const adapter=args.includes('--fixture')?await loadFixture(new URL('./fixtures/public-query-v1.json',import.meta.url)):await retainedAdapter(args.includes('--release')?args[args.indexOf('--release')+1]:'.work/public-release-v2');
  const server=queryServer(adapter);server.listen(4174,'127.0.0.1',()=>process.stdout.write(`${adapter.release.fixture?'Synthetic fixture':'Retained four-instrument corpus'} preview: http://127.0.0.1:4174/lex-mex/agents/\n`));
}
