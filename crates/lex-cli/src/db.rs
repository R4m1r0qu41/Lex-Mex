use anyhow::Result;
use clap::Subcommand;
use std::{
    fs,
    path::{Path, PathBuf},
};

#[derive(Debug, Subcommand)]
pub enum DbCommand {
    /// Build and independently read back a small in-memory adversarial fixture.
    ContractProbe,
    /// Independently verify an immutable release against pinned Git inputs.
    Verify {
        #[arg(long)]
        database: PathBuf,
        #[arg(long)]
        manifest: PathBuf,
        #[arg(long)]
        corpus_commit: String,
    },
    /// Run the fixed corpus golden-query acceptance fixtures read-only.
    GoldenQueries {
        #[arg(long)]
        database: PathBuf,
    },
    /// Build an immutable local projection from pinned corpus and builder commits.
    Project {
        #[arg(long)]
        corpus_commit: String,
        #[arg(long)]
        builder_commit: String,
        #[arg(long, default_value = "derived/db")]
        output_dir: PathBuf,
    },
    /// Inventory immutable Git inputs; requires the committed production debt seed.
    InspectInputs {
        #[arg(long)]
        corpus_commit: String,
    },
    /// Verify a public release against an accepted release and write a new binding.
    PublicBind {
        /// Accepted release directory holding `corpus.sqlite` and `manifest.json`.
        #[arg(long)]
        database_dir: PathBuf,
        /// Public release directory holding its `manifest.json`.
        #[arg(long)]
        release_dir: PathBuf,
        #[arg(long)]
        output: PathBuf,
    },
    /// Serve bounded public queries read-only: one JSON request per stdin line,
    /// or one `--request`; refuses to start on a missing or stale binding.
    PublicQuery {
        #[arg(long)]
        database: PathBuf,
        #[arg(long)]
        binding: PathBuf,
        #[arg(long)]
        request: Option<String>,
        /// Report per-request wall time in microseconds on stderr.
        #[arg(long)]
        timings: bool,
    },
    /// Validate an explicitly proposed seed against pinned canonical Git blobs.
    CheckSeed {
        #[arg(long)]
        corpus_commit: String,
        #[arg(long)]
        seed: PathBuf,
    },
}

pub fn run(root: &Path, command: DbCommand) -> Result<()> {
    match command {
        DbCommand::Verify {
            database,
            manifest,
            corpus_commit,
        } => println!(
            "{}",
            serde_json::to_string_pretty(&lex_db::verify::verify(
                root,
                &resolve(root, &database),
                &resolve(root, &manifest),
                &corpus_commit
            )?)?
        ),
        DbCommand::GoldenQueries { database } => {
            let c = lex_db::verify::open_readonly(&resolve(root, &database))?;
            println!(
                "{}",
                serde_json::to_string_pretty(&lex_db::queries::golden(&c)?)?
            );
        }
        DbCommand::Project {
            corpus_commit,
            builder_commit,
            output_dir,
        } => println!(
            "{}",
            serde_json::to_string_pretty(&lex_db::build::project(
                root,
                &corpus_commit,
                &builder_commit,
                &output_dir
            )?)?
        ),
        DbCommand::ContractProbe => println!(
            "{}",
            serde_json::to_string_pretty(&lex_db::probe::run_probe()?)?
        ),
        DbCommand::InspectInputs { corpus_commit } => println!(
            "{}",
            serde_json::to_string_pretty(&lex_db::input::inspect(root, &corpus_commit)?)?
        ),
        DbCommand::PublicBind {
            database_dir,
            release_dir,
            output,
        } => {
            let binding =
                lex_db::public::bind(&resolve(root, &database_dir), &resolve(root, &release_dir))?;
            lex_db::public::write_binding(&binding, &resolve(root, &output))?;
            println!("{}", serde_json::to_string_pretty(&binding)?);
        }
        DbCommand::PublicQuery {
            database,
            binding,
            request,
            timings,
        } => public_query(
            &resolve(root, &database),
            &resolve(root, &binding),
            request.as_deref(),
            timings,
        )?,
        DbCommand::CheckSeed {
            corpus_commit,
            seed,
        } => {
            let git = lex_db::input::GitInput::new(root, &corpus_commit)?;
            let bytes = fs::read(if seed.is_absolute() {
                seed
            } else {
                root.join(seed)
            })?;
            let result = lex_db::contract::validate_seed(&git, &bytes)?;
            println!(
                "validated {} accepted debt rows from candidate seed against {corpus_commit}",
                result.rows.len()
            );
        }
    }
    Ok(())
}

