import {Buffer} from 'node:buffer';
import {timingSafeEqual} from 'node:crypto';
import {retrieveEvidence} from './rag-evidence.mjs';
import {generateAnswer} from './rag-answer.mjs';
const reply=(status,value)=>Response.json(value,{status,headers:{'Cache-Control':'no-store','X-Content-Type-Options':'nosniff'}});
// Local Node integration boundary. Hosting adapter and real generator remain explicit inputs.
export async function handleRag(request,{adapter,generate,bearerToken,authorizeBudget}){
  if(new URL(request.url).pathname!=='/lex-mex/api/answer')return reply(404,{error:'not_found'});
  if(request.method!=='POST')return reply(405,{error:'method_not_allowed'});
  if(typeof bearerToken!=='string'||bearerToken.length<32||typeof generate!=='function'||typeof authorizeBudget!=='function')return reply(503,{error:'unavailable'});
  const supplied=Buffer.from(request.headers.get('authorization')||''),expected=Buffer.from('Bearer '+bearerToken);
  if(supplied.length!==expected.length||!timingSafeEqual(supplied,expected))return reply(401,{error:'unauthorized'});
  if(request.headers.get('content-type')?.split(';')[0]!=='application/json')return reply(415,{error:'json_required'});
  let input;
  try{
    if(!request.body)return reply(400,{error:'body_required'});
    const reader=request.body.getReader(),parts=[];let size=0;
    for(;;){const {done,value}=await reader.read();if(done)break;size+=value.byteLength;if(size>8192){await reader.cancel();return reply(413,{error:'request_too_large'});}parts.push(value);}
    input=JSON.parse(Buffer.concat(parts).toString('utf8'));
    if(!input||Object.keys(input).sort().join(',')!=='queries,question')return reply(400,{error:'invalid_request'});
  }catch{return reply(400,{error:'invalid_request'});}
  let bundle;try{bundle=await retrieveEvidence(adapter,input);}catch{return reply(400,{error:'retrieval_failed'});}
  // An injected limiter must approve each model call. Never enable unmetered public generation.
  try{if(!await authorizeBudget(request))return reply(429,{error:'budget_unavailable'});}catch{return reply(503,{error:'budget_unavailable'});}
  try{return reply(200,{answer:await generateAnswer(bundle,generate),evidence:bundle});}catch{return reply(502,{error:'answer_unavailable'});}
}
