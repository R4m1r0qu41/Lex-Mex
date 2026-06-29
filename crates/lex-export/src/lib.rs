use std::{fmt::Write as _, fs, path::Path};

use anyhow::{Context, Result};
use lex_core::{Corpus, Provision, ProvisionType, ReviewItem, ReviewItemStatus, ValidationReport};

pub fn write_canonical(corpus: &Corpus, output_dir: &Path) -> Result<()> {
    fs::create_dir_all(output_dir)?;
    write_json(&corpus.instrument, &output_dir.join("instrument.json"))?;
    write_json(&corpus.provisions, &output_dir.join("provisions.json"))
}

pub fn write_validation(report: &ValidationReport, output_dir: &Path) -> Result<()> {
    fs::create_dir_all(output_dir)?;
    write_json(report, &output_dir.join("validation.json"))
}

pub fn write_markdown(corpus: &Corpus, output_dir: &Path) -> Result<()> {
    fs::create_dir_all(output_dir)?;
    for provision in &corpus.provisions {
        let filename = markdown_filename(provision);
        let content = standard_markdown(corpus, provision);
        fs::write(output_dir.join(filename), content)?;
    }
    fs::write(output_dir.join("README.md"), markdown_index(corpus, false))?;
    Ok(())
}

pub fn write_obsidian(
    corpus: &Corpus,
    review_items: &[ReviewItem],
    output_dir: &Path,
) -> Result<()> {
    let instrument_dir = output_dir
        .join("Corpus")
        .join(&corpus.instrument.short_name);
    fs::create_dir_all(&instrument_dir)?;
    let mut generated_files = Vec::with_capacity(corpus.provisions.len() + 1);
    for provision in &corpus.provisions {
        let filename = markdown_filename(provision);
        let content = obsidian_markdown(corpus, provision);
        fs::write(instrument_dir.join(&filename), content)?;
        generated_files.push(filename);
    }
    let index_filename = format!("{}.md", corpus.instrument.short_name);
    fs::write(instrument_dir.join(&index_filename), obsidian_index(corpus))?;
    generated_files.push(index_filename);
    write_json(
        &serde_json::json!({
            "schema_version": "0.1.0",
            "instrument_id": corpus.instrument.id,
            "source_sha256": corpus.instrument.source_sha256,
            "generated_files": generated_files,
        }),
        &instrument_dir.join("_lex-mex-export.json"),
    )?;
    fs::write(
        output_dir.join("Corpus/Revisiones pendientes.md"),
        obsidian_review_queue(review_items),
    )?;
    Ok(())
}

fn write_json<T: serde::Serialize>(value: &T, path: &Path) -> Result<()> {
    let json = serde_json::to_vec_pretty(value)?;
    fs::write(path, json).with_context(|| format!("failed to write {}", path.display()))
}

fn front_matter(corpus: &Corpus, provision: &Provision) -> String {
    let alias = format!("{} — {}", corpus.instrument.short_name, provision.label);
    format!(
        "---\nid: {}\ninstrument_id: {}\ninstrument: {}\nprovision_type: {}\nnumber: \"{}\"\naliases: [{}]\ngenerated: true\ntemporal_status: {}\nreview_status: {}\nsource_url: {}\nsource_sha256: {}\n---\n\n",
        provision.id,
        provision.instrument_id,
        corpus.instrument.short_name,
        json_name(&provision.provision_type),
        provision.number,
        serde_json::to_string(&alias).expect("serializing alias cannot fail"),
        json_name(&provision.temporal_status),
        json_name(&provision.review_status),
        corpus.instrument.source_url,
        corpus.instrument.source_sha256,
    )
}

fn standard_markdown(corpus: &Corpus, provision: &Provision) -> String {
    let mut output = front_matter(corpus, provision);
    if let Some(title) = &provision.heading_context.title {
        let _ = write!(output, "> {title}");
        if let Some(chapter) = &provision.heading_context.chapter {
            let _ = write!(output, " · {chapter}");
        }
        output.push_str("\n\n");
    }
    let _ = write!(output, "# {}\n\n{}\n", provision.label, provision.text);
    output
}

