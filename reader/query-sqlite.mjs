// Read-only public retrieval adapter over an accepted SQLite corpus projection (node:sqlite).
// It returns PUBLIC DTOs only, built from named columns; extras and review payloads never leave this module.
//
// Semantics (explicit, and different from the in-memory fixture adapter):
//  - search: the query is normalized (lower case, accents removed), split into letter/digit tokens and
//    matched as ONE adjacent-token phrase against label, heading and text through the FTS5 index
//    (unicode61, diacritics removed). Punctuation and case are ignored; partial words do not match.
//  - ranking: article, then transitory, then annex; within a kind by FTS5 bm25, then provision id.
//  - pagination: opaque cursor bound to release, route and query; results are reachable to MAX_DEPTH
//    only. A deeper cursor is a client error; refine the query or filter by instrument.
//  - missing or held provisions are never invented: lookup returns null and the transport answers 404.
import {DatabaseSync} from 'node:sqlite';
import {hash,normalize,MAX_RESULTS,checkRelease} from './query-contract.mjs';

export const MAX_DEPTH=1000;
export const MAX_TOKENS=12;
const TYPE=/:(article|transitory|annex):([a-z0-9-]{1,80})$/;
const LOOSE_TYPE=/:(article|transitory|annex):[^:]+$/;
const bad=m=>{throw Error(m);};
// A provision is addressable only when its id ends in a URL-safe number (the public route is [a-z0-9-]{1,80}).
// Ordinal signs such as `transitory:1º` are not; those provisions are counted in the coverage notice, never guessed.
const SERVABLE="(length(p.id)>length(p.instrument_id)+length(p.provision_type)+2 AND length(p.id)-length(p.instrument_id)-length(p.provision_type)-2<=80 AND substr(p.id,length(p.instrument_id)+length(p.provision_type)+3) NOT GLOB '*[^a-z0-9-]*')";
const tokens=query=>{const t=normalize(query).split(/[^\p{L}\p{N}]+/u).filter(Boolean);if(!t.length||t.length>MAX_TOKENS)bad('Invalid search');return t;};

// Excerpt start in code points, found on the accent-folded text so accented spellings match.
function excerpt(text,first){
  const cps=Array.from(text);let norm='';const at=[];
  for(let i=0;i<cps.length;i++){const n=normalize(cps[i]);for(let k=0;k<n.length;k++)at.push(i);norm+=n;}
  const idx=first?norm.indexOf(first):-1;const start=idx<0?0:Math.max(0,at[idx]-50);
  return cps.slice(start,start+160).join('');
}

