// Builds a small corpus projection with the real table shapes the SQLite adapter reads, for tests that must
// not depend on a 480 MB database. Synthetic data only.
import {DatabaseSync} from 'node:sqlite';
import {mkdtempSync} from 'node:fs';
import {tmpdir} from 'node:os';
import {join} from 'node:path';

export const RELEASE='a'.repeat(64);
const COMMIT='b'.repeat(40);
const SHA='c'.repeat(64);
const ID='urn:lex-mx:federal:statute:';

export function miniProjection({matches=40}={}){
  const path=join(mkdtempSync(join(tmpdir(),'lexmex-mini-')),'corpus.sqlite');
  const db=new DatabaseSync(path);
  db.exec(`
   CREATE TABLE corpus_release(id TEXT PRIMARY KEY, specification TEXT NOT NULL);
   CREATE TABLE instruments(id TEXT PRIMARY KEY, slug TEXT GENERATED ALWAYS AS (CASE WHEN input_path LIKE 'corpus/mx/%/instrument.json' THEN substr(input_path,11,instr(substr(input_path,11),'/')-1) ELSE id END) VIRTUAL UNIQUE, ordinal INTEGER NOT NULL, input_path TEXT NOT NULL, source_pointer TEXT NOT NULL, extras TEXT NOT NULL);
   CREATE TABLE source_manifests(input_path TEXT NOT NULL, source_pointer TEXT NOT NULL, instrument_id TEXT NOT NULL, role TEXT NOT NULL, ordinal INTEGER NOT NULL, extras TEXT NOT NULL);
   CREATE TABLE provisions(row_id INTEGER PRIMARY KEY, id TEXT NOT NULL UNIQUE, instrument_id TEXT NOT NULL, ordinal INTEGER NOT NULL, text TEXT NOT NULL, heading_title TEXT NOT NULL DEFAULT '', input_path TEXT NOT NULL, source_pointer TEXT NOT NULL, extras TEXT NOT NULL, label TEXT GENERATED ALWAYS AS(json_extract(extras,'$.label')) VIRTUAL, number TEXT GENERATED ALWAYS AS(json_extract(extras,'$.number')) VIRTUAL, provision_type TEXT GENERATED ALWAYS AS(json_extract(extras,'$.provision_type')) VIRTUAL);
   CREATE VIRTUAL TABLE provisions_fts USING fts5(label,heading_title,text,content='provisions',content_rowid='row_id',tokenize='unicode61 remove_diacritics 2');
   CREATE TABLE reference_edges(id TEXT PRIMARY KEY, source_provision_id TEXT NOT NULL, target_instrument_id TEXT NOT NULL, target_provision_id TEXT NOT NULL, resolution_status TEXT NOT NULL, ordinal INTEGER NOT NULL, input_path TEXT NOT NULL, source_pointer TEXT NOT NULL, extras TEXT NOT NULL);
   CREATE TABLE reference_qualifiers(edge_id TEXT NOT NULL, ordinal INTEGER NOT NULL, payload TEXT NOT NULL, PRIMARY KEY(edge_id,ordinal));
  `);
  db.prepare('INSERT INTO corpus_release VALUES(?,?)').run(RELEASE,JSON.stringify({corpus_commit:COMMIT}));
  let ord=0;
  for(const slug of ['alfa','beta']){
    db.prepare('INSERT INTO instruments(id,ordinal,input_path,source_pointer,extras) VALUES(?,?,?,?,?)').run(ID+slug,ord++,`corpus/mx/${slug}/instrument.json`,'/',JSON.stringify({official_title:`Ley ${slug}`,short_name:slug.toUpperCase(),source_url:`https://example.org/${slug}.pdf`,source_sha256:SHA,retrieved_at:'2026-01-01',latest_reform_date:null}));
    db.prepare('INSERT INTO source_manifests VALUES(?,?,?,?,?,?)').run(`corpus/mx/${slug}/source-manifest.json`,'/',ID+slug,'source-manifest.json',0,JSON.stringify({publisher:'Publicador de prueba'}));
  }
  let row=1,pord=0;
  const add=(slug,type,number,label,text)=>{
    const id=`${ID}${slug}:${type}:${number}`;
    db.prepare('INSERT INTO provisions(row_id,id,instrument_id,ordinal,text,heading_title,input_path,source_pointer,extras) VALUES(?,?,?,?,?,?,?,?,?)').run(row,id,ID+slug,pord++,text,'',`corpus/mx/${slug}/provisions.json`,`/${row}`,JSON.stringify({label,number,provision_type:type,temporal_status:'effective',review_status:'machine_accepted'}));
    db.prepare('INSERT INTO provisions_fts(rowid,label,heading_title,text) VALUES(?,?,?,?)').run(row,label,'',text);
    row++;return id;
  };
  for(let i=1;i<=matches;i++)add('alfa','article',String(i),`Artículo ${i}`,`norma comun numero ${i}`);
  add('alfa','transitory','9º','9º','transitorio con signo ordinal');   // id not URL-safe: no public route
  add('beta','article','1','Artículo 1','otra norma comun');
  const source=`${ID}alfa:article:1`;
  const edge=(n,target,status,instrument,span)=>{
    const id=`edge-${n}`;
    db.prepare('INSERT INTO reference_edges VALUES(?,?,?,?,?,?,?,?,?)').run(id,source,instrument,target,status,n,'corpus/mx/alfa/references.json',`/${n}`,JSON.stringify({source_span:span,resolution_rule:null}));return id;
  };
  const e0=edge(0,`${ID}beta:article:1`,'resolved',ID+'beta','artículo 1 de la Ley beta');
  edge(1,`${ID}alfa:transitory:9º`,'resolved',ID+'alfa','transitorio 9º');
  edge(2,`${ID}gone:article:3`,'historical_target_unavailable',ID+'gone','artículo 3 de la ley abrogada');
  db.prepare('INSERT INTO reference_qualifiers VALUES(?,?,?)').run(e0,0,JSON.stringify({qualifier_type:'fraction',text:'fracción II',start_char:0,end_char:3}));
  db.close();
  return path;
}
