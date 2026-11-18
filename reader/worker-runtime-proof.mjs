// Local only: node reader/worker-runtime-proof.mjs PROJECTION TOOL_NODE_MODULES NEW_OUTPUT_DIR
import {readFile,writeFile,mkdir} from 'node:fs/promises';
import {resolve,join} from 'node:path';import {pathToFileURL} from 'node:url';import {gzipSync} from 'node:zlib';
import {hash} from './query-contract.mjs';
const [projectionPath,toolModules,outputPath]=process.argv.slice(2);if(!projectionPath||!toolModules||!outputPath)throw Error('Expected projection, installed tooling node_modules and new output directory');
const output=resolve(outputPath);await mkdir(output);const bytes=await readFile(projectionPath),digest=hash(bytes),data=JSON.parse(bytes).data;
const {build}=await import(pathToFileURL(join(resolve(toolModules),'esbuild/lib/main.js')));
const {Miniflare}=await import(pathToFileURL(join(resolve(toolModules),'miniflare/dist/src/index.js')));
const code=`import {Buffer} from 'node:buffer';import {projectionWorker} from ${JSON.stringify(new URL('./query-worker.mjs',import.meta.url).pathname)};import projection from ${JSON.stringify(resolve(projectionPath))};const worker=projectionWorker(Buffer.from(JSON.stringify(projection)),${JSON.stringify(digest)});export default {fetch(request){return worker.fetch(request);}};`;
const bundle=join(output,'bundle.mjs');await build({stdin:{contents:code,resolveDir:process.cwd(),sourcefile:'worker-proof-entry.mjs'},outfile:bundle,bundle:true,format:'esm',platform:'neutral',mainFields:['module','main'],external:['node:crypto','node:buffer'],minify:true});
const mf=new Miniflare({modules:true,scriptPath:bundle,compatibilityDate:'2026-05-15',compatibilityFlags:['nodejs_compat']});const results=[];
try{
 for(const path of ['/lex-mex/agents/',`/lex-mex/api/r/${data.release.id}/search?q=segregados`,`/lex-mex/api/r/${data.release.id}/mx/lritf/article/46`]){
  const start=performance.now(),response=await mf.dispatchFetch('https://example.test'+path),body=await response.text();if(response.status!==200)throw Error('Runtime response '+response.status+': '+body);if(path.endsWith('/46')&&!body.includes('segregados'))throw Error('Wrong provision');results.push({path,status:response.status,bytes:Buffer.byteLength(body),wall_ms:performance.now()-start});
 }
 const bundled=await readFile(bundle);const proof={runtime:'local Miniflare/workerd',projection_sha256:digest,bundle_bytes:bundled.length,gzip_bytes:gzipSync(bundled).length,results,limits:'Local wall time, not production CPU/memory acceptance'};await writeFile(join(output,'proof.json'),JSON.stringify(proof,null,2)+'\n');console.log(JSON.stringify(proof));
}finally{await mf.dispose();}
