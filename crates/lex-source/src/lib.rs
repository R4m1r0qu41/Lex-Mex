use std::{fs, path::Path, process::Command, time::Duration};

use anyhow::{Context, Result, bail};
use chrono::Utc;
use lex_core::{LRITF_INSTRUMENT_ID, SCHEMA_VERSION, SourceManifest};
use reqwest::{
    blocking::Client,
    header::{CONTENT_LENGTH, CONTENT_TYPE, ETAG, LAST_MODIFIED},
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use url::Url;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceConfig {
    pub slug: String,
    pub official_title: String,
    pub short_name: String,
    pub source_url: Url,
    pub reference_url: Option<Url>,
    pub publisher: String,
    pub publication_date: String,
    pub expected_min_articles: usize,
    pub expected_transitories: usize,
}

#[derive(Debug)]
pub struct Acquisition {
    pub bytes: Vec<u8>,
    pub manifest: SourceManifest,
}

pub fn load_config(path: &Path) -> Result<SourceConfig> {
    let bytes = fs::read(path)
        .with_context(|| format!("failed to read adapter config {}", path.display()))?;
    serde_json::from_slice(&bytes)
        .with_context(|| format!("invalid adapter config {}", path.display()))
}

#[must_use]
pub fn discover(config: &SourceConfig) -> serde_json::Value {
    serde_json::json!({
        "slug": config.slug,
        "official_title": config.official_title,
        "operational_source": "camara_de_diputados",
        "source_url": config.source_url,
        "reference_url": config.reference_url,
        "format": "pdf",
    })
}

pub fn fetch(config: &SourceConfig) -> Result<Acquisition> {
    let client = Client::builder()
        .timeout(Duration::from_mins(1))
        .user_agent(concat!("lex-mex/", env!("CARGO_PKG_VERSION")))
        .build()
        .context("failed to create HTTP client")?;

    let response = client
        .get(config.source_url.clone())
        .send()
        .with_context(|| format!("failed to download {}", config.source_url))?;
    let status = response.status();
    if !status.is_success() {
        bail!("source returned HTTP {status}");
    }

    let headers = response.headers().clone();
    let declared_length = headers
        .get(CONTENT_LENGTH)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.parse::<u64>().ok());
    let bytes = response
        .bytes()
        .context("failed to read source body")?
        .to_vec();
    if bytes.len() < 4 || &bytes[..4] != b"%PDF" {
        bail!("source is not a PDF; refusing to parse unexpected content");
    }
    if let Some(expected) = declared_length
        && expected != bytes.len() as u64
    {
        bail!(
            "content length mismatch: header declared {expected}, received {}",
            bytes.len()
        );
    }

    let manifest = SourceManifest {
        schema_version: SCHEMA_VERSION.to_owned(),
        instrument_id: LRITF_INSTRUMENT_ID.to_owned(),
        operational_source: "camara_de_diputados".to_owned(),
        formal_publication_source: "dof".to_owned(),
        publisher: config.publisher.clone(),
        official_url: config.source_url.clone(),
        reference_url: config.reference_url.clone(),
        retrieved_at: Utc::now(),
        http_status: status.as_u16(),
        content_type: header_string(&headers, CONTENT_TYPE),
        content_length: Some(bytes.len() as u64),
        etag: header_string(&headers, ETAG),
        last_modified: header_string(&headers, LAST_MODIFIED),
        source_sha256: sha256_hex(&bytes),
        extracted_text_sha256: None,
        extraction_tool: None,
        parser_version: env!("CARGO_PKG_VERSION").to_owned(),
        schema_version_used: SCHEMA_VERSION.to_owned(),
        resulting_git_commit: git_commit(),
    };

    Ok(Acquisition { bytes, manifest })
}

pub fn write_acquisition(
    acquisition: &Acquisition,
    pdf_path: &Path,
    manifest_path: &Path,
) -> Result<()> {
    if let Some(parent) = pdf_path.parent() {
        fs::create_dir_all(parent)?;
    }
    if let Some(parent) = manifest_path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(pdf_path, &acquisition.bytes)
        .with_context(|| format!("failed to write {}", pdf_path.display()))?;
    write_manifest(&acquisition.manifest, manifest_path)
}

pub fn write_manifest(manifest: &SourceManifest, path: &Path) -> Result<()> {
    let json = serde_json::to_vec_pretty(manifest)?;
    fs::write(path, json).with_context(|| format!("failed to write {}", path.display()))
}

#[must_use]
pub fn sha256_hex(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn header_string(
    headers: &reqwest::header::HeaderMap,
    name: reqwest::header::HeaderName,
) -> Option<String> {
    headers
        .get(name)
        .and_then(|value| value.to_str().ok())
        .map(ToOwned::to_owned)
}

fn git_commit() -> Option<String> {
    let output = Command::new("git")
        .args(["rev-parse", "HEAD"])
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    String::from_utf8(output.stdout)
        .ok()
        .map(|value| value.trim().to_owned())
        .filter(|value| !value.is_empty())
}

#[cfg(test)]
mod tests {
    use super::sha256_hex;

    #[test]
    fn computes_known_sha256() {
        assert_eq!(
            sha256_hex(b"lex-mex"),
            "a983fd20035b83efe2583e098ba966a63f59098f7fc8b83797ad43fda9afb54c"
        );
    }
}
