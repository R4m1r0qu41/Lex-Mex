import {createServer} from 'node:http';
import {createReadStream} from 'node:fs';
import {stat,realpath} from 'node:fs/promises';
import {resolve,join,sep} from 'node:path';
import {pathToFileURL} from 'node:url';
export function createReaderServer(directory,{base='/lex-mex/'}={}) {
  const root=resolve(directory);
  const mime={'.html':'text/html; charset=utf-8','.css':'text/css; charset=utf-8','.js':'text/javascript; charset=utf-8','.mjs':'text/javascript; charset=utf-8','.json':'application/json; charset=utf-8','.md':'text/markdown; charset=utf-8','.txt':'text/plain; charset=utf-8','.pdf':'application/pdf','.svg':'image/svg+xml'};
  return createServer(async(req,res)=>{
    try{
      if(!['GET','HEAD'].includes(req.method)){res.writeHead(405,{Allow:'GET, HEAD'}).end();return;}
      const pathname=decodeURIComponent(new URL(req.url,'http://localhost').pathname);
      if(pathname==='/'||pathname===base.slice(0,-1)){res.writeHead(302,{Location:base}).end();return;}
      if(!pathname.startsWith(base)){res.writeHead(404).end();return;}
      const relative=pathname.slice(base.length)||'index.html';
      const file=resolve(root,relative);if(!file.startsWith(root+sep)||relative.includes('\\')){res.writeHead(404).end();return;}
      const actual=await realpath(file);if(!actual.startsWith(await realpath(root)+sep)){res.writeHead(404).end();return;}
      const info=await stat(actual);if(!info.isFile()){res.writeHead(404).end();return;}
      const extension=actual.slice(actual.lastIndexOf('.'));const type=mime[extension];if(!type){res.writeHead(404).end();return;}
      const headers={'Content-Type':type,'X-Content-Type-Options':'nosniff','Accept-Ranges':'bytes','Cache-Control':relative.startsWith('releases/')?'public, max-age=31536000, immutable':'no-cache','Referrer-Policy':'no-referrer'};
      const range=req.headers.range;let start=0,end=info.size-1,status=200;
      if(range){
        const match=/^bytes=(\d*)-(\d*)$/.exec(range);
        if(!match||(!match[1]&&!match[2])){res.writeHead(416,{'Content-Range':`bytes */${info.size}`}).end();return;}
        if(match[1]){start=Number(match[1]);if(match[2])end=Number(match[2]);}
        else {start=Math.max(0,info.size-Number(match[2]));}
        if(!Number.isSafeInteger(start)||!Number.isSafeInteger(end)||start<0||end<start||start>=info.size){res.writeHead(416,{'Content-Range':`bytes */${info.size}`}).end();return;}
        end=Math.min(end,info.size-1);status=206;headers['Content-Range']=`bytes ${start}-${end}/${info.size}`;
      }
      headers['Content-Length']=info.size===0?0:end-start+1;res.writeHead(status,headers);
      if(req.method==='HEAD'||info.size===0){res.end();return;}
      const stream=createReadStream(actual,{start,end});stream.on('error',()=>res.destroy());stream.pipe(res);
    }catch(error){res.writeHead(['ENOENT','ENOTDIR'].includes(error.code)?404:400).end();}
  });
}
if(process.argv[1]&&import.meta.url===pathToFileURL(resolve(process.argv[1])).href){
  const args=process.argv.slice(2),i=args.indexOf('--dir'),p=args.indexOf('--port');
  if(i<0||!args[i+1]){console.error('Usage: node reader/serve.mjs --dir PUBLICATION [--port 4173]');process.exitCode=1;}
  else{const port=p<0?4173:Number(args[p+1]);const server=createReaderServer(args[i+1]);server.listen(port,'127.0.0.1',()=>console.log(`Lex-Mex preview: http://127.0.0.1:${port}/lex-mex/`));}
}
