import {Server} from '@modelcontextprotocol/sdk/server/index.js';
import {WebStandardStreamableHTTPServerTransport} from '@modelcontextprotocol/sdk/server/webStandardStreamableHttp.js';
import {ListToolsRequestSchema,CallToolRequestSchema} from '@modelcontextprotocol/sdk/types.js';
import {retrievalTools,callRetrievalTool} from './mcp-tools.mjs';
export async function handleMcp(request,adapter,{allowedOrigins=[]}={}){
 const url=new URL(request.url),origin=request.headers.get('origin');
 if(url.pathname!=='/lex-mex/mcp')return new Response('Not found',{status:404});
 if(origin&&origin!==url.origin&&!allowedOrigins.includes(origin))return new Response('Forbidden origin',{status:403});
 if(request.method==='POST'){
  if(!request.body)return new Response('Body required',{status:400});
  const reader=request.body.getReader(),parts=[];let size=0;
  try{for(;;){const {done,value}=await reader.read();if(done)break;size+=value.byteLength;if(size>8192){await reader.cancel();return new Response('Request too large',{status:413});}parts.push(value);}}catch{return new Response('Invalid body',{status:400});}
  const body=new Uint8Array(size);let offset=0;for(const part of parts){body.set(part,offset);offset+=part.byteLength;}
  request=new Request(request,{body});
 }
 // Public read-only evidence; host applies abuse limits. No model credentials or writes.
 const server=new Server({name:'lex-mex',version:'0.1.0'},{capabilities:{tools:{}},instructions:`Selected partial legal corpus. Pin release ${adapter.release.id}. Search then retrieve complete provisions; cite sources and disclose freshness limits.`});
 server.setRequestHandler(ListToolsRequestSchema,async()=>({tools:retrievalTools}));
 server.setRequestHandler(CallToolRequestSchema,async({params})=>callRetrievalTool(adapter,params.name,params.arguments));
 const transport=new WebStandardStreamableHTTPServerTransport({sessionIdGenerator:undefined,enableJsonResponse:true});
 await server.connect(transport);
 try{const response=await transport.handleRequest(request);const bytes=await response.arrayBuffer();return new Response(bytes.byteLength?bytes:null,{status:response.status,headers:{...Object.fromEntries(response.headers),'Cache-Control':'no-store','X-Content-Type-Options':'nosniff'}});}finally{await server.close();}
}
