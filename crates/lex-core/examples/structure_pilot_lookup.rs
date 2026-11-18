//! Local command-line lookup for the fintech structural pilot (`pilot-v1`).
//!
//! Loads the given `provisions.json` files (the instrument slug is the name
//! of the file's directory), builds one projection per provision at the
//! stated release, then reads one JSON request per line from standard input
//! and writes one JSON response per line. Read-only: it writes nothing but
//! standard output.
//!
//! ```text
//! cargo run --locked -p lex-core --example structure_pilot_lookup -- \
//!     --release local-<40-hex commit> corpus/mx/lritf/provisions.json … < requests.jsonl
//! cargo run --locked -p lex-core --example structure_pilot_lookup -- \
//!     --release local-<commit> --projection <provision_id> corpus/mx/lic/provisions.json
//! ```

use std::io::{BufRead, Write};
use std::path::Path;

use lex_core::structure_pilot::{LookupContext, is_eligible};
use serde_json::Value;

fn main() -> Result<(), String> {
    let mut args = std::env::args().skip(1);
    let mut release = None;
    let mut projection = None;
    let mut files = Vec::new();
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--release" => release = args.next(),
            "--projection" => projection = args.next(),
            _ => files.push(arg),
        }
    }
    let release = release.ok_or("--release local-<commit> is required")?;
    let mut context = LookupContext::new(&release);
    for file in &files {
        let path = Path::new(file);
        let slug = path
            .parent()
            .and_then(Path::file_name)
            .and_then(|name| name.to_str())
            .ok_or_else(|| format!("{file}: no instrument directory"))?;
        let text = std::fs::read_to_string(path).map_err(|e| format!("{file}: {e}"))?;
        let provisions: Vec<Value> =
            serde_json::from_str(&text).map_err(|e| format!("{file}: {e}"))?;
        for provision in &provisions {
            let (Some(id), Some(body)) = (provision["id"].as_str(), provision["text"].as_str())
            else {
                return Err(format!("{file}: provision without id or text"));
            };
            let kind = provision["provision_type"].as_str().unwrap_or_default();
            context.insert(id, body, is_eligible(slug, kind));
        }
    }
    let stdout = std::io::stdout();
    let mut out = stdout.lock();
    if let Some(id) = projection {
        let projection = context
            .projection(&id)
            .ok_or_else(|| format!("unknown provision {id}"))?;
        writeln!(out, "{}", projection.to_json()).map_err(|e| e.to_string())?;
        return Ok(());
    }
    for line in std::io::stdin().lock().lines() {
        let line = line.map_err(|e| e.to_string())?;
        if line.trim().is_empty() {
            continue;
        }
        let request: Value = serde_json::from_str(&line).map_err(|e| format!("request: {e}"))?;
        writeln!(out, "{}", context.lookup(&request)).map_err(|e| e.to_string())?;
    }
    Ok(())
}
