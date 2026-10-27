// Public transport DTOs only. Never pass database extras/raw records to this layer.
import {createHash} from 'node:crypto';
export const hash=b=>createHash('sha256').update(b).digest('hex');
export const VERSION='public-retrieval-v1';
export const MAX_RESULTS=10;
export const normalize=s=>s.toLowerCase().normalize('NFD').replace(/\p{M}/gu,'');
const fail=()=>{throw Error('Invalid public retrieval DTO');};
function object(value,keys){if(!value||typeof value!=='object'||Array.isArray(value)||Object.keys(value).length!==keys.length||keys.some(k=>!Object.hasOwn(value,k)))fail();}
function string(s,max=2048){if(typeof s!=='string'||Array.from(s).length>max)fail();}
function digest(s){if(typeof s!=='string'||!/^[a-f0-9]{64}$/.test(s))fail();}
function array(a,check,max=MAX_RESULTS){if(!Array.isArray(a)||a.length>max)fail();a.forEach(check);}
function cursor(s){if(s!==null){string(s,1024);if(!/^[A-Za-z0-9_-]+$/.test(s))fail();}}
export function checkRelease(r){object(r,['id','corpus_commit','fixture']);digest(r.id);if(!/^[a-f0-9]{40,64}$/.test(r.corpus_commit)||typeof r.fixture!=='boolean')fail();}
export function checkInstrument(i){object(i,['slug','id','title','short_name']);if(!/^[a-z0-9-]{1,80}$/.test(i.slug))fail();string(i.id,512);string(i.title,1024);string(i.short_name,80);}
// Recorded official URLs are preserved as published: http or https, never credentials or another scheme.
export function isOfficialSourceUrl(s){if(typeof s!=='string'||Array.from(s).length>2048)return false;let u;try{u=new URL(s);}catch{return false;}return (u.protocol==='https:'||u.protocol==='http:')&&!u.username&&!u.password;}
export function checkSource(s){object(s,['official_url','publisher','retrieved_at','source_sha256','recorded_reform_date']);if(!isOfficialSourceUrl(s.official_url))fail();string(s.publisher,512);string(s.retrieved_at,80);digest(s.source_sha256);if(s.recorded_reform_date!==null&&!/^\d{4}-\d{2}-\d{2}$/.test(s.recorded_reform_date))fail();}
export function checkArticle(p){object(p,['id','instrument','provision_type','number','label','text','text_sha256','source','temporal_status','review_status']);string(p.id,512);checkInstrument(p.instrument);if(!['article','transitory','annex'].includes(p.provision_type))fail();if(!/^[a-z0-9-]{1,80}$/.test(p.number))fail();string(p.label,160);string(p.text,10_000_000);digest(p.text_sha256);if(hash(Buffer.from(p.text))!==p.text_sha256)fail();checkSource(p.source);string(p.temporal_status,80);string(p.review_status,80);}
export function checkHit(h){object(h,['id','instrument','provision_type','number','label','excerpt']);string(h.id,512);checkInstrument(h.instrument);if(!['article','transitory','annex'].includes(h.provision_type)||!/^[a-z0-9-]{1,80}$/.test(h.number))fail();string(h.label,160);string(h.excerpt,240);}
export function checkEdge(e){object(e,['target_id','target','source_span','resolution_status','resolution_rule','qualifiers']);string(e.target_id,512);if(e.target!==null){object(e.target,['slug','provision_type','number']);if(!/^[a-z0-9-]{1,80}$/.test(e.target.slug)||!['article','transitory','annex'].includes(e.target.provision_type)||!/^[a-z0-9-]{1,80}$/.test(e.target.number))fail();}string(e.source_span,512);if(!['resolved','unresolved','historical_target_unavailable','historical_superseded_reference_outdated'].includes(e.resolution_status))fail();if(e.resolution_status!=='resolved'&&e.target!==null)fail();if(e.resolution_rule!==null)string(e.resolution_rule,80);array(e.qualifiers,q=>{object(q,['type','text']);if(!['fraction','paragraph','subsection'].includes(q.type))fail();string(q.text,512);},100);}
export function checkResponse(response){
  object(response,['schema_version','release','result']);if(response.schema_version!==VERSION)fail();checkRelease(response.release);
  const r=response.result;object(r,['kind','data']);const d=r.data;
  switch(r.kind){
    case 'catalog':object(d,['items','next_cursor']);array(d.items,checkInstrument);cursor(d.next_cursor);break;
    case 'search':object(d,['query','items','next_cursor']);string(d.query,256);array(d.items,checkHit);cursor(d.next_cursor);break;
    case 'article':object(d,['provision']);checkArticle(d.provision);break;
    case 'references':object(d,['items','next_cursor']);array(d.items,checkEdge);cursor(d.next_cursor);break;
    case 'coverage':object(d,['scope','notice']);if(d.scope!=='selected_partial')fail();string(d.notice,2048);break;
    case 'error':object(d,['code','message']);if(!['bad_request','not_found','unavailable','response_too_large'].includes(d.code))fail();string(d.message,512);break;
    default:fail();
  }
  return response;
}
