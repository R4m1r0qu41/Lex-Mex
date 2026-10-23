use std::fmt::Write;
use std::process::Command;
fn visit(root: &std::path::Path, dir: &std::path::Path, out: &mut Vec<String>) {
    let mut entries = std::fs::read_dir(dir)
        .unwrap()
        .map(|e| e.unwrap().path())
        .collect::<Vec<_>>();
    entries.sort();
    for path in entries {
        if path.is_dir() {
            visit(root, &path, out);
        } else if path.extension().is_some_and(|e| e == "rs" || e == "json") {
            out.push(
                path.strip_prefix(root)
                    .unwrap()
                    .to_str()
                    .unwrap()
                    .to_owned(),
            );
        }
    }
}

fn main() {
    let rustc = std::env::var("RUSTC").expect("Cargo supplies RUSTC");
    let output = Command::new(rustc)
        .arg("--version")
        .output()
        .expect("rustc version");
    assert!(output.status.success());
    println!(
        "cargo:rustc-env=LEX_DB_RUSTC={}",
        String::from_utf8(output.stdout)
            .expect("UTF-8 version")
            .trim()
    );
    let root = std::path::PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap()).join("../..");
    let mut files = Vec::new();
    let dirs = [
        "crates/lex-db/src",
        "crates/lex-analysis/src",
        "crates/lex-core/src",
        "crates/lex-grammar/src",
        "crates/lex-link/src",
        "crates/lex-parse/src",
        "schemas",
        "fixtures/database",
    ];
    for dir in dirs {
        visit(&root, &root.join(dir), &mut files);
    }
    for file in [
        "crates/lex-analysis/Cargo.toml",
        "crates/lex-core/Cargo.toml",
        "crates/lex-grammar/Cargo.toml",
        "crates/lex-link/Cargo.toml",
        "crates/lex-parse/Cargo.toml",
        "crates/lex-cli/src/db.rs",
    ] {
        files.push(file.into());
    }
    // Directory-level triggers: adding a file to a pinned directory reruns
    // this script, not only editing a file already in the inventory.
    for dir in dirs {
        println!("cargo:rerun-if-changed={}", root.join(dir).display());
    }
    let mut code = String::from("&[\n");
    for path in files {
        let full = root.join(&path).canonicalize().unwrap();
        println!("cargo:rerun-if-changed={}", full.display());
        writeln!(
            code,
            "({path:?},include_bytes!({:?}).as_slice()),",
            full.to_str().unwrap()
        )
        .unwrap();
    }
    code.push(']');
    std::fs::write(
        std::path::PathBuf::from(std::env::var("OUT_DIR").unwrap()).join("builder_inputs.rs"),
        code,
    )
    .unwrap();
    let listed = dirs
        .iter()
        .map(|dir| format!("{dir:?}"))
        .collect::<Vec<_>>()
        .join(", ");
    std::fs::write(
        std::path::PathBuf::from(std::env::var("OUT_DIR").unwrap()).join("builder_dirs.rs"),
        format!("&[{listed}]"),
    )
    .unwrap();
    println!("cargo:rerun-if-changed=build.rs");
}