fn public_query(
    database: &Path,
    binding: &Path,
    request: Option<&str>,
    timings: bool,
) -> Result<()> {
    use lex_db::public::REQUEST_BYTES;
    use std::io::Write;
    let reader =
        lex_db::public::PublicReader::open(database, binding, lex_db::public::Limits::default())?;
    let emit = |response: &serde_json::Value, micros: u128| -> Result<()> {
        let mut out = std::io::stdout().lock();
        serde_json::to_writer(&mut out, response)?;
        writeln!(out)?;
        out.flush()?;
        if timings {
            eprintln!(
                "{}",
                serde_json::json!({"micros": micros, "operation": response["operation"], "outcome": response["outcome"], "bytes": response.to_string().len()})
            );
        }
        Ok(())
    };
    let answer = |line: &str| -> Result<()> {
        let started = std::time::Instant::now();
        let response = reader.handle(line);
        emit(&response, started.elapsed().as_micros())
    };
    match request {
        Some(line) => answer(line)?,
        None => read_frames(
            std::io::stdin().lock(),
            REQUEST_BYTES,
            |frame| match frame {
                Frame::Request(line) if line.trim().is_empty() => Ok(()),
                Frame::Request(line) => answer(&line),
                Frame::Oversized => emit(&reader.reject("request exceeds 8192 bytes"), 0),
                Frame::InvalidUtf8 => emit(&reader.reject("request is not valid UTF-8"), 0),
            },
        )?,
    }
    Ok(())
}

/// One stdin request. Frames end at LF; one CR before the LF is part of the
/// delimiter, and delimiters do not count toward the byte limit. A final
/// frame without LF is accepted at end of input.
#[derive(Debug, PartialEq, Eq)]
enum Frame {
    Request(String),
    Oversized,
    InvalidUtf8,
}

/// Split input into frames while buffering at most `cap + 1` bytes. An
/// over-limit frame is reported as soon as the limit is exceeded, before more
/// input is read; the rest of that frame is then discarded through its LF
/// without being stored, and the next frame is served normally. Ingress
/// timeouts belong to the serving host, not to this reader.
fn read_frames<R: std::io::BufRead>(
    mut input: R,
    cap: usize,
    mut each: impl FnMut(Frame) -> Result<()>,
) -> Result<()> {
    let mut frame = Vec::with_capacity(cap + 1);
    let mut discarding = false;
    let finish = |frame: &mut Vec<u8>| {
        if frame.last() == Some(&b'\r') {
            frame.pop();
        }
        String::from_utf8(std::mem::take(frame)).map_or(Frame::InvalidUtf8, Frame::Request)
    };
    loop {
        let chunk = input.fill_buf()?;
        if chunk.is_empty() {
            if !discarding && !frame.is_empty() {
                each(finish(&mut frame))?;
            }
            return Ok(());
        }
        let take = chunk
            .iter()
            .position(|b| *b == b'\n')
            .map_or(chunk.len(), |i| i + 1);
        let terminated = chunk[take - 1] == b'\n';
        let body = &chunk[..take - usize::from(terminated)];
        if discarding {
            discarding = !terminated;
        } else {
            // Room for `cap` content bytes plus a CR that may be the delimiter.
            let room = (cap + 1).saturating_sub(frame.len());
            frame.extend_from_slice(&body[..body.len().min(room)]);
            if body.len() > room || (frame.len() == cap + 1 && frame.last() != Some(&b'\r')) {
                frame.clear();
                discarding = !terminated;
                input.consume(take);
                each(Frame::Oversized)?;
                continue;
            }
            if terminated {
                let request = finish(&mut frame);
                input.consume(take);
                each(request)?;
                continue;
            }
        }
        input.consume(take);
    }
}

