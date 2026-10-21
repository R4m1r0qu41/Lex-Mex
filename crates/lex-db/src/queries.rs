//! Bounded parameterized read-only query API; no service or presentation layer.
use anyhow::{Context, Result, ensure};
use rusqlite::{Connection, OpenFlags, OptionalExtension, params};
use serde::Serialize;
use serde_json::{Value, json};
use std::path::Path;

pub struct Reader {
    connection: Connection,
}
#[derive(Debug, Serialize)]
#[serde(tag = "outcome", rename_all = "snake_case")]
pub enum Lookup {
    Found { provision: Value },
    Qualified { qualifications: Vec<Value> },
    NotEstablished { message: String },
}
#[derive(Debug, Serialize)]
pub struct SearchHit {
    pub id: String,
    pub score: f64,
}
impl Reader {
    pub fn open(path: &Path) -> Result<Self> {
        let connection = Connection::open_with_flags(path, OpenFlags::SQLITE_OPEN_READ_ONLY)?;
        connection.execute_batch("PRAGMA query_only=ON; PRAGMA foreign_keys=ON;")?;
        Ok(Self { connection })
    }
    /// Exact canonical ID lookup; qualifications never assert legal nonexistence.
    pub fn lookup(&self, id: &str) -> Result<Lookup> {
        let raw: Option<(String, String, String, String)> = self
            .connection
            .query_row(
                "SELECT extras,id,instrument_id,text FROM provisions WHERE id=?1",
                [id],
                |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?)),
            )
            .optional()?;
        if let Some((raw, id, owner, text)) = raw {
            let mut provision: Value = serde_json::from_str(&raw)?;
            provision["id"] = json!(id);
            provision["instrument_id"] = json!(owner);
            provision["text"] = json!(text);
            return Ok(Lookup::Found { provision });
        }
        let mut qualifications = Vec::new();
        if let Some((owner, label)) = id.rsplit_once(":article:") {
            let mut stmt=self.connection.prepare("SELECT printed_label,extras,input_path,source_pointer FROM known_structural_debt WHERE instrument_id=?1 ORDER BY ordinal")?;
            let mut cursor = stmt.query([owner])?;
            while let Some(row) = cursor.next()? {
                let printed: String = row.get(0)?;
                if lex_parse::labels::canonical_slug(&printed) == label {
                    qualifications.push(json!({"kind":"not_separately_parsed","printed_label":printed,"evidence":serde_json::from_str::<Value>(&row.get::<_,String>(1)?)?,"input_path":row.get::<_,String>(2)?,"source_pointer":row.get::<_,String>(3)?}));
                }
            }
        }
        let mut stmt=self.connection.prepare("SELECT source_instrument_id,multiplicity,reason,input_path,source_pointer FROM cited_but_missing WHERE target_provision_id=?1 ORDER BY source_instrument_id")?;
        let mut cursor = stmt.query([id])?;
        while let Some(row) = cursor.next()? {
            qualifications.push(json!({"kind":"cited_not_in_corpus","source_instrument_id":row.get::<_,String>(0)?,"multiplicity":row.get::<_,i64>(1)?,"reason":row.get::<_,String>(2)?,"input_path":row.get::<_,String>(3)?,"source_pointer":row.get::<_,String>(4)?}));
        }
        let mut stmt = self.connection.prepare(
            "SELECT id,metadata_path,exclusion_reason FROM standards_catalog ORDER BY id",
        )?;
        let mut cursor = stmt.query([])?;
        while let Some(row) = cursor.next()? {
            let standard: String = row.get(0)?;
            if id == standard || id.starts_with(&format!("{standard}:")) {
                qualifications.push(json!({"kind":"excluded","instrument_id":standard,"input_path":row.get::<_,String>(1)?,"reason":row.get::<_,String>(2)?}));
            }
        }
        if qualifications.is_empty() {
            Ok(Lookup::NotEstablished {
                message: "not found in this release; coverage not established".into(),
            })
        } else {
            Ok(Lookup::Qualified { qualifications })
        }
    }
    pub fn article_ids(&self, instrument: &str, kind: &str, number: &str) -> Result<Vec<String>> {
        Ok(self.connection.prepare("SELECT id FROM provisions WHERE instrument_id=?1 AND provision_type=?2 AND number=?3 ORDER BY input_path,ordinal")?.query_map(params![instrument,kind,number],|r|r.get(0))?.collect::<rusqlite::Result<_>>()?)
    }
    pub fn context_ids(&self, id: &str, radius: u8) -> Result<Vec<String>> {
        ensure!(radius <= 10, "context radius exceeds10");
        let (path, n): (String, i64) = self.connection.query_row(
            "SELECT input_path,ordinal FROM provisions WHERE id=?1",
            [id],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )?;
        Ok(self.connection.prepare("SELECT id FROM provisions WHERE input_path=?1 AND ordinal BETWEEN ?2 AND ?3 ORDER BY ordinal")?.query_map(params![path,n-i64::from(radius),n+i64::from(radius)],|r|r.get(0))?.collect::<rusqlite::Result<_>>()?)
    }
    /// Literal phrase escaping is independent of SQL parameter binding.
    pub fn search(
        &self,
        literal: &str,
        instrument: Option<&str>,
        kind: Option<&str>,
        limit: u32,
    ) -> Result<Vec<SearchHit>> {
        ensure!(
            !literal.trim().is_empty() && literal.chars().count() <= 512,
            "invalid search length"
        );
        ensure!((1..=100).contains(&limit), "search limit must be1..100");
        let phrase = format!("\"{}\"", literal.replace('"', "\"\""));
        Ok(self.connection.prepare("SELECT p.id,bm25(provisions_fts,10,3,1) AS score FROM provisions_fts JOIN provisions p ON p.row_id=provisions_fts.rowid WHERE provisions_fts MATCH ?1 AND (?2 IS NULL OR p.instrument_id=?2) AND (?3 IS NULL OR p.provision_type=?3) ORDER BY score,p.id LIMIT ?4")?.query_map(params![phrase,instrument,kind,limit],|r|Ok(SearchHit{id:r.get(0)?,score:r.get(1)?}))?.collect::<rusqlite::Result<_>>()?)
    }
}

