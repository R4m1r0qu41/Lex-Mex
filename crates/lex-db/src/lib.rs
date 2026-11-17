//! Immutable `SQLite` corpus projection, independent verification and read-only queries.
pub mod build;
pub mod contract;
pub mod input;
pub mod load;
pub mod logical;
mod parity;
pub mod probe;
pub mod queries;
pub mod validation;
pub mod verify;

use sha2::{Digest, Sha256};

#[must_use]
/// The SQLite sidecar files of `database`: SQLite appends `-wal`, `-shm` and
/// `-journal` to the full file name, whatever its extension (`lex.db` has
/// `lex.db-wal`), so `Path::with_extension` would miss them (Codex audit
/// 2026-11-17, R5).
pub fn sqlite_sidecars(database: &std::path::Path) -> [std::path::PathBuf; 3] {
    ["-wal", "-shm", "-journal"].map(|suffix| {
        let mut name = database.as_os_str().to_owned();
        name.push(suffix);
        std::path::PathBuf::from(name)
    })
}

#[must_use]
pub fn digest(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

#[cfg(test)]
mod sidecar_tests {
    #[test]
    fn sidecars_follow_the_full_file_name_whatever_the_extension() {
        let names = super::sqlite_sidecars(std::path::Path::new("/x/lex.db"));
        assert_eq!(names[0], std::path::Path::new("/x/lex.db-wal"));
        let sqlite = super::sqlite_sidecars(std::path::Path::new("/x/corpus.sqlite"));
        assert_eq!(sqlite[1], std::path::Path::new("/x/corpus.sqlite-shm"));
        assert_eq!(sqlite[2], std::path::Path::new("/x/corpus.sqlite-journal"));
    }
}