fn resolve(root: &Path, path: &Path) -> PathBuf {
    if path.is_absolute() {
        path.to_owned()
    } else {
        root.join(path)
    }
}

#[cfg(test)]
mod tests {
    use super::{Frame, read_frames};
    use std::{cell::Cell, io::BufRead, rc::Rc};

    fn frames(input: &[u8], cap: usize) -> Vec<Frame> {
        let mut out = Vec::new();
        read_frames(input, cap, |frame| {
            out.push(frame);
            Ok(())
        })
        .unwrap();
        out
    }

    fn request(text: &str) -> Frame {
        Frame::Request(text.to_owned())
    }

    #[test]
    fn exact_limit_crlf_multibyte_and_eof_are_defined() {
        assert_eq!(frames(b"12345678\n", 8), [request("12345678")]);
        assert_eq!(
            frames(b"12345678\r\nok", 8),
            [request("12345678"), request("ok")]
        );
        assert_eq!(
            frames(b"123456789\nnext\n", 8),
            [Frame::Oversized, request("next")]
        );
        assert_eq!(frames(b"12345678\r9\n", 8), [Frame::Oversized]);
        assert_eq!(frames("áááá\n".as_bytes(), 8), [request("áááá")]);
        assert_eq!(
            frames("ááááá\nok".as_bytes(), 8),
            [Frame::Oversized, request("ok")]
        );
        assert_eq!(
            frames(b"\xff\xfe\nok\n", 8),
            [Frame::InvalidUtf8, request("ok")]
        );
        assert_eq!(frames(b"tail", 8), [request("tail")]);
        assert_eq!(frames(b"123456789", 8), [Frame::Oversized]);
        assert_eq!(frames(b"\n\r\n", 8), [request(""), request("")]);
        assert!(frames(b"", 8).is_empty());
    }

    /// Yields `chunks` lazily and counts reads past the first chunk.
    struct Lazy {
        chunk: Vec<u8>,
        left: usize,
        tail: Vec<u8>,
        reads: Rc<Cell<usize>>,
        offset: usize,
    }

    impl std::io::Read for Lazy {
        fn read(&mut self, _: &mut [u8]) -> std::io::Result<usize> {
            unreachable!("framing reads through BufRead")
        }
    }

    impl BufRead for Lazy {
        fn fill_buf(&mut self) -> std::io::Result<&[u8]> {
            self.reads.set(self.reads.get() + 1);
            if self.left > 0 {
                Ok(&self.chunk[self.offset..])
            } else {
                Ok(&self.tail[self.offset.min(self.tail.len())..])
            }
        }
        fn consume(&mut self, amount: usize) {
            self.offset += amount;
            if self.left > 0 && self.offset == self.chunk.len() {
                self.left -= 1;
                self.offset = 0;
            }
        }
    }

    #[test]
    fn oversized_unterminated_input_is_rejected_before_more_is_read() {
        let reads = Rc::new(Cell::new(0));
        // 16 MiB without a newline, delivered lazily, then one valid request.
        let input = Lazy {
            chunk: vec![b'x'; 64 * 1024],
            left: 256,
            tail: b"\n{\"operation\":\"coverage\"}\n".to_vec(),
            reads: Rc::clone(&reads),
            offset: 0,
        };
        let mut seen = Vec::new();
        read_frames(input, 8192, |frame| {
            seen.push((reads.get(), frame));
            Ok(())
        })
        .unwrap();
        assert_eq!(
            seen[0],
            (1, Frame::Oversized),
            "rejected on the first chunk"
        );
        assert_eq!(seen[1].1, request("{\"operation\":\"coverage\"}"));
        assert_eq!(seen.len(), 2);
    }
}