fn obsidian_markdown(corpus: &Corpus, provision: &Provision) -> String {
    let mut output = front_matter(corpus, provision);
    let _ = write!(
        output,
        "[[Corpus/{0}/{0}|← Índice {0}]]\n\n",
        corpus.instrument.short_name
    );
    let _ = write!(output, "# {}\n\n{}\n", provision.label, provision.text);
    output
}

fn obsidian_index(corpus: &Corpus) -> String {
    let mut output = format!(
        "---\nid: {}\naliases: [{}]\ngenerated: true\nsource_url: {}\nsource_sha256: {}\n---\n\n[[Inicio|← Inicio]]\n\n",
        corpus.instrument.id,
        serde_json::to_string(&corpus.instrument.official_title)
            .expect("serializing alias cannot fail"),
        corpus.instrument.source_url,
        corpus.instrument.source_sha256,
    );
    output.push_str(&markdown_index(corpus, true));
    output
}

fn obsidian_review_queue(items: &[ReviewItem]) -> String {
    let mut output = String::from(
        "---\ngenerated: true\n---\n\n[[Inicio|← Inicio]]\n\n\
         # Revisiones temporales pendientes\n\n\
         > [!warning]\n\
         > This dashboard is generated. Record legal decisions in the human-authored \
         Revisiones folder.\n\n",
    );
    let pending: Vec<_> = items
        .iter()
        .filter(|item| item.status == ReviewItemStatus::Pending)
        .collect();
    if pending.is_empty() {
        output.push_str("No hay revisiones pendientes.\n");
        return output;
    }
    for item in pending {
        let determination = &item.proposed_machine_conclusion;
        let _ = write!(
            output,
            "## {}\n\n\
             - **ID:** {}\n\
             - **Conclusión propuesta:** {}\n\
             - **Confianza:** {:.2}\n\
             - **Problema:** {}\n\
             - **Fuente:** [Cámara de Diputados]({})\n\n\
             **Texto relevante**\n\n> {}\n\n",
            item.evidence.label,
            item.provision_id,
            json_name(&determination.temporal_status),
            determination.confidence,
            item.exact_issue,
            item.camara_source_url,
            item.evidence.text.replace('\n', "\n> "),
        );
    }
    output
}

fn markdown_index(corpus: &Corpus, obsidian: bool) -> String {
    let mut output = format!(
        "# {}\n\nFuente operativa: [{}]({})\n\n",
        corpus.instrument.official_title,
        corpus.instrument.operational_source,
        corpus.instrument.source_url
    );
    output.push_str("## Artículos\n\n");
    for provision in corpus
        .provisions
        .iter()
        .filter(|item| item.provision_type == ProvisionType::Article)
    {
        output.push_str(&index_link(provision, obsidian));
    }
    output.push_str("\n## Disposiciones transitorias\n\n");
    for provision in corpus
        .provisions
        .iter()
        .filter(|item| item.provision_type == ProvisionType::Transitory)
    {
        output.push_str(&index_link(provision, obsidian));
    }
    output
}

fn index_link(provision: &Provision, obsidian: bool) -> String {
    let stem = markdown_filename(provision)
        .trim_end_matches(".md")
        .to_owned();
    if obsidian {
        format!("- [[{stem}|{}]]\n", provision.label)
    } else {
        format!("- [{}]({stem}.md)\n", provision.label)
    }
}

fn markdown_filename(provision: &Provision) -> String {
    match provision.provision_type {
        ProvisionType::Article => format!("articulo-{}.md", provision.number.replace(' ', "-")),
        ProvisionType::Transitory => format!(
            "transitorio-{}.md",
            provision
                .number
                .to_lowercase()
                .replace('á', "a")
                .replace('é', "e")
                .replace('í', "i")
                .replace('ó', "o")
                .replace('ú', "u")
                .replace(' ', "-")
        ),
    }
}

fn json_name<T: serde::Serialize>(value: &T) -> String {
    serde_json::to_string(value)
        .expect("serializing enum cannot fail")
        .trim_matches('"')
        .to_owned()
}

