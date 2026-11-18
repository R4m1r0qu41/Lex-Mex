import {Buffer} from 'node:buffer';
import {VERSION,MAX_RESULTS,normalize,checkRelease,checkResponse} from './query-contract.mjs';
const escape=s=>String(s).replace(/[&<>"'\r]/g,c=>({'&':'&amp;','<':'&lt;','>':'&gt;','"':'&quot;',"'":'&#39;','\r':'&#13;'}[c]));
const link=(href,label)=>`<a href="${escape(href)}">${escape(label)}</a>`;
const warning='Colección parcial. Una coincidencia no establece aplicabilidad ni vigencia actual o histórica. No es asesoría jurídica.';
const html=(title,body,fixture,notice='')=>`<!doctype html><html lang="es"><head><meta charset="utf-8"><meta name="viewport" content="width=device-width"><title>${escape(title)} — Lex-Mex</title></head><body>${fixture?'<p><strong>DEMOSTRACIÓN SINTÉTICA. No contiene textos legales reales.</strong></p>':notice?`<p><strong>${escape(notice)}</strong></p>`:''}<main><h1>${escape(title)}</h1>${body}</main><footer><p>${warning}</p></footer></body></html>\n`;
const articleUrl=(root,p)=>root+`mx/${p.instrument?.slug||p.slug}/${p.provision_type}/${p.number}`;
const headers={'X-Content-Type-Options':'nosniff','Content-Security-Policy':"default-src 'none'; base-uri 'none'; frame-ancestors 'none'; form-action 'self'",'Referrer-Policy':'no-referrer','Cache-Control':'no-store'};
function send(body,type,status=200,head=false){const bytes=Buffer.from(body);return new Response(head?null:bytes,{status,headers:{...headers,'Content-Type':type,'Content-Length':String(bytes.length)}});}
function params(url,allowed){for(const key of url.searchParams.keys())if(!allowed.includes(key)||url.searchParams.getAll(key).length!==1)throw Error('Invalid query parameters');}
function pageArgs(url){const n=url.searchParams.get('limit')||'5';if(!/^[1-9][0-9]*$/.test(n)||Number(n)>MAX_RESULTS)throw Error('Invalid limit');const cursor=url.searchParams.get('cursor');if(cursor&&(!/^[A-Za-z0-9_-]+$/.test(cursor)||cursor.length>1024))throw Error('Invalid cursor');return {limit:Number(n),cursor};}
function next(url,token){if(!token)return '';const u=new URL(url);u.searchParams.set('cursor',token);return `<p>${link(u.pathname+u.search,'Siguiente página')}</p>`;}
function searchForm(root,query='',instrument=null){return `<form method="get" action="${root}search"><label>Buscar <input name="q" value="${escape(query)}" required maxlength="256"></label>${instrument?`<input type="hidden" name="instrument" value="${escape(instrument)}">`:''}<button>Buscar</button></form>`;}
function render(response,url,root,notice){
  const {kind,data}=response.result;
  const home=`<nav>${link('/lex-mex/agents/','Acceso para agentes')} · ${link(root+'instruments','Instrumentos')} · ${link(root+'coverage','Cobertura')}</nav>`;
  let body,title;
  switch(kind){
    case 'catalog':title='Instrumentos';body=`<ul>${data.items.map(i=>`<li>${link(root+'mx/'+i.slug,i.short_name+' — '+i.title)}</li>`).join('')}</ul>${next(url,data.next_cursor)}`;break;
    case 'search':title='Resultados: '+data.query;body=searchForm(root,url.searchParams.get('q')||'',url.searchParams.get('instrument'))+`<p>Coincidencias de navegación; lea las disposiciones completas. Cero resultados no acredita ausencia normativa.</p>${data.items.length?`<ol>${data.items.map(p=>`<li>${link(articleUrl(root,p),p.instrument.short_name+' · '+p.label)}<blockquote>${escape(p.excerpt)}</blockquote></li>`).join('')}</ol>`:`<p role="status">No se encontraron coincidencias en esta selección.</p><p>${response.release.fixture?'Esta demostración solo contiene textos sintéticos. Pruebe «cuentas» u «obligación».':'Pruebe una palabra o frase más breve, cambie el filtro de instrumento o explore el catálogo.'}</p>`}${next(url,data.next_cursor)}`;break;
    case 'article':{
      title=data.provision.instrument.short_name+' · '+data.provision.label;const p=data.provision,s=p.source;
      body=`<h2>${escape(p.instrument.title)}</h2><p>Cita estable: <code>${escape(p.id)}</code>. Publicación: <code>${response.release.id}</code>.</p><p>Estado registrado: ${escape(p.temporal_status)}; revisión: ${escape(p.review_status)}. No acredita vigencia.</p><p>Fuente: ${link(s.official_url,s.publisher)}. Recuperada: ${escape(s.retrieved_at)}. Reforma registrada en esa fuente: ${escape(s.recorded_reform_date??'no disponible')}; no es una comprobación de actualidad.</p><p>SHA-256 fuente: <code>${s.source_sha256}</code>. SHA-256 texto: <code>${p.text_sha256}</code>.</p><h2>Texto completo</h2><pre><span>${escape(p.text)}</span></pre><p>${link(articleUrl(root,p)+'/references','Referencias expresas registradas')}</p>`;break;
    }
    case 'references':title='Referencias expresas registradas';body=`<ul>${data.items.map(e=>`<li>${escape(e.source_span)}: ${e.target?link(articleUrl(root,e.target),e.target_id):escape(e.target_id)} — ${escape(e.resolution_status)}; regla: ${escape(e.resolution_rule??'no registrada')}. ${e.qualifiers.map(q=>`${escape(q.type)}: ${escape(q.text)}`).join('; ')}. No equivale a un juicio de aplicabilidad.</li>`).join('')}</ul>${next(url,data.next_cursor)}`;break;
    case 'coverage':title='Cobertura';body=`<p>${escape(data.scope)}</p><p>${escape(data.notice)}</p>`;break;
    case 'error':title='Consulta no completada';body=`<p>${escape(data.code)}: ${escape(data.message)}</p>`;break;
  }
  return html(title,home+body,response.release.fixture,notice);
}

// Adapter methods return PUBLIC DTOs. Invalid/extra fields fail closed before output.
export async function handlePublicQuery(request,adapter){
  const url=new URL(request.url),head=request.method==='HEAD';
  if(!['GET','HEAD'].includes(request.method))return new Response(null,{status:405,headers:{...headers,Allow:'GET, HEAD'}});
  const release=structuredClone(adapter.release);checkRelease(release);
  const root=`/lex-mex/r/${release.id}/`,apiRoot=`/lex-mex/api/r/${release.id}/`;
  const api=url.pathname.startsWith('/lex-mex/api/');
  const envelope=(kind,data)=>checkResponse({schema_version:VERSION,release,result:{kind,data}});
  const output=(kind,data,status=200)=>{
    const response=envelope(kind,data),body=api?JSON.stringify(response):render(response,url,root,adapter.notice);
    if(Buffer.byteLength(body)>(kind==='article'?2*1024*1024:16384))throw Error('Response budget exceeded');
    return send(body,api?'application/json':'text/html; charset=utf-8',status,head);
  };
  try{
    if(url.pathname==='/lex-mex/current.json'){params(url,[]);return send(JSON.stringify({schema_version:VERSION,release,discovery_url:'/lex-mex/agents/',catalog_url:root+'instruments',coverage_url:root+'coverage'}),'application/json',200,head);}
    if(url.pathname==='/lex-mex/llms.txt'){params(url,[]);return send(`# Lex-Mex\n${release.fixture?'SYNTHETIC FIXTURE ONLY.\n':''}Read ${new URL('/lex-mex/agents/',url).href}. Pin public release ${release.id}. Follow HTML search and complete articles. Do not download the corpus or integrity manifests for ordinary queries.\n`,'text/plain; charset=utf-8',200,head);}
    if(url.pathname==='/lex-mex/'||url.pathname==='/lex-mex/agents/'){
      params(url,[]);
      const body=html('Lex-Mex — búsqueda y lectura',`<p>Publicación: <code>${release.id}</code>.</p><p>Busque una palabra o frase española mediante ${link(root+'search?q=cuentas','un ejemplo de consulta')}; cambie <code>q</code> en la URL. Filtro opcional: <code>instrument</code>. Lea el texto completo de los resultados y sus referencias; no interprete fragmentos como normas completas.</p>${searchForm(root)}<p>${link(root+'instruments','Explorar instrumentos')} · ${link(root+'coverage','Cobertura y límites')} · ${link('/lex-mex/current.json','Descriptor pequeño')}</p><p>Artículo conocido: <code>${root}mx/SLUG/article/NUMERO</code> (sufijos separados por guion); transitorios y anexos usan <code>transitory</code> y <code>annex</code>. Confirme identidad y fuente. No cargue manifiestos de integridad ni instrumentos JSON completos para consultas individuales.</p>`,release.fixture,adapter.notice);
      return send(body,'text/html; charset=utf-8',200,head);
    }
    const prefix=api?apiRoot:root;
    if(!url.pathname.startsWith(prefix))return output('error',{code:'not_found',message:'Publicación o ruta no disponible; no se sustituyó por otra versión.'},404);
    const route=url.pathname.slice(prefix.length);
    if(route==='instruments'){params(url,['limit','cursor']);return output('catalog',await adapter.catalog(pageArgs(url)));}
    if(route==='search'){
      params(url,['q','instrument','limit','cursor']);const query=url.searchParams.get('q')?.trim(),instrument=url.searchParams.get('instrument');
      if(!query||!normalize(query).trim()||Array.from(query).length>256||(instrument!==null&&!/^[a-z0-9-]{1,80}$/.test(instrument)))throw Error('Invalid search');
      return output('search',await adapter.search({query,instrument,...pageArgs(url)}));
    }
    if(route==='coverage'){params(url,[]);return output('coverage',await adapter.coverage());}
    const index=/^mx\/([a-z0-9-]{1,80})$/.exec(route);
    if(index){params(url,['limit','cursor']);return output('search',await adapter.index({slug:index[1],...pageArgs(url)}));}
    const match=/^mx\/([a-z0-9-]{1,80})\/(article|transitory|annex)\/([a-z0-9-]{1,80})(\/references)?$/.exec(route);
    if(!match)return output('error',{code:'not_found',message:'Ruta no disponible en esta publicación.'},404);
    params(url,match[4]?['limit','cursor']:[]);
    const p=await adapter.lookup({slug:match[1],type:match[2],number:match[3]});
    if(!p)return output('error',{code:'not_found',message:'No encontrado en esta publicación; cobertura no establecida. No demuestra inexistencia jurídica.'},404);
    envelope('article',{provision:p});
    if(p.instrument.slug!==match[1]||p.provision_type!==match[2]||p.number!==match[3])throw Error('Adapter identity mismatch');
    return match[4]?output('references',await adapter.references({id:p.id,...pageArgs(url)})):output('article',{provision:p});
  }catch(error){
    const budget=error.message==='Response budget exceeded';
    const bad=/^Invalid (query parameters|limit|cursor|search)$/.test(error.message);
    // Never reflect adapter exceptions, SQL or private fields into the response.
    return output('error',{code:budget?'response_too_large':bad?'bad_request':'unavailable',message:budget?'Respuesta demasiado grande. No se truncó el texto; solicite un límite menor cuando corresponda.':bad?'Consulta inválida. Revise parámetros y paginación.':'No se pudo completar la consulta; no es evidencia de ausencia normativa.'},budget?413:bad?400:503);
  }
}
