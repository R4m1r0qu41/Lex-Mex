//! Versioned relational digest; FTS layout and environment-specific release provenance are verified separately.
use crate::digest;
use anyhow::Result;
use rusqlite::{Connection, types::ValueRef};
use std::collections::BTreeMap;

pub fn hash(c: &Connection) -> Result<String> {
    let tables=c.prepare("SELECT name FROM pragma_table_list WHERE schema='main' AND type='table' AND name NOT LIKE 'sqlite_%' AND name<>'corpus_release' ORDER BY name")?.query_map([],|r|r.get::<_,String>(0))?.collect::<rusqlite::Result<Vec<_>>>()?;
    let mut logical = BTreeMap::new();
    for table in tables {
        let mut stmt = c.prepare(&format!("SELECT * FROM \"{}\"", table.replace('"', "\"\"")))?;
        let width = stmt.column_count();
        let mut rows = stmt.query([])?;
        let mut values = Vec::new();
        while let Some(row) = rows.next()? {
            let mut fields = Vec::new();
            for i in 0..width {
                fields.push(match row.get_ref(i)? {
                    ValueRef::Null => serde_json::json!(["null"]),
                    ValueRef::Integer(n) => serde_json::json!(["integer", n]),
                    ValueRef::Real(n) => serde_json::json!(["real", n]),
                    ValueRef::Text(t) => serde_json::json!(["text", std::str::from_utf8(t)?]),
                    ValueRef::Blob(b) => serde_json::json!(["blob", b.len(), digest(b)]),
                });
            }
            values.push(serde_json::to_string(&fields)?);
        }
        values.sort();
        logical.insert(table, values);
    }
    Ok(digest(&serde_json::to_vec(&logical)?))
}

#[cfg(test)]
mod tests {
    #[test]
    fn environment_provenance_and_fts_storage_do_not_change_content_digest() {
        let mut c = crate::probe::connection().unwrap();
        crate::probe::build_fixture(
            &mut c,
            include_bytes!("../../../fixtures/database/adversarial.json"),
        )
        .unwrap();
        c.execute(
            "INSERT INTO corpus_release VALUES('first','{\"sqlite\":\"environment-a\"}')",
            [],
        )
        .unwrap();
        let before = super::hash(&c).unwrap();
        c.execute(
            "UPDATE corpus_release SET id='second',specification='{\"sqlite\":\"environment-b\"}'",
            [],
        )
        .unwrap();
        c.execute(
            "INSERT INTO provisions_fts(provisions_fts) VALUES('delete-all')",
            [],
        )
        .unwrap();
        assert_eq!(before, super::hash(&c).unwrap());
        c.execute(
            "UPDATE provisions SET text='changed content' WHERE ordinal=0",
            [],
        )
        .unwrap();
        assert_ne!(before, super::hash(&c).unwrap());
    }
}