#[cfg(test)]
mod tests {
    use std::fs;

    use lex_core::{
        Corpus, HeadingContext, Instrument, InstrumentStatus, InstrumentType, LRITF_INSTRUMENT_ID,
        Provision, ProvisionType, ReviewStatus, SCHEMA_VERSION, TemporalStatus,
    };
    use tempfile::tempdir;

    use super::{markdown_filename, write_obsidian};

    #[test]
    fn produces_stable_presentation_filename() {
        let provision = sample_provision();
        assert_eq!(
            markdown_filename(&provision),
            "transitorio-decima-primera.md"
        );
    }

    #[test]
    fn publishes_below_corpus_without_touching_human_notes() {
        let temp = tempdir().unwrap();
        let notes = temp.path().join("Notas");
        fs::create_dir_all(&notes).unwrap();
        fs::write(notes.join("criterio.md"), "Conservar.").unwrap();
        let corpus = Corpus {
            instrument: sample_instrument(),
            provisions: vec![sample_provision()],
        };

        write_obsidian(&corpus, &[], temp.path()).unwrap();

        assert_eq!(
            fs::read_to_string(notes.join("criterio.md")).unwrap(),
            "Conservar."
        );
        assert!(
            temp.path()
                .join("Corpus/LRITF/transitorio-decima-primera.md")
                .is_file()
        );
        assert!(temp.path().join("Corpus/LRITF/LRITF.md").is_file());
        assert!(
            temp.path()
                .join("Corpus/LRITF/_lex-mex-export.json")
                .is_file()
        );
        assert!(
            temp.path()
                .join("Corpus/Revisiones pendientes.md")
                .is_file()
        );
    }

    fn sample_provision() -> Provision {
        Provision {
            schema_version: SCHEMA_VERSION.to_owned(),
            id: format!("{LRITF_INSTRUMENT_ID}:transitory:decima-primera"),
            instrument_id: LRITF_INSTRUMENT_ID.to_owned(),
            provision_type: ProvisionType::Transitory,
            label: "DÉCIMA PRIMERA".to_owned(),
            number: "DÉCIMA PRIMERA".to_owned(),
            heading_context: HeadingContext {
                title: None,
                chapter: None,
            },
            text: "Texto.".to_owned(),
            publication_date: chrono::NaiveDate::from_ymd_opt(2018, 3, 9).unwrap(),
            effective_from: None,
            effective_to: None,
            temporal_status: TemporalStatus::Unknown,
            temporal_basis: None,
            temporal_confidence: None,
            review_status: ReviewStatus::NotAnalyzed,
        }
    }

    fn sample_instrument() -> Instrument {
        Instrument {
            schema_version: SCHEMA_VERSION.to_owned(),
            id: LRITF_INSTRUMENT_ID.to_owned(),
            jurisdiction: "mx".to_owned(),
            level: "federal".to_owned(),
            instrument_type: InstrumentType::Statute,
            official_title: "Ley para Regular las Instituciones de Tecnología Financiera"
                .to_owned(),
            short_name: "LRITF".to_owned(),
            operational_source: "camara_de_diputados".to_owned(),
            formal_publication_source: "dof".to_owned(),
            publication_date: chrono::NaiveDate::from_ymd_opt(2018, 3, 9).unwrap(),
            latest_reform_date: chrono::NaiveDate::from_ymd_opt(2025, 11, 14),
            retrieved_at: chrono::DateTime::parse_from_rfc3339("2026-06-28T00:00:00Z")
                .unwrap()
                .to_utc(),
            source_url: "https://www.diputados.gob.mx/LeyesBiblio/pdf/LRITF.pdf"
                .parse()
                .unwrap(),
            source_sha256: "d6f645e6a7d3c2eeb46905d4d24ecd8e078907057dc034cda715abf019ce8491"
                .to_owned(),
            extracted_text_sha256:
                "429a8916f3b1aa7035c0b700e27cd132a3af1662b1661ac703b9b0c7847b25a6".to_owned(),
            parser_version: "0.1.0".to_owned(),
            status: InstrumentStatus::InForce,
        }
    }
}
