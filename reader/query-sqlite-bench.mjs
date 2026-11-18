// Measures the SQLite adapter on representative full-corpus workloads. Usage:
//   node reader/query-sqlite-bench.mjs <corpus.sqlite> [runs]
// It reports measurements only (time, process memory, response bytes); it makes no latency promise.
import {sqliteAdapter} from './query-sqlite.mjs';
import {handlePublicQuery} from './query-http.mjs';
const path=process.argv[2],runs=Number(process.argv[3]||15);
if(!path){console.error('usage: node reader/query-sqlite-bench.mjs <corpus.sqlite> [runs]');process.exit(2);}
const adapter=sqliteAdapter(path);const api=`/lex-mex/api/r/${adapter.release.id}/`;
const call=async p=>{const r=await handlePublicQuery(new Request('https://bench.invalid'+p),adapter);const b=await r.text();return {status:r.status,bytes:Buffer.byteLength(b)};};
const q=encodeURIComponent;
const workloads=[
  ['catalog first page',`${api}instruments?limit=10`],
  ['article lookup (lac 78 Terdecies)',`${api}mx/lac/article/78-terdecies`],
  ['search rare token: terdecies',`${api}search?q=terdecies&limit=10`],
  ['search phrase: contrato de seguro',`${api}search?q=${q('contrato de seguro')}&limit=10`],
  ['search accented: obligación (2,134 hits)',`${api}search?q=${q('obligación')}&limit=10`],
  ['search common: ley',`${api}search?q=ley&limit=10`],
  ['search stop-word: de',`${api}search?q=de&limit=10`],
  ['search filtered: contrato in lscs',`${api}search?q=contrato&instrument=lscs&limit=10`],
  ['index of ccf, first page',`${api}mx/ccf?limit=10`],
  ['references of a provision',`${api}mx/lrsic/transitory/quinto/references?limit=10`],
];
// deep page: walk to the last reachable cursor once, then time that request
let cursor=null;for(let i=0;i<99;i++){const r=await adapter.search({query:'ley',instrument:null,limit:10,cursor});cursor=r.next_cursor;if(!cursor)break;}
if(cursor)workloads.push(['search deepest reachable page: ley (offset 990)',`${api}search?q=ley&limit=10&cursor=${cursor}`]);
const out=[];
for(const [name,p] of workloads){
  const times=[];let last;const rss0=process.memoryUsage().rss;
  for(let i=0;i<runs;i++){const t=performance.now();last=await call(p);times.push(performance.now()-t);}
  const warm=times.slice(1).sort((a,b)=>a-b);const at=f=>warm[Math.min(warm.length-1,Math.floor(f*warm.length))];
  out.push({workload:name,status:last.status,response_bytes:last.bytes,cold_ms:+times[0].toFixed(1),warm_median_ms:+at(.5).toFixed(1),warm_p95_ms:+at(.95).toFixed(1),warm_max_ms:+warm[warm.length-1].toFixed(1),rss_growth_mb:+((process.memoryUsage().rss-rss0)/1048576).toFixed(1)});
}
console.log(JSON.stringify({node:process.version,release:adapter.release.id,runs,peak_rss_mb:+(process.resourceUsage().maxRSS/1024).toFixed(0),results:out},null,1));
