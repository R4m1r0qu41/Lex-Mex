//! Text extraction from retained PDF, DOC and HTML sources.

use std::{
    fs,
    path::Path,
    process::{Command, Stdio},
};

use anyhow::{Context, Result, bail};

#[derive(Debug)]
pub struct Extraction {
    pub text: String,
    pub tool_version: String,
}

/// Extract text from a PDF with Poppler. `keep_page_breaks` retains the
/// form-feed page markers; the DCG parser needs them to merge paragraphs
/// deterministically across page boundaries.
pub fn extract_pdf(
    pdf_path: &Path,
    text_path: &Path,
    keep_page_breaks: bool,
) -> Result<Extraction> {
    if let Some(parent) = text_path.parent() {
        fs::create_dir_all(parent)?;
    }
    let mut args = vec!["-layout"];
    if !keep_page_breaks {
        args.push("-nopgbrk");
    }
    let status = Command::new("pdftotext")
        .args(args)
        .arg(pdf_path)
        .arg(text_path)
        .stdin(Stdio::null())
        .status()
        .context("failed to execute pdftotext; install Poppler")?;
    if !status.success() {
        bail!("pdftotext failed with status {status}");
    }
    let text = fs::read_to_string(text_path)
        .with_context(|| format!("failed to read {}", text_path.display()))?;
    if text.trim().is_empty() {
        bail!("pdftotext produced empty output");
    }
    Ok(Extraction {
        text,
        tool_version: pdftotext_version(),
    })
}

/// Extract text from a legacy binary Microsoft Word document without
/// altering the acquired source bytes. macOS provides `textutil`; other
/// environments may provide `antiword`. The chosen executable and its
/// version are recorded in the source manifest.
pub fn extract_doc(doc_path: &Path, text_path: &Path) -> Result<Extraction> {
    if let Some(parent) = text_path.parent() {
        fs::create_dir_all(parent)?;
    }

    let textutil = Command::new("textutil")
        .args(["-convert", "txt", "-stdout"])
        .arg(doc_path)
        .stdin(Stdio::null())
        .output();
    if let Ok(output) = textutil
        && output.status.success()
        && !output.stdout.iter().all(u8::is_ascii_whitespace)
    {
        let text = String::from_utf8(output.stdout)
            .context("textutil produced non-UTF-8 output for Word source")?;
        fs::write(text_path, &text)?;
        return Ok(Extraction {
            text,
            tool_version: command_version("textutil", &["-help"])
                .unwrap_or_else(|| "textutil (version unavailable)".to_owned()),
        });
    }

    let antiword = Command::new("antiword")
        .arg(doc_path)
        .stdin(Stdio::null())
        .output()
        .context("failed to extract Word source; install textutil (macOS) or antiword")?;
    if !antiword.status.success() {
        bail!("antiword failed with status {}", antiword.status);
    }
    let text = String::from_utf8(antiword.stdout).context("antiword produced non-UTF-8 output")?;
    if text.trim().is_empty() {
        bail!("Word extraction produced empty output");
    }
    fs::write(text_path, &text)?;
    Ok(Extraction {
        text,
        tool_version: command_version("antiword", &["-h"])
            .unwrap_or_else(|| "antiword (version unavailable)".to_owned()),
    })
}

fn pdftotext_version() -> String {
    Command::new("pdftotext")
        .arg("-v")
        .output()
        .ok()
        .and_then(|output| {
            let text = if output.stderr.is_empty() {
                String::from_utf8(output.stdout).ok()?
            } else {
                String::from_utf8(output.stderr).ok()?
            };
            text.lines().next().map(str::to_owned)
        })
        .unwrap_or_else(|| "pdftotext (version unavailable)".to_owned())
}

fn command_version(command: &str, args: &[&str]) -> Option<String> {
    let output = Command::new(command).args(args).output().ok()?;
    let bytes = if output.stdout.is_empty() {
        output.stderr
    } else {
        output.stdout
    };
    String::from_utf8(bytes)
        .ok()?
        .lines()
        .find(|line| !line.trim().is_empty())
        .map(|line| format!("{command}: {}", line.trim()))
}
