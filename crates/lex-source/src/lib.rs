use std::{collections::BTreeMap, fs, path::Path, process::Command, time::Duration};

use anyhow::{Context, Result, bail};
use chrono::Utc;
use lex_core::{SCHEMA_VERSION, SourceManifest};
use reqwest::{
    blocking::Client,
    header::{CONTENT_LENGTH, CONTENT_TYPE, ETAG, LAST_MODIFIED},
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use url::Url;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SourceFormat {
    Pdf,
    Html,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExternalInstrument {
    /// Lowercase official-name fragment that identifies the cited instrument
    /// in provision text, for example
    /// "ley para regular las instituciones de tecnología financiera".
    pub name_marker: String,
    pub instrument_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FormalSource {
    pub url: Url,
    pub format: SourceFormat,
    pub publisher: String,
    pub publication_code: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceConfig {
    pub slug: String,
    pub instrument_id: String,
    pub instrument_type: String,
    /// Deterministic parser used for this source: `lritf` for the statute
    /// vertical slice, `ifpe-dcg` for the CNBV/Banco de México DCG layout.
    /// The parser choice also selects the reference-extraction policy.
    pub parser: String,
    pub official_title: String,
    pub short_name: String,
    pub operational_source: String,
    pub source_url: Url,
    pub reference_url: Option<Url>,
    pub publisher: String,
    pub publication_date: String,
    pub expected_min_articles: usize,
    /// Exact article count for closed instruments; validation uses this in
    /// addition to the minimum when present.
    #[serde(default)]
    pub expected_articles: Option<usize>,
    pub expected_transitories: usize,
    #[serde(default)]
    pub expected_annexes: usize,
    #[serde(default)]
    pub issuing_authorities: Vec<String>,
    /// Formal publication acquired directly (for example, the DOF note that
    /// uniquely carries annex bodies). Distinct from
    /// `formal_publication_urls`, which only records locator URLs.
    #[serde(default)]
    pub formal_source: Option<FormalSource>,
    pub formal_publication_urls: BTreeMap<String, Url>,
    #[serde(default)]
    pub relevant_reform_transitories: BTreeMap<String, Vec<String>>,
    /// Articles whose source layout is a two-column term/definition table
    /// that requires column-aware reconstruction.
    #[serde(default)]
    pub definition_layout_articles: Vec<String>,
    /// Phrases that mark a citation as internal to this instrument.
    /// When absent, the parser's statute defaults apply.
    #[serde(default)]
    pub internal_reference_markers: Option<Vec<String>>,
    /// Named external instruments whose express citations should become
    /// cross-instrument reference edges.
    #[serde(default)]
    pub external_instruments: Vec<ExternalInstrument>,
    /// Public intermediate CA certificates (PEM, relative to the adapter
    /// file) for official hosts that serve an incomplete TLS chain, such as
    /// www.cnbv.gob.mx and www.dof.gob.mx. Each certificate still chains to
    /// a standard root.
    #[serde(default)]
    pub tls_intermediate_ca_pems: Vec<std::path::PathBuf>,
    /// PEM bytes loaded from `tls_intermediate_ca_pems` at config load time.
    #[serde(skip)]
    pub tls_intermediate_cas: Vec<Vec<u8>>,
}

#[derive(Debug)]
pub struct Acquisition {
    pub bytes: Vec<u8>,
    pub manifest: SourceManifest,
}

pub fn load_config(path: &Path) -> Result<SourceConfig> {
    let bytes = fs::read(path)
        .with_context(|| format!("failed to read adapter config {}", path.display()))?;
    let mut config: SourceConfig = serde_json::from_slice(&bytes)
        .with_context(|| format!("invalid adapter config {}", path.display()))?;
    for pem_path in &config.tls_intermediate_ca_pems {
        let resolved = path
            .parent()
            .map_or_else(|| pem_path.clone(), |parent| parent.join(pem_path));
        config.tls_intermediate_cas.push(
            fs::read(&resolved).with_context(|| {
                format!("failed to read intermediate CA {}", resolved.display())
            })?,
        );
    }
    Ok(config)
}

#[must_use]
pub fn discover(config: &SourceConfig) -> serde_json::Value {
    serde_json::json!({
        "slug": config.slug,
        "official_title": config.official_title,
        "operational_source": config.operational_source,
        "source_url": config.source_url,
        "reference_url": config.reference_url,
        "format": "pdf",
    })
}

pub fn fetch(config: &SourceConfig) -> Result<Acquisition> {
    fetch_resource(
        config,
        &config.source_url.clone(),
        SourceFormat::Pdf,
        &config.operational_source,
        &config.publisher,
    )
}

/// Fetch the formal publication configured for the instrument, when the
/// formal source itself must be acquired (for example, annex bodies published
/// only in the DOF note).
pub fn fetch_formal(config: &SourceConfig) -> Result<Option<Acquisition>> {
    let Some(formal) = &config.formal_source else {
        return Ok(None);
    };
    fetch_resource(
        config,
        &formal.url.clone(),
        formal.format,
        "dof",
        &formal.publisher,
    )
    .map(Some)
}

fn fetch_resource(
    config: &SourceConfig,
    url: &Url,
    format: SourceFormat,
    operational_source: &str,
    publisher: &str,
) -> Result<Acquisition> {
    let mut builder = Client::builder()
        .timeout(Duration::from_mins(1))
        .user_agent(concat!("lex-mex/", env!("CARGO_PKG_VERSION")));
    for pem in &config.tls_intermediate_cas {
        builder = builder.add_root_certificate(
            reqwest::Certificate::from_pem(pem).context("invalid intermediate CA certificate")?,
        );
    }
    let client = builder.build().context("failed to create HTTP client")?;

    let response = client
        .get(url.clone())
        .send()
        .with_context(|| format!("failed to download {url}"))?;
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
    verify_format(&bytes, format)?;
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
        instrument_id: config.instrument_id.clone(),
        operational_source: operational_source.to_owned(),
        formal_publication_source: "dof".to_owned(),
        publisher: publisher.to_owned(),
        official_url: url.clone(),
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

fn verify_format(bytes: &[u8], format: SourceFormat) -> Result<()> {
    match format {
        SourceFormat::Pdf => {
            if bytes.len() < 4 || &bytes[..4] != b"%PDF" {
                bail!("source is not a PDF; refusing to parse unexpected content");
            }
        }
        SourceFormat::Html => {
            let head: Vec<u8> = bytes
                .iter()
                .copied()
                .skip_while(u8::is_ascii_whitespace)
                .take(1)
                .collect();
            if head != b"<" {
                bail!("source is not an HTML document; refusing to parse unexpected content");
            }
        }
    }
    Ok(())
}

pub fn write_acquisition(
    acquisition: &Acquisition,
    source_path: &Path,
    manifest_path: &Path,
) -> Result<()> {
    if let Some(parent) = source_path.parent() {
        fs::create_dir_all(parent)?;
    }
    if let Some(parent) = manifest_path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(source_path, &acquisition.bytes)
        .with_context(|| format!("failed to write {}", source_path.display()))?;
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
    use super::{SourceFormat, sha256_hex, verify_format};

    #[test]
    fn computes_known_sha256() {
        assert_eq!(
            sha256_hex(b"lex-mex"),
            "a983fd20035b83efe2583e098ba966a63f59098f7fc8b83797ad43fda9afb54c"
        );
    }

    #[test]
    fn verifies_expected_source_formats() {
        assert!(verify_format(b"%PDF-1.5 rest", SourceFormat::Pdf).is_ok());
        assert!(verify_format(b"<html>", SourceFormat::Pdf).is_err());
        assert!(verify_format(b"\n\t <html>", SourceFormat::Html).is_ok());
        assert!(verify_format(b"%PDF-1.5", SourceFormat::Html).is_err());
    }
}