export function sqliteAdapter(path,{notice,maxDepth=MAX_DEPTH}={}){
  if(!Number.isSafeInteger(maxDepth)||maxDepth<1||maxDepth>MAX_DEPTH)bad('Invalid depth');
  const db=new DatabaseSync(path,{readOnly:true});db.exec('PRAGMA query_only=ON');
  const spec=db.prepare('SELECT id,specification FROM corpus_release').all();
  if(spec.length!==1)bad('Unexpected release table');
  const release={id:spec[0].id,corpus_commit:JSON.parse(spec[0].specification).corpus_commit,fixture:false};checkRelease(release);
  const q={
    unserved:db.prepare(`SELECT count(*) AS n FROM provisions p WHERE NOT ${SERVABLE}`),
    count:db.prepare('SELECT count(*) AS n FROM instruments'),
    catalog:db.prepare("SELECT id,slug,json_extract(extras,'$.official_title') AS title,json_extract(extras,'$.short_name') AS short_name FROM instruments ORDER BY slug LIMIT ? OFFSET ?"),
    bySlug:db.prepare("SELECT id,slug,json_extract(extras,'$.official_title') AS title,json_extract(extras,'$.short_name') AS short_name FROM instruments WHERE slug=?"),
    byId:db.prepare("SELECT slug FROM instruments WHERE id=?"),
    provision:db.prepare("SELECT p.id,p.instrument_id,p.text,p.label,p.provision_type,json_extract(p.extras,'$.temporal_status') AS temporal_status,json_extract(p.extras,'$.review_status') AS review_status FROM provisions p WHERE p.id=?"),
    index:db.prepare(`SELECT p.id,p.instrument_id,p.text,p.label,p.provision_type FROM provisions p WHERE p.instrument_id=? AND ${SERVABLE} ORDER BY p.ordinal LIMIT ? OFFSET ?`),
    separateAnnexSources:db.prepare("SELECT count(*) AS n FROM source_manifests WHERE instrument_id=? AND role='annex-source-manifests.json'"),
    source:db.prepare("SELECT json_extract(i.extras,'$.source_url') AS official_url,json_extract(i.extras,'$.source_sha256') AS source_sha256,json_extract(i.extras,'$.retrieved_at') AS retrieved_at,json_extract(i.extras,'$.latest_reform_date') AS recorded_reform_date,(SELECT json_extract(m.extras,'$.publisher') FROM source_manifests m WHERE m.instrument_id=i.id AND m.role='source-manifest.json') AS publisher FROM instruments i WHERE i.id=?"),
    edges:db.prepare("SELECT id,target_provision_id,target_instrument_id,resolution_status,json_extract(extras,'$.source_span') AS source_span,json_extract(extras,'$.resolution_rule') AS resolution_rule FROM reference_edges WHERE source_provision_id=? ORDER BY ordinal LIMIT ? OFFSET ?"),
    qualifiers:db.prepare("SELECT json_extract(payload,'$.qualifier_type') AS type,json_extract(payload,'$.text') AS text FROM reference_qualifiers WHERE edge_id=? ORDER BY ordinal"),
    search:db.prepare(`SELECT p.id,p.instrument_id,p.text,p.label,p.provision_type FROM provisions_fts f JOIN provisions p ON p.row_id=f.rowid WHERE provisions_fts MATCH ? AND ${SERVABLE} ORDER BY CASE p.provision_type WHEN 'article' THEN 0 WHEN 'transitory' THEN 1 ELSE 2 END,bm25(provisions_fts),p.id LIMIT ? OFFSET ?`),
    searchIn:db.prepare(`SELECT p.id,p.instrument_id,p.text,p.label,p.provision_type FROM provisions_fts f JOIN provisions p ON p.row_id=f.rowid WHERE provisions_fts MATCH ? AND p.instrument_id=? AND ${SERVABLE} ORDER BY CASE p.provision_type WHEN 'article' THEN 0 WHEN 'transitory' THEN 1 ELSE 2 END,bm25(provisions_fts),p.id LIMIT ? OFFSET ?`),
  };
  const instruments=new Map();
  const instrument=row=>({slug:row.slug,id:row.id,title:String(row.title??''),short_name:String(row.short_name??'')});
  const instrumentById=id=>{if(!instruments.has(id)){const slug=q.byId.get(id)?.slug;if(!slug)bad('Unknown instrument');instruments.set(id,instrument(q.bySlug.get(slug)));}return instruments.get(id);};
  const numberOf=id=>{const m=TYPE.exec(id);if(!m)bad('Unrecognized provision id');return m[2];};
  const hit=(row,first='')=>({id:row.id,instrument:instrumentById(row.instrument_id),provision_type:row.provision_type,number:numberOf(row.id),label:row.label,excerpt:excerpt(row.text,first)});
  function page(binding,limit,token){
    const scope=hash(JSON.stringify([release.id,...binding]));let offset=0;
    if(!Number.isSafeInteger(limit)||limit<1||limit>MAX_RESULTS)bad('Invalid limit');
    if(token){let d;try{d=JSON.parse(Buffer.from(token,'base64url').toString());}catch{bad('Invalid cursor');}
      if(!d||typeof d!=='object'||Array.isArray(d)||d.scope!==scope||!Number.isSafeInteger(d.offset)||d.offset<0||d.offset>=maxDepth)bad('Invalid cursor');offset=d.offset;}
    // The last page is cut to the remaining depth, so a walk of any page size ends at maxDepth results at most.
    const take=Math.min(limit,maxDepth-offset);
    return {offset,take,finish(rows){const more=rows.length>take&&offset+take<maxDepth;
      return {rows:rows.slice(0,take),next_cursor:more?Buffer.from(JSON.stringify({scope,offset:offset+take})).toString('base64url'):null};}};
  }
  return {
    release,notice,
    close(){db.close();},
    async catalog({limit,cursor}){const p=page(['catalog'],limit,cursor);const r=p.finish(q.catalog.all(p.take+1,p.offset));return {items:r.rows.map(instrument),next_cursor:r.next_cursor};},
    async search({query,instrument:slug,limit,cursor}){
      const t=tokens(query),match='"'+t.join(' ')+'"';
      const p=page(['search',query,slug??null],limit,cursor);
      let inst=null;if(slug){inst=q.bySlug.get(slug);if(!inst)return {items:[],next_cursor:null,query};}
      const rows=inst?q.searchIn.all(match,inst.id,p.take+1,p.offset):q.search.all(match,p.take+1,p.offset);
      const r=p.finish(rows);return {items:r.rows.map(row=>hit(row,t[0])),next_cursor:r.next_cursor,query};
    },
    async index({slug,limit,cursor}){
      const inst=q.bySlug.get(slug);const p=page(['index',slug],limit,cursor);
      const r=p.finish(inst?q.index.all(inst.id,p.take+1,p.offset):[]);return {items:r.rows.map(row=>hit(row)),next_cursor:r.next_cursor,query:`Índice de ${slug}`};
    },
    async lookup({slug,type,number}){
      const inst=q.bySlug.get(slug);if(!inst)return null;
      const row=q.provision.get(`${inst.id}:${type}:${number}`);if(!row)return null;
      // The accepted schema does not bind an annex provision to a specific annex manifest.
      // Never guess from ordinal/URL or silently cite the parent PDF.
      if(type==='annex'&&q.separateAnnexSources.get(inst.id).n)bad('Separate annex source mapping unavailable');
      const s=q.source.get(inst.id);
      return {id:row.id,instrument:instrument(inst),provision_type:row.provision_type,number,label:row.label,text:row.text,
        text_sha256:hash(Buffer.from(row.text)),
        source:{official_url:s.official_url,publisher:String(s.publisher??''),retrieved_at:String(s.retrieved_at??''),source_sha256:s.source_sha256,recorded_reform_date:s.recorded_reform_date??null},
        temporal_status:String(row.temporal_status??'unknown'),review_status:String(row.review_status??'not_analyzed')};
    },
    async references({id,limit,cursor}){
      const p=page(['references',id],limit,cursor);const r=p.finish(q.edges.all(id,p.take+1,p.offset));
      return {items:r.rows.map(e=>{
        const resolved=e.resolution_status==='resolved';let target=null;
        // A resolved target keeps its exact canonical id and status. If its number has no public route (an ordinal sign
        // such as `9º`), `target` is null: the edge is neither dropped nor given an invented normalized id.
        if(resolved){const m=TYPE.exec(e.target_provision_id),slugRow=q.byId.get(e.target_instrument_id);
          if(!slugRow||!LOOSE_TYPE.test(e.target_provision_id))bad('Unresolvable edge target');
          if(m)target={slug:slugRow.slug,provision_type:m[1],number:m[2]};}
        return {target_id:e.target_provision_id,target,source_span:String(e.source_span??''),resolution_status:e.resolution_status,resolution_rule:e.resolution_rule??null,qualifiers:q.qualifiers.all(e.id).map(x=>({type:x.type,text:x.text}))};
      }),next_cursor:r.next_cursor};
    },
    async coverage(){return {scope:'selected_partial',notice:notice??`Colección parcial de ${q.count.get().n} instrumentos de la proyección ${release.id}. ${q.unserved.get().n} disposiciones con identificadores no aptos para URL (por ejemplo signos ordinales) no se sirven por este contrato. Una ausencia no acredita inexistencia jurídica, y un resultado no acredita vigencia ni aplicabilidad.`};},
  };
}
