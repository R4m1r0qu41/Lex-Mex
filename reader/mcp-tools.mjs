import {handlePublicQuery} from './query-http.mjs';
// Protocol-independent tool definitions; an MCP SDK transport must register these.
const text={type:'string',minLength:1,maxLength:256};
const schema=properties=>({type:'object',properties:{release_id:{type:'string',pattern:'^[a-f0-9]{64}$'},...properties},required:['release_id',...Object.keys(properties).filter(k=>k!=='cursor'&&(k!=='instrument'||!Object.hasOwn(properties,'query')))],additionalProperties:false});
const annotations={readOnlyHint:true,destructiveHint:false,idempotentHint:true,openWorldHint:false};
export const retrievalTools=[
 {name:'search',description:'Search selected legal provisions. Read full provisions before answering; no match does not prove legal absence.',inputSchema:schema({query:text,instrument:text,cursor:{type:'string',minLength:1,maxLength:1024}}),annotations},
 {name:'get_provision',description:'Get complete exact provision text with official source, release and recorded review status.',inputSchema:schema({instrument:text,type:{enum:['article','transitory','annex'],type:'string'},number:text}),annotations},
 {name:'get_references',description:'Get registered express references; unresolved targets do not establish legal absence.',inputSchema:schema({instrument:text,type:{enum:['article','transitory','annex'],type:'string'},number:text,cursor:{type:'string',minLength:1,maxLength:1024}}),annotations},
 {name:'get_coverage',description:'Get partial coverage and freshness limitations for the pinned release.',inputSchema:schema({}),annotations},
];
export async function callRetrievalTool(adapter,name,args){
 const tool=retrievalTools.find(t=>t.name===name);if(!tool)throw Error('Unknown retrieval tool');
 if(!args||typeof args!=='object'||Array.isArray(args)||Object.keys(args).some(k=>!Object.hasOwn(tool.inputSchema.properties,k))||tool.inputSchema.required.some(k=>!Object.hasOwn(args,k)))throw Error('Invalid tool arguments');
 for(const [key,value] of Object.entries(args)){const spec=tool.inputSchema.properties[key];if(typeof value!=='string'||!value.length||value.length>(spec.maxLength??256)||(spec.enum&&!spec.enum.includes(value)))throw Error('Invalid tool argument');}
 if(!/^[a-f0-9]{64}$/.test(args.release_id)||args.release_id!==adapter.release.id)throw Error('Release unavailable');
 const root=`https://lex-mex.invalid/lex-mex/api/r/${args.release_id}/`;let path;
 if(name==='search'){const q=new URLSearchParams({q:args.query});if(args.instrument)q.set('instrument',args.instrument);if(args.cursor)q.set('cursor',args.cursor);path='search?'+q;}
 else if(name==='get_coverage')path='coverage';
 else{path=`mx/${encodeURIComponent(args.instrument)}/${args.type}/${encodeURIComponent(args.number)}`;if(name==='get_references')path+='/references'+(args.cursor?'?cursor='+encodeURIComponent(args.cursor):'');}
 const response=await handlePublicQuery(new Request(root+path),adapter),data=await response.json();
 return {content:[{type:'text',text:JSON.stringify(data)}],structuredContent:data,isError:!response.ok};
}
