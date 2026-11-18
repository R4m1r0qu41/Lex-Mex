import {handleMcp} from './mcp-http.mjs';
import {projectionAdapter} from './query-projection.mjs';
import {handlePublicQuery} from './query-http.mjs';
import {handleRag} from './rag-http.mjs';
// Host owns bindings, metering, credentials and fallback. Only immutable public data is retained.
export function projectionWorker(bytes,sha256){
  const adapter=projectionAdapter(bytes,sha256);
  return {async fetch(request,rag={}){
    const url=new URL(request.url);
    if(url.pathname==='/lex-mex/mcp')return handleMcp(request,adapter);
    if(url.pathname==='/lex-mex/api/answer')return handleRag(request,{...rag,adapter});
    return handlePublicQuery(request,adapter);
  }};
}