#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct Golden {
    schema_version: String,
    queries: Vec<GoldenQuery>,
}
#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct GoldenQuery {
    name: String,
    sql: String,
    parameters: Vec<String>,
    expected: Value,
}
/// Fixed corpus acceptance fixtures, not arbitrary operator SQL execution.
pub fn golden(connection: &Connection) -> Result<Vec<String>> {
    let fixture: Golden = serde_json::from_str(include_str!(
        "../../../fixtures/database/golden-queries.json"
    ))?;
    ensure!(
        fixture.schema_version == "2",
        "unknown golden query version"
    );
    let mut passed = Vec::new();
    for query in fixture.queries {
        let mut stmt = connection
            .prepare(&query.sql)
            .with_context(|| query.name.clone())?;
        ensure!(stmt.readonly(), "golden query must be read-only");
        let width = stmt.column_count();
        let mut cursor = stmt.query(rusqlite::params_from_iter(&query.parameters))?;
        let mut actual = Vec::new();
        while let Some(row) = cursor.next()? {
            let mut cells = Vec::new();
            for column in 0..width {
                cells.push(match row.get_ref(column)? {
                    rusqlite::types::ValueRef::Null => Value::Null,
                    rusqlite::types::ValueRef::Integer(n) => json!(n),
                    rusqlite::types::ValueRef::Real(n) => json!(n),
                    rusqlite::types::ValueRef::Text(v) => json!(std::str::from_utf8(v)?),
                    rusqlite::types::ValueRef::Blob(_) => {
                        anyhow::bail!("unexpected golden query BLOB")
                    }
                });
            }
            actual.push(Value::Array(cells));
        }
        ensure!(
            Value::Array(actual) == query.expected,
            "golden query failed: {}",
            query.name
        );
        passed.push(query.name);
    }
    Ok(passed)
}
