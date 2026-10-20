use std::{collections::HashMap, fmt::Write as _, fs, path::Path};

use anyhow::{Context, Result};
use lex_core::{
    Corpus, Provision, ProvisionType, ReferenceForm, ReferenceResolutionStatus, ReviewItem,
    ReviewItemStatus, TemporalBoundary, TemporalBoundaryType, ValidationReport,
};

/// Presentation location of a link target, possibly in another instrument.
#[derive(Debug, Clone)]
pub struct LinkTarget {
    /// Canonical instrument identity: decides local versus cross-instrument
    /// Markdown paths. The display short name never does.
    pub instrument_id: String,
    pub instrument_short_name: String,
    /// Corpus directory slug used for standard-Markdown relative paths.
    pub instrument_slug: String,
    pub filename: String,
    /// Anchor keys of the fraction paragraphs the target provision has
    /// (`ii`, `v-bis`), each carrying a `^f-<key>` block anchor in its
    /// generated note.
    pub fractions: std::collections::HashSet<String>,
    /// Every block anchor actually emitted in the target's generated note
    /// (`f-ii`, `t-cuenta`), without the `^`. A repeated fraction numeral
    /// emits no anchor, so a link can never name an ambiguous or absent one.
    pub anchors: std::collections::HashSet<String>,
}

/// Canonical provision ID to presentation location, across every loaded
/// instrument.
pub type LinkTargets = HashMap<String, LinkTarget>;

/// Presentation location of a defined term's definition entry: the glossary
/// note plus the block anchor of the definition itself.
#[derive(Debug, Clone)]
pub struct TermTarget {
    pub note: LinkTarget,
    /// Block anchor without the `#^` prefix: `f-ii` for a fraction-style
    /// definition, `t-<slug>` for a colon-style one. `None` when the note
    /// emits no such unique anchor, so the link targets the whole note.
    pub anchor: Option<String>,
}

/// Defined-term ID to definition location, across every loaded instrument.
pub type TermTargets = HashMap<String, TermTarget>;

/// Build the link-target lookup for a set of loaded corpora, given each
/// corpus directory slug.
///
/// # Panics
///
/// Never in practice: the only panic is an `expect` on a static, valid
/// regular expression.
#[must_use]
pub fn link_targets(corpora: &[(&Corpus, &str)]) -> LinkTargets {
    let mut targets = LinkTargets::new();
    for (corpus, slug) in corpora {
        for provision in &corpus.provisions {
            let anchors: std::collections::HashSet<String> = paragraph_anchors(corpus, provision)
                .into_iter()
                .flatten()
                .collect();
            let fractions = anchors
                .iter()
                .filter_map(|anchor| anchor.strip_prefix("f-").map(str::to_owned))
                .collect();
            targets.insert(
                provision.id.clone(),
                LinkTarget {
                    instrument_id: corpus.instrument.id.clone(),
                    instrument_short_name: corpus.instrument.short_name.clone(),
                    instrument_slug: (*slug).to_owned(),
                    filename: markdown_filename(provision),
                    fractions,
                    anchors,
                },
            );
        }
    }
    targets
}

/// Build the defined-term target lookup for a set of loaded corpora.
#[must_use]
pub fn term_targets(corpora: &[(&Corpus, &str)], targets: &LinkTargets) -> TermTargets {
    let mut output = TermTargets::new();
    for (corpus, _) in corpora {
        for term in &corpus.terms {
            let Some(note) = targets.get(&term.defining_provision_id) else {
                continue;
            };
            // An inciso that shares its paragraph with another (LIEPS VIII
            // `…tabaco. c) Otros tabacos labrados…`) has no anchor of its
            // own, so it links to the paragraph that holds it.
            let anchor = Some(term_anchor(term))
                .filter(|anchor| note.anchors.contains(anchor))
                .or_else(|| {
                    term.inciso
                        .as_ref()
                        .and_then(|_| containing_paragraph_anchor(corpus, term))
                });
            output.insert(
                term.id.clone(),
                TermTarget {
                    note: note.clone(),
                    anchor,
                },
            );
        }
    }
    output
}

/// A roman numeral inside a qualifier phrase, with its character offset
/// relative to the phrase start.
struct QualifierNumeral<'a> {
    text: &'a str,
    offset_chars: usize,
}

fn roman_numerals(text: &str) -> Vec<QualifierNumeral<'_>> {
    let roman_re = regex::Regex::new(&format!(
        r"\b[IVXLCDM]+(?:\s+{})?\b",
        lex_core::FRACTION_QUALIFIER_PATTERN
    ))
    .expect("static regex");
    roman_re
        .find_iter(text)
        .map(|item| QualifierNumeral {
            text: item.as_str(),
            offset_chars: text[..item.start()].chars().count(),
        })
        .collect()
}

/// The block-anchor key of a fraction label: `V` gives `v`, and `V Bis` or
/// `V BIS` gives `v-bis`.
fn fraction_key(label: &str) -> String {
    lex_core::normalize_fraction_label(label)
        .to_lowercase()
        .replace(' ', "-")
}

/// The emitted anchor of the paragraph that holds a term's definition.
fn containing_paragraph_anchor(corpus: &Corpus, term: &lex_core::DefinedTerm) -> Option<String> {
    let provision = corpus
        .provisions
        .iter()
        .find(|provision| provision.id == term.defining_provision_id)?;
    let mut offset = 0;
    for (paragraph, anchor) in provision
        .text
        .split("\n\n")
        .zip(paragraph_anchors(corpus, provision))
    {
        let end = offset + paragraph.chars().count();
        if (offset..end).contains(&term.start_char) {
            return anchor;
        }
        offset = end + 2;
    }
    None
}

fn term_anchor(term: &lex_core::DefinedTerm) -> String {
    if let (Some(fraction), Some(inciso)) = (&term.fraction, &term.inciso) {
        format!("f-{}-{inciso}", fraction_key(fraction))
    } else if let Some(fraction) = &term.fraction {
        format!("f-{}", fraction_key(fraction))
    } else {
        let slug = term.id.rsplit(":term:").next().unwrap_or("term");
        format!("t-{slug}")
    }
}

fn append_outdated_reference_notices(output: &mut String, corpus: &Corpus, source_id: &str) {
    for edge in &corpus.references {
        if edge.source_provision_id == source_id
            && edge.resolution_status
                == ReferenceResolutionStatus::HistoricalSupersededReferenceOutdated
            && let Some(approval) = &edge.outdated_reference_approval
        {
            let _ = write!(
                output,
                "> Referencia histórica desactualizada: «{}» → `{}`. Se conserva la cita sin enlace vigente. {}\n\n",
                edge.source_span, edge.target_provision_id, approval.rationale
            );
        }
    }
}

fn append_source_repeal_notice(output: &mut String, corpus: &Corpus) {
    if let Some(notice) = &corpus.instrument.source_repeal_notice {
        if let Some(clause) = &notice.successor_clause {
            let _ = write!(
                output,
                "> **Instrumento abrogado — texto histórico.** «{}» (`{}`, instrumento sucesor `{}`).\n> Fuente formal: [Diario Oficial de la Federación]({}). No se infieren fechas ni efectos por disposición.\n\n",
                notice.source_note,
                clause.provision_id,
                notice
                    .successor_instrument_id
                    .as_deref()
                    .unwrap_or_default(),
                notice.formal_publication_url
            );
            return;
        }
        let _ = write!(
            output,
            "> **Ley abrogada — texto histórico.** {}\n> Fuente formal: [Diario Oficial de la Federación]({}). No se infieren fechas ni efectos por disposición.\n\n",
            notice.source_note, notice.formal_publication_url
        );
    }
}

fn evidence_kind_label(kind: &lex_core::EnactmentEvidenceKind) -> &'static str {
    match kind {
        lex_core::EnactmentEvidenceKind::Transitory => "Transitorio del decreto",
        lex_core::EnactmentEvidenceKind::ContainingArticle => "Artículo del acto contenedor",
        lex_core::EnactmentEvidenceKind::ElidedSiblingArticle => {
            "Artículo omitido en la compilación"
        }
    }
}

fn enactment_markdown(corpus: &Corpus) -> String {
    let neutral = corpus
        .enactment_evidence
        .iter()
        .any(|e| e.owner == lex_core::EnactmentOwner::ContainingAct);
    let mut output = String::from(if neutral {
        "# Evidencia del acto contenedor\n\nEstos textos pertenecen al acto contenedor, no a las disposiciones propias del instrumento. Se conserva el título formal del acto; no contienen una determinación temporal.\n\n"
    } else {
        "# Evidencia del decreto expedidor\n\nEstos textos pertenecen al decreto, no a los transitorios propios del instrumento. No contienen una determinación temporal.\n\n"
    });
    for item in &corpus.enactment_evidence {
        let _ = write!(
            output,
            "## {} — {}\n\nID: {}\n\nPropietario: {}. Tipo: {}.\n\n{}\n\n",
            item.number,
            item.source_heading,
            item.id,
            if item.owner == lex_core::EnactmentOwner::ContainingAct {
                "acto contenedor"
            } else {
                "decreto expedidor"
            },
            if item.owner == lex_core::EnactmentOwner::ContainingAct
                && item.kind == lex_core::EnactmentEvidenceKind::Transitory
            {
                "Transitorio del acto contenedor"
            } else {
                evidence_kind_label(&item.kind)
            },
            item.text
        );
    }
    output
}

fn write_enactment_presentation(corpus: &Corpus, directory: &Path) -> Result<Vec<String>> {
    if corpus.enactment_evidence.is_empty() {
        for name in ["decreto-expedicion.md", "enactment-evidence.json"] {
            let path = directory.join(name);
            if path.is_file() {
                fs::remove_file(path)?;
            }
        }
        return Ok(Vec::new());
    }
    fs::write(
        directory.join("decreto-expedicion.md"),
        enactment_markdown(corpus),
    )?;
    write_json(
        &corpus.enactment_evidence,
        &directory.join("enactment-evidence.json"),
    )?;
    Ok(vec![
        "decreto-expedicion.md".to_owned(),
        "enactment-evidence.json".to_owned(),
    ])
}

pub fn write_canonical(corpus: &Corpus, output_dir: &Path) -> Result<()> {
    fs::create_dir_all(output_dir)?;
    write_json(&corpus.instrument, &output_dir.join("instrument.json"))?;
    write_json(&corpus.provisions, &output_dir.join("provisions.json"))?;
    write_json(&corpus.references, &output_dir.join("references.json"))?;
    write_json(&corpus.terms, &output_dir.join("terms.json"))?;
    write_json(&corpus.term_usages, &output_dir.join("term-usages.json"))?;
    let evidence_path = output_dir.join("enactment-evidence.json");
    if !corpus.enactment_evidence.is_empty() || evidence_path.exists() {
        write_json(&corpus.enactment_evidence, &evidence_path)?;
    }
    // The REFERENCIAS legend only exists for compiled documents with
    // margin markers; skip the file entirely for instruments without one.
    if corpus.amendment_references.is_empty() {
        Ok(())
    } else {
        write_json(
            &corpus.amendment_references,
            &output_dir.join("amendment-references.json"),
        )
    }
}

pub fn write_validation(report: &ValidationReport, output_dir: &Path) -> Result<()> {
    fs::create_dir_all(output_dir)?;
    write_json(report, &output_dir.join("validation.json"))
}

pub fn write_markdown(
    corpus: &Corpus,
    targets: &LinkTargets,
    terms: &TermTargets,
    output_dir: &Path,
) -> Result<()> {
    fs::create_dir_all(output_dir)?;
    let mut generated_files: std::collections::HashSet<String> =
        corpus.provisions.iter().map(markdown_filename).collect();
    generated_files.insert("README.md".to_owned());
    generated_files.extend(write_enactment_presentation(corpus, output_dir)?);
    // This directory is a generated presentation target. Remove Markdown
    // files no longer produced by the current canonical corpus so a parser
    // correction cannot leave stale, contradictory provision notes behind.
    for entry in fs::read_dir(output_dir)? {
        let entry = entry?;
        let path = entry.path();
        let filename = entry.file_name();
        if entry.file_type()?.is_file()
            && path.extension().is_some_and(|extension| extension == "md")
            && filename
                .to_str()
                .is_some_and(|name| !generated_files.contains(name))
        {
            fs::remove_file(path)?;
        }
    }
    for provision in &corpus.provisions {
        let filename = markdown_filename(provision);
        let content = standard_markdown(corpus, targets, terms, provision);
        fs::write(output_dir.join(filename), content)?;
    }
    fs::write(
        output_dir.join("README.md"),
        markdown_index(corpus, targets, terms, false),
    )?;
    Ok(())
}

pub fn write_obsidian(
    corpus: &Corpus,
    targets: &LinkTargets,
    terms: &TermTargets,
    review_items: &[ReviewItem],
    output_dir: &Path,
) -> Result<()> {
    let instrument_dir = output_dir
        .join("Corpus")
        .join(&corpus.instrument.short_name);
    // With many instruments, two adapters mis-sharing a short_name would
    // silently overwrite each other's vault folder; refuse when the
    // folder's export manifest names a different instrument.
    let manifest_path = instrument_dir.join("_lex-mex-export.json");
    if manifest_path.exists() {
        let existing: serde_json::Value = serde_json::from_slice(&fs::read(&manifest_path)?)?;
        if let Some(existing_id) = existing.get("instrument_id").and_then(|id| id.as_str())
            && existing_id != corpus.instrument.id
        {
            anyhow::bail!(
                "Corpus/{} already belongs to {existing_id}; refusing to overwrite it with {}",
                corpus.instrument.short_name,
                corpus.instrument.id
            );
        }
    }
    fs::create_dir_all(&instrument_dir)?;
    let mut generated_files = Vec::with_capacity(corpus.provisions.len() + 1);
    generated_files.extend(write_enactment_presentation(corpus, &instrument_dir)?);
    for provision in &corpus.provisions {
        let filename = markdown_filename(provision);
        let content = obsidian_markdown(corpus, targets, terms, provision);
        fs::write(instrument_dir.join(&filename), content)?;
        generated_files.push(filename);
    }
    let index_filename = format!("{}.md", corpus.instrument.short_name);
    fs::write(
        instrument_dir.join(&index_filename),
        obsidian_index(corpus, targets, terms),
    )?;
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

/// A one-line JSON front-matter entry, omitted for an empty list.
fn record_front_matter<T: serde::Serialize>(key: &str, value: &T) -> String {
    let json = serde_json::to_value(value).expect("serializing a record cannot fail");
    if json.as_array().is_some_and(Vec::is_empty) {
        return String::new();
    }
    format!("{key}: {json}\n")
}

fn front_matter(corpus: &Corpus, provision: &Provision) -> String {
    let alias = format!("{} — {}", corpus.instrument.short_name, provision.label);
    let effect_types: Vec<_> = provision
        .transitory_effects
        .iter()
        .map(|effect| json_name(&effect.effect_type))
        .collect();
    let effect_front_matter = if effect_types.is_empty() {
        String::new()
    } else {
        format!(
            "transitory_effects: {}\n",
            serde_json::to_string(&effect_types).expect("serializing effect types cannot fail")
        )
    };
    let marks_front_matter = if provision.amendment_marks.is_empty() {
        String::new()
    } else {
        format!(
            "amendment_marks: {}\n",
            serde_json::to_string(&provision.amendment_marks)
                .expect("serializing marker numbers cannot fail")
        )
    };
    let marks_front_matter = format!(
        "{marks_front_matter}{}{}",
        record_front_matter("repeals", &provision.repeals),
        provision
            .commencement_condition
            .as_ref()
            .map_or_else(String::new, |condition| {
                record_front_matter("commencement_condition", condition)
            }),
    );
    let marks_front_matter =
        format!(
            "{marks_front_matter}{}{}",
            provision
                .numbering_basis
                .as_ref()
                .map_or_else(String::new, |v| record_front_matter("numbering_basis", v)),
            provision.publication_period.as_ref().map_or_else(
                String::new,
                |v| record_front_matter("publication_period", v)
            )
        );
    format!(
        "---\nid: {}\ninstrument_id: {}\ninstrument: {}\nname: {}\nprovision_type: {}\nnumber: \"{}\"\naliases: [{}]\ngenerated: true\ntemporal_status: {}\nreview_status: {}\n{}{}source_url: {}\nsource_sha256: {}\n---\n\n",
        provision.id,
        provision.instrument_id,
        corpus.instrument.short_name,
        serde_json::to_string(&corpus.instrument.official_title)
            .expect("serializing title cannot fail"),
        json_name(&provision.provision_type),
        provision.number,
        serde_json::to_string(&alias).expect("serializing alias cannot fail"),
        json_name(&provision.temporal_status),
        json_name(&provision.review_status),
        effect_front_matter,
        marks_front_matter,
        corpus.instrument.source_url,
        corpus.instrument.source_sha256,
    )
}

fn standard_markdown(
    corpus: &Corpus,
    targets: &LinkTargets,
    terms: &TermTargets,
    provision: &Provision,
) -> String {
    let mut output = front_matter(corpus, provision);
    append_source_repeal_notice(&mut output, corpus);
    append_outdated_reference_notices(&mut output, corpus, &provision.id);
    if let Some(title) = &provision.heading_context.title {
        let _ = write!(output, "> {title}");
        if let Some(chapter) = &provision.heading_context.chapter {
            let _ = write!(output, " · {chapter}");
        }
        output.push_str("\n\n");
    } else if let Some(chapter) = &provision.heading_context.chapter {
        let _ = write!(output, "> {chapter}");
        if let Some(section) = &provision.heading_context.section {
            let _ = write!(output, " · {section}");
        }
        if let Some(apartado) = &provision.heading_context.apartado {
            let _ = write!(output, " · {apartado}");
        }
        output.push_str("\n\n");
    }
    let _ = write!(
        output,
        "# {}\n\n{}\n",
        provision.label,
        linked_string(
            corpus,
            targets,
            terms,
            &provision.text,
            &provision.id,
            false
        )
    );
    if let Some(period) = &provision.publication_period {
        let _ = write!(
            output,
            "\nPublicación seriada, sin día exacto: {} a {} (inclusive).\n",
            period.start, period.end
        );
    }
    append_transitory_effects(&mut output, provision);
    output
}

fn obsidian_markdown(
    corpus: &Corpus,
    targets: &LinkTargets,
    terms: &TermTargets,
    provision: &Provision,
) -> String {
    let mut output = front_matter(corpus, provision);
    append_source_repeal_notice(&mut output, corpus);
    append_outdated_reference_notices(&mut output, corpus, &provision.id);
    let _ = write!(
        output,
        "[[Corpus/{0}/{0}|← Índice {0}]]\n\n",
        corpus.instrument.short_name
    );
    let body = linked_string(corpus, targets, terms, &provision.text, &provision.id, true);
    let body = with_block_anchors(corpus, provision, &body);
    let _ = write!(output, "# {}\n\n{body}\n", provision.label);
    if let Some(period) = &provision.publication_period {
        let _ = write!(
            output,
            "\nPublicación seriada, sin día exacto: {} a {} (inclusive).\n",
            period.start, period.end
        );
    }
    append_transitory_effects(&mut output, provision);
    output
}

/// Append Obsidian block anchors to the rendered paragraphs of a provision:
/// `^f-<roman>` on fraction paragraphs (hovering a fraction link previews
/// only that fraction) and `^t-<slug>` on colon-style definition entries.
/// Anchors are decided from the canonical text's paragraphs; the rendered
/// body has identical paragraph structure because link injection never
/// crosses paragraph boundaries.
fn with_block_anchors(corpus: &Corpus, provision: &Provision, body: &str) -> String {
    let anchors = paragraph_anchors(corpus, provision);
    let rendered: Vec<&str> = body.split("\n\n").collect();
    if rendered.len() != anchors.len() {
        return body.to_owned();
    }
    rendered
        .iter()
        .zip(anchors)
        .map(|(paragraph, anchor)| match anchor {
            Some(anchor) => format!("{paragraph} ^{anchor}"),
            None => (*paragraph).to_owned(),
        })
        .collect::<Vec<_>>()
        .join("\n\n")
}

/// The one registry of block anchors a provision's note actually emits, per
/// canonical paragraph: `f-<roman>` on fraction paragraphs, `t-<slug>` on
/// definition entries that start a paragraph, and `f-<roman>-<letter>` on a
/// defining inciso that starts one. An anchor that more than one
/// paragraph would carry (a fraction numeral repeated under different
/// apartados) is not emitted at all, because a link to it could not identify
/// one target. Links consult this registry and fall back to the whole note.
fn paragraph_anchors(corpus: &Corpus, provision: &Provision) -> Vec<Option<String>> {
    let fraction_re = regex::Regex::new(&format!(
        r"^([IVXLCDM]+(?:\.?\s+{})?)\.\s",
        lex_core::FRACTION_QUALIFIER_PATTERN
    ))
    .expect("static regex");
    let mut anchors_by_offset: HashMap<usize, String> = HashMap::new();
    for term in &corpus.terms {
        if term.defining_provision_id == provision.id
            && (term.fraction.is_none() || term.inciso.is_some())
        {
            anchors_by_offset.insert(term.start_char, term_anchor(term));
        }
    }

    let mut offset = 0;
    let mut anchors = Vec::new();
    for paragraph in provision.text.split("\n\n") {
        let anchor = anchors_by_offset.get(&offset).cloned().or_else(|| {
            fraction_re
                .captures(paragraph)
                .map(|captures| format!("f-{}", fraction_key(&captures[1])))
        });
        anchors.push(anchor);
        offset += paragraph.chars().count() + 2;
    }
    let mut counts: HashMap<String, usize> = HashMap::new();
    for anchor in anchors.iter().flatten() {
        *counts.entry(anchor.clone()).or_default() += 1;
    }
    anchors
        .into_iter()
        .map(|anchor| anchor.filter(|anchor| counts.get(anchor) == Some(&1)))
        .collect()
}

/// Build the reference-edge injections for one provision (or the
/// instrument title): whole-note links for article citations, fraction
/// block-anchor links for anchored fraction qualifiers, and self-anchor
/// links for `fracción N del presente artículo` edges.
fn reference_injections<'a>(
    corpus: &'a Corpus,
    targets: &'a LinkTargets,
    source_id: &str,
    obsidian: bool,
) -> Vec<Injection<'a>> {
    let mut injections: Vec<Injection<'a>> = Vec::new();
    // Range expansions are excluded: the expanded middle articles share
    // one span with the range endpoints, which already link directly.
    for edge in corpus.references.iter().filter(|edge| {
        edge.source_provision_id == source_id
            && edge.resolution_status == ReferenceResolutionStatus::Resolved
            && matches!(
                edge.reference_form,
                ReferenceForm::Direct | ReferenceForm::Relative
            )
    }) {
        let Some(target) = targets.get(&edge.target_provision_id) else {
            continue;
        };
        if edge.target_provision_id == source_id {
            // A `fracción N del presente artículo` edge: the numeral links
            // to the provision's own fraction block. Meaningless without
            // block anchors, so Obsidian only.
            let numeral = fraction_key(&edge.source_span);
            if obsidian && target.fractions.contains(&numeral) {
                injections.push(Injection {
                    start_char: edge.start_char,
                    end_char: edge.end_char,
                    expected_span: &edge.source_span,
                    target,
                    anchor: Some(format!("f-{numeral}")),
                });
            }
            continue;
        }
        injections.push(Injection {
            start_char: edge.start_char,
            end_char: edge.end_char,
            expected_span: &edge.source_span,
            target,
            anchor: None,
        });
        // Fraction qualifiers with anchored spans additionally link each
        // numeral to the target's fraction block (`la fracción XI del
        // artículo 36` — hovering `XI` previews only that fraction).
        if !obsidian {
            continue;
        }
        for qualifier in &edge.qualifiers {
            if qualifier.qualifier_type != lex_core::ReferenceQualifierType::Fraction {
                continue;
            }
            let (Some(start), Some(_)) = (qualifier.start_char, qualifier.end_char) else {
                continue;
            };
            for numeral in roman_numerals(&qualifier.text) {
                let lower = fraction_key(numeral.text);
                if !target.fractions.contains(&lower) {
                    continue;
                }
                injections.push(Injection {
                    start_char: start + numeral.offset_chars,
                    end_char: start + numeral.offset_chars + numeral.text.chars().count(),
                    expected_span: numeral.text,
                    target,
                    anchor: Some(format!("f-{lower}")),
                });
            }
        }
    }

    injections
}

/// One planned link inside a provision's rendered text: a resolved
/// reference edge or a defined-term usage.
struct Injection<'a> {
    start_char: usize,
    end_char: usize,
    expected_span: &'a str,
    target: &'a LinkTarget,
    /// Obsidian block anchor inside the target note (`f-ii`, `t-cuenta`),
    /// used for defined-term definitions.
    anchor: Option<String>,
}

/// Inject resolved direct reference links and defined-term links into
/// `text`, whose reference edges are anchored at `source_id` (a provision
/// ID, or the instrument ID for the official title). Canonical text is
/// never modified; links exist only in the returned presentation string.
/// Term links target the definition's block anchor, so hovering shows only
/// the definition entry; to keep notes readable, only the first usage of
/// each term per provision becomes a link, and usages overlapping a
/// reference span are skipped.
fn linked_string(
    corpus: &Corpus,
    targets: &LinkTargets,
    terms: &TermTargets,
    text: &str,
    source_id: &str,
    obsidian: bool,
) -> String {
    let mut injections = reference_injections(corpus, targets, source_id, obsidian);

    let mut linked_terms = std::collections::HashSet::new();
    let mut usages: Vec<_> = corpus
        .term_usages
        .iter()
        .filter(|usage| usage.provision_id == source_id)
        .collect();
    usages.sort_by_key(|usage| usage.start_char);
    for usage in usages {
        if !linked_terms.insert(&usage.term_id) {
            continue;
        }
        let Some(term_target) = terms.get(&usage.term_id) else {
            continue;
        };
        let overlaps = injections.iter().any(|existing| {
            usage.start_char < existing.end_char && existing.start_char < usage.end_char
        });
        if overlaps {
            continue;
        }
        injections.push(Injection {
            start_char: usage.start_char,
            end_char: usage.end_char,
            expected_span: &usage.span,
            target: &term_target.note,
            anchor: term_target.anchor.clone(),
        });
    }
    injections.sort_by_key(|injection| (injection.start_char, injection.end_char));

    let chars: Vec<_> = text.chars().collect();
    let mut output = String::new();
    let mut cursor = 0;
    for injection in injections {
        if injection.start_char < cursor
            || injection.end_char > chars.len()
            || injection.start_char >= injection.end_char
        {
            continue;
        }
        let displayed: String = chars[injection.start_char..injection.end_char]
            .iter()
            .collect();
        if displayed != injection.expected_span {
            continue;
        }
        output.extend(chars[cursor..injection.start_char].iter());
        let target = injection.target;
        let stem = target.filename.trim_end_matches(".md");
        if obsidian {
            let anchor = injection
                .anchor
                .as_ref()
                .map_or_else(String::new, |anchor| format!("#^{anchor}"));
            let _ = write!(
                output,
                "[[Corpus/{}/{stem}{anchor}|{displayed}]]",
                target.instrument_short_name
            );
        } else if target.instrument_id == corpus.instrument.id {
            let _ = write!(output, "[{displayed}]({stem}.md)");
        } else {
            let _ = write!(
                output,
                "[{displayed}](../../{}/markdown/{stem}.md)",
                target.instrument_slug
            );
        }
        cursor = injection.end_char;
    }
    output.extend(chars[cursor..].iter());
    output
}

fn append_transitory_effects(output: &mut String, provision: &Provision) {
    if provision.transitory_effects.is_empty() {
        return;
    }
    output.push_str("\n## Efectos transitorios estructurados\n\n");
    for (index, effect) in provision.transitory_effects.iter().enumerate() {
        let authorities = if effect.responsible_authorities.is_empty() {
            "Ninguna identificada".to_owned()
        } else {
            effect.responsible_authorities.join("; ")
        };
        let _ = write!(
            output,
            "### Efecto {} — {}\n\n\
             - **Alcance afectado:** {}\n\
             - **Regla de aplicación:** {}\n\
             - **Detonante:** {}\n\
             - **Condición de terminación:** {}\n\
             - **Autoridades responsables:** {}\n\
             - **Verificación:** {}\n",
            index + 1,
            json_name(&effect.effect_type),
            effect.affected_scope,
            json_name(&effect.application_rule),
            format_boundary(&effect.trigger),
            format_boundary(&effect.end_condition),
            authorities,
            json_name(&effect.verification_status),
        );
        if let Some(source_url) = &effect.verification_source_url {
            let _ = writeln!(output, "- **Fuente de verificación:** {source_url}");
        }
        if let Some(event_date) = effect.verified_event_date {
            let _ = writeln!(output, "- **Fecha del evento verificado:** {event_date}");
        }
        if let Some(note) = &effect.verification_note {
            let _ = writeln!(output, "- **Nota de verificación:** {note}");
        }
        output.push('\n');
    }
    if output.ends_with("\n\n") {
        output.pop();
    }
}

fn format_boundary(boundary: &TemporalBoundary) -> String {
    if boundary.boundary_type == TemporalBoundaryType::None {
        return "No aplica".to_owned();
    }
    let mut value = json_name(&boundary.boundary_type);
    if let Some(date) = boundary.date {
        let _ = write!(value, " ({date})");
    }
    if let Some(description) = &boundary.description {
        let _ = write!(value, ": {description}");
    }
    value
}

fn obsidian_index(corpus: &Corpus, targets: &LinkTargets, terms: &TermTargets) -> String {
    let mut output = format!(
        "---\nid: {}\naliases: [{}]\ngenerated: true\nsource_url: {}\nsource_sha256: {}\n---\n\n[[Inicio|← Inicio]]\n\n",
        corpus.instrument.id,
        serde_json::to_string(&corpus.instrument.official_title)
            .expect("serializing alias cannot fail"),
        corpus.instrument.source_url,
        corpus.instrument.source_sha256,
    );
    output.push_str(&markdown_index(corpus, targets, terms, true));
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
        let formal_source = item.formal_source_url.as_ref().map_or_else(
            || "No disponible".to_owned(),
            |url| format!("[Diario Oficial de la Federación]({url})"),
        );
        let provision_diff = item.provision_diff.as_deref().unwrap_or("No disponible");
        let _ = write!(
            output,
            "## {}\n\n\
             - **ID:** {}\n\
             - **Conclusión propuesta:** {}\n\
             - **Confianza:** {:.2}\n\
             - **Problema:** {}\n\
             - **Fuente operativa:** [Cámara de Diputados]({})\n\
             - **Fuente formal:** {}\n\
             - **Diferencia:** {}\n\n\
             **Texto relevante**\n\n> {}\n\n",
            item.evidence.label,
            item.provision_id,
            json_name(&determination.temporal_status),
            determination.confidence,
            item.exact_issue,
            item.camara_source_url,
            formal_source,
            provision_diff,
            item.evidence.text.replace('\n', "\n> "),
        );
        output.push_str("**Efectos propuestos**\n\n");
        for effect in &determination.effects {
            let _ = writeln!(
                output,
                "- **{}:** {}. Regla: `{}`. Detonante: {}. Terminación: {}. \
                 Verificación: `{}`.",
                json_name(&effect.effect_type),
                effect.affected_scope,
                json_name(&effect.application_rule),
                format_boundary(&effect.trigger),
                format_boundary(&effect.end_condition),
                json_name(&effect.verification_status),
            );
        }
        output.push('\n');
    }
    output
}

fn append_enactment_index(output: &mut String, corpus: &Corpus) {
    if !corpus.enactment_evidence.is_empty() {
        let neutral = corpus
            .enactment_evidence
            .iter()
            .any(|e| e.owner == lex_core::EnactmentOwner::ContainingAct);
        output.push_str(if neutral {
            "\n## Evidencia del acto contenedor (no son disposiciones propias)\n\n"
        } else {
            "\n## Evidencia del decreto expedidor (no son transitorios propios)\n\n"
        });
        for item in &corpus.enactment_evidence {
            let _ = writeln!(
                output,
                "- {} — {} ({}): {}",
                item.number,
                evidence_kind_label(&item.kind),
                if neutral {
                    "acto contenedor"
                } else {
                    "decreto"
                },
                item.source_heading
            );
        }
        output.push_str("\nVéase enactment-evidence.json y decreto-expedicion.md.\n");
    }
}

fn markdown_index(
    corpus: &Corpus,
    targets: &LinkTargets,
    terms: &TermTargets,
    obsidian: bool,
) -> String {
    let linked_title = linked_string(
        corpus,
        targets,
        terms,
        &corpus.instrument.official_title,
        &corpus.instrument.id,
        obsidian,
    );
    let mut output = format!(
        "# {}\n\nFuente operativa: [{}]({})\n\n",
        linked_title, corpus.instrument.operational_source, corpus.instrument.source_url
    );
    append_source_repeal_notice(&mut output, corpus);
    if let Some(formal_url) = &corpus.instrument.formal_publication_url {
        let code = corpus
            .instrument
            .formal_publication_code
            .as_deref()
            .unwrap_or("dof");
        let _ = write!(
            output,
            "Fuente formal: [Diario Oficial de la Federación {code}]({formal_url})\n\n"
        );
    }
    if let Some(period) = &corpus.instrument.publication_period {
        let _ = write!(
            output,
            "Publicación seriada (sin día exacto): {} a {} (inclusive). {}\n\n",
            period.start, period.end, period.source_note
        );
    }
    output.push_str("## Artículos\n\n");
    for provision in corpus
        .provisions
        .iter()
        .filter(|item| item.provision_type == ProvisionType::Article)
    {
        output.push_str(&index_link(
            provision,
            &corpus.instrument.short_name,
            obsidian,
        ));
    }
    output.push_str("\n## Disposiciones transitorias\n\n");
    for provision in corpus
        .provisions
        .iter()
        .filter(|item| item.provision_type == ProvisionType::Transitory)
    {
        output.push_str(&index_link(
            provision,
            &corpus.instrument.short_name,
            obsidian,
        ));
    }
    append_enactment_index(&mut output, corpus);
    let annexes: Vec<_> = corpus
        .provisions
        .iter()
        .filter(|item| item.provision_type == ProvisionType::Annex)
        .collect();
    if !annexes.is_empty() {
        output.push_str("\n## Anexos\n\n");
        for provision in annexes {
            output.push_str(&index_link(
                provision,
                &corpus.instrument.short_name,
                obsidian,
            ));
        }
    }
    if !corpus.amendment_references.is_empty() {
        output.push_str(
            "\n## Referencias de reforma\n\nLos números de margen `(N)` del documento compilado remiten a estas resoluciones:\n\n",
        );
        for reference in &corpus.amendment_references {
            let _ = writeln!(output, "{}. {}", reference.marker, reference.description);
        }
    }
    // An instrument may have no original transitories. Keep the empty
    // section visible, but do not emit a trailing blank line that fails the
    // repository whitespace gate.
    if output.ends_with("\n\n") {
        output.pop();
    }
    output
}

/// Index entry for one provision. Obsidian links use the full vault path:
/// with more than one instrument in the vault, bare note stems such as
/// `articulo-1` are ambiguous across instruments.
fn index_link(provision: &Provision, instrument_short_name: &str, obsidian: bool) -> String {
    let stem = markdown_filename(provision)
        .trim_end_matches(".md")
        .to_owned();
    if obsidian {
        format!(
            "- [[Corpus/{instrument_short_name}/{stem}|{}]]\n",
            provision.label
        )
    } else {
        format!("- [{}]({stem}.md)\n", provision.label)
    }
}

fn markdown_filename(provision: &Provision) -> String {
    match provision.provision_type {
        ProvisionType::Article => format!(
            "articulo-{}.md",
            provision.number.replace([' ', '/', '\\'], "-")
        ),
        ProvisionType::Annex => format!("anexo-{}.md", provision.number.replace(' ', "-")),
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
    #[test]
    fn record_front_matter_is_omitted_when_empty_and_one_line_otherwise() {
        let empty: Vec<lex_core::ProvisionRepeal> = Vec::new();
        assert_eq!(super::record_front_matter("repeals", &empty), "");
        let repeals = vec![lex_core::ProvisionRepeal {
            scope: lex_core::RepealScope::Paragraph,
            ordinals: vec!["1".to_owned()],
            cause: lex_core::RepealCause::Judicial,
            dof_date: None,
            renumbering: false,
        }];
        let line = super::record_front_matter("repeals", &repeals);
        assert!(line.starts_with("repeals: [{"));
        assert!(line.contains("\"cause\":\"judicial\""));
        assert_eq!(line.matches('\n').count(), 1);
    }

    use std::fs;

    use lex_core::{
        Basis, Corpus, HeadingContext, Instrument, InstrumentStatus, InstrumentType,
        LRITF_INSTRUMENT_ID, Provision, ProvisionType, ReferenceEdge, ReferenceForm,
        ReferenceQualifier, ReferenceQualifierType, ReferenceResolutionStatus, ReviewStatus,
        SCHEMA_VERSION, TemporalStatus,
    };
    use tempfile::tempdir;

    use super::{link_targets, markdown_filename, term_targets, write_markdown, write_obsidian};

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
            enactment_evidence: Vec::new(),
            instrument: sample_instrument(),
            provisions: vec![sample_provision()],
            references: Vec::new(),
            terms: Vec::new(),
            term_usages: Vec::new(),
            amendment_references: Vec::new(),
        };
        let targets = link_targets(&[(&corpus, "lritf")]);
        let terms = term_targets(&[(&corpus, "lritf")], &targets);

        write_obsidian(&corpus, &targets, &terms, &[], temp.path()).unwrap();

        assert_eq!(
            fs::read_to_string(notes.join("criterio.md")).unwrap(),
            "Conservar."
        );
        assert!(
            temp.path()
                .join("Corpus/LRITF/transitorio-decima-primera.md")
                .is_file()
        );
        assert!(
            fs::read_to_string(
                temp.path()
                    .join("Corpus/LRITF/transitorio-decima-primera.md")
            )
            .unwrap()
            .contains("## Efectos transitorios estructurados")
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

    #[test]
    fn injects_resolved_links_without_changing_canonical_text() {
        let temp = tempdir().unwrap();
        let mut source = sample_provision();
        source.text = "Véanse los artículos 48, segundo párrafo y 54 de esta Ley.".to_owned();
        let target_48 = sample_article("48");
        let target_54 = sample_article("54");
        let start_48 = source
            .text
            .chars()
            .position(|character| character == '4')
            .unwrap();
        let start_54 = source
            .text
            .chars()
            .collect::<Vec<_>>()
            .iter()
            .rposition(|character| *character == '5')
            .unwrap();
        let canonical_text = source.text.clone();
        let corpus = Corpus {
            enactment_evidence: Vec::new(),
            instrument: sample_instrument(),
            provisions: vec![source, target_48, target_54],
            references: vec![
                sample_reference(
                    "48",
                    start_48,
                    vec![ReferenceQualifier {
                        qualifier_type: ReferenceQualifierType::Paragraph,
                        text: "segundo párrafo".to_owned(),
                        start_char: None,
                        end_char: None,
                    }],
                ),
                sample_reference("54", start_54, Vec::new()),
            ],
            terms: Vec::new(),
            term_usages: Vec::new(),
            amendment_references: Vec::new(),
        };

        let targets = link_targets(&[(&corpus, "lritf")]);
        let terms = term_targets(&[(&corpus, "lritf")], &targets);
        write_markdown(&corpus, &targets, &terms, &temp.path().join("markdown")).unwrap();
        write_obsidian(&corpus, &targets, &terms, &[], temp.path()).unwrap();

        let standard =
            fs::read_to_string(temp.path().join("markdown/transitorio-decima-primera.md")).unwrap();
        let obsidian = fs::read_to_string(
            temp.path()
                .join("Corpus/LRITF/transitorio-decima-primera.md"),
        )
        .unwrap();
        assert!(standard.contains("artículos [48](articulo-48.md), segundo párrafo"));
        assert!(obsidian.contains("[[Corpus/LRITF/articulo-48|48]], segundo párrafo"));
        assert!(obsidian.contains("[[Corpus/LRITF/articulo-54|54]]"));
        assert_eq!(corpus.provisions[0].text, canonical_text);
    }

    #[test]
    fn standard_markdown_removes_stale_generated_notes() {
        let temp = tempdir().unwrap();
        let output = temp.path().join("markdown");
        fs::create_dir_all(&output).unwrap();
        fs::write(output.join("articulo-1-1.md"), "stale").unwrap();
        fs::write(output.join("inspection.txt"), "keep").unwrap();
        let corpus = Corpus {
            enactment_evidence: Vec::new(),
            instrument: sample_instrument(),
            provisions: vec![sample_provision()],
            references: Vec::new(),
            terms: Vec::new(),
            term_usages: Vec::new(),
            amendment_references: Vec::new(),
        };
        let targets = link_targets(&[(&corpus, "lritf")]);
        let terms = term_targets(&[(&corpus, "lritf")], &targets);

        write_markdown(&corpus, &targets, &terms, &output).unwrap();

        assert!(!output.join("articulo-1-1.md").exists());
        assert!(output.join("transitorio-decima-primera.md").is_file());
        assert!(output.join("README.md").is_file());
        assert_eq!(
            fs::read_to_string(output.join("inspection.txt")).unwrap(),
            "keep"
        );
    }

    #[test]
    fn index_without_transitories_has_no_trailing_blank_line() {
        let temp = tempdir().unwrap();
        let output = temp.path().join("markdown");
        let corpus = Corpus {
            enactment_evidence: Vec::new(),
            instrument: sample_instrument(),
            provisions: vec![sample_article("1")],
            references: Vec::new(),
            terms: Vec::new(),
            term_usages: Vec::new(),
            amendment_references: Vec::new(),
        };
        let targets = link_targets(&[(&corpus, "lritf")]);
        let terms = term_targets(&[(&corpus, "lritf")], &targets);

        write_markdown(&corpus, &targets, &terms, &output).unwrap();

        let index = fs::read_to_string(output.join("README.md")).unwrap();
        assert!(index.ends_with("## Disposiciones transitorias\n"));
        assert!(!index.ends_with("\n\n"));
    }

    fn sample_provision() -> Provision {
        Provision {
            numbering_basis: None,
            schema_version: SCHEMA_VERSION.to_owned(),
            id: format!("{LRITF_INSTRUMENT_ID}:transitory:decima-primera"),
            instrument_id: LRITF_INSTRUMENT_ID.to_owned(),
            provision_type: ProvisionType::Transitory,
            label: "DÉCIMA PRIMERA".to_owned(),
            number: "DÉCIMA PRIMERA".to_owned(),
            heading_context: HeadingContext {
                libro: None,
                title: None,
                chapter: None,
                section: None,
                apartado: None,
            },
            text: "Texto.".to_owned(),
            publication_date: Some(chrono::NaiveDate::from_ymd_opt(2018, 3, 9).unwrap()),
            publication_period: None,
            effective_from: None,
            effective_to: None,
            temporal_status: TemporalStatus::Unknown,
            temporal_basis: None,
            temporal_confidence: None,
            review_status: ReviewStatus::NotAnalyzed,
            transitory_effects: vec![lex_core::TransitoryEffect {
                effect_type: lex_core::TransitoryEffectType::ImplementationDeadline,
                affected_scope: "Primera sesión".to_owned(),
                application_rule: lex_core::TransitoryApplicationRule::NotApplicable,
                trigger: lex_core::TemporalBoundary {
                    boundary_type: lex_core::TemporalBoundaryType::EffectiveDate,
                    date: chrono::NaiveDate::from_ymd_opt(2018, 3, 10),
                    description: None,
                },
                end_condition: lex_core::TemporalBoundary {
                    boundary_type: lex_core::TemporalBoundaryType::RelativePeriod,
                    date: None,
                    description: Some("seis meses".to_owned()),
                },
                responsible_authorities: vec!["Grupo de Innovación Financiera".to_owned()],
                verification_status:
                    lex_core::TemporalVerificationStatus::ExternalVerificationRequired,
                verification_source_url: None,
                verified_event_date: None,
                verification_note: None,
            }],
            amendment_marks: Vec::new(),
            repeals: Vec::new(),
            commencement_condition: None,
        }
    }

    fn sample_article(number: &str) -> Provision {
        Provision {
            numbering_basis: None,
            schema_version: SCHEMA_VERSION.to_owned(),
            id: format!("{LRITF_INSTRUMENT_ID}:article:{number}"),
            instrument_id: LRITF_INSTRUMENT_ID.to_owned(),
            provision_type: ProvisionType::Article,
            label: format!("Artículo {number}"),
            number: number.to_owned(),
            heading_context: HeadingContext {
                libro: None,
                title: None,
                chapter: None,
                section: None,
                apartado: None,
            },
            text: format!("Texto del artículo {number}."),
            publication_date: Some(chrono::NaiveDate::from_ymd_opt(2018, 3, 9).unwrap()),
            publication_period: None,
            effective_from: None,
            effective_to: None,
            temporal_status: TemporalStatus::Unknown,
            temporal_basis: None,
            temporal_confidence: None,
            review_status: ReviewStatus::NotAnalyzed,
            transitory_effects: Vec::new(),
            amendment_marks: Vec::new(),
            repeals: Vec::new(),
            commencement_condition: None,
        }
    }

    fn sample_reference(
        target_number: &str,
        start_char: usize,
        qualifiers: Vec<ReferenceQualifier>,
    ) -> ReferenceEdge {
        ReferenceEdge {
            schema_version: SCHEMA_VERSION.to_owned(),
            id: format!(
                "{LRITF_INSTRUMENT_ID}:transitory:decima-primera:reference:{start_char}:article:{target_number}"
            ),
            source_provision_id: format!("{LRITF_INSTRUMENT_ID}:transitory:decima-primera"),
            source_span: target_number.to_owned(),
            start_char,
            end_char: start_char + target_number.chars().count(),
            target_instrument_id: LRITF_INSTRUMENT_ID.to_owned(),
            target_provision_id: format!("{LRITF_INSTRUMENT_ID}:article:{target_number}"),
            qualifiers,
            basis: Basis::ExpressCrossReference,
            confidence: 1.0,
            resolution_status: ReferenceResolutionStatus::Resolved,
            reference_form: ReferenceForm::Direct,
            outdated_reference_approval: None,
            legal_operator_review: None,
            resolution_rule: None,
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
            publication_date: Some(chrono::NaiveDate::from_ymd_opt(2018, 3, 9).unwrap()),
            publication_period: None,
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
            source_repeal_notice: None,
            issuing_authorities: Vec::new(),
            formal_publication_url: None,
            formal_publication_code: None,
            formal_source_sha256: None,
            formal_extracted_text_sha256: None,
        }
    }
    fn corpus_of(
        instrument: Instrument,
        provisions: Vec<Provision>,
        references: Vec<ReferenceEdge>,
    ) -> Corpus {
        Corpus {
            enactment_evidence: Vec::new(),
            instrument,
            provisions,
            references,
            terms: Vec::new(),
            term_usages: Vec::new(),
            amendment_references: Vec::new(),
        }
    }

    #[test]
    fn repeated_fraction_numerals_emit_no_anchor_and_links_fall_back_to_the_article() {
        // L2: CPEUM article 20 repeats fraction I under several apartados.
        let mut target = sample_article("20");
        target.text = "Apartado A\n\nI. Primero.\n\nII. Segundo.\n\nApartado B\n\nI. Otro primero."
            .to_owned();
        let mut source = sample_article("5");
        source.text = "Conforme a la fracción I del artículo 20, Apartado B.".to_owned();
        let mut reference = sample_reference(
            "20",
            38,
            vec![ReferenceQualifier {
                qualifier_type: lex_core::ReferenceQualifierType::Fraction,
                text: "fracción I".to_owned(),
                start_char: Some(14),
                end_char: Some(24),
            }],
        );
        reference.source_provision_id = source.id.clone();
        reference.source_span = "20".to_owned();
        reference.start_char = 38;
        reference.end_char = 40;
        let corpus = corpus_of(
            sample_instrument(),
            vec![target.clone(), source.clone()],
            vec![reference],
        );
        let targets = link_targets(&[(&corpus, "lritf")]);
        let anchors = &targets[&target.id].anchors;
        assert!(anchors.contains("f-ii"));
        assert!(!anchors.contains("f-i"), "{anchors:?}");
        let temp = tempdir().unwrap();
        write_obsidian(
            &corpus,
            &targets,
            &term_targets(&[(&corpus, "lritf")], &targets),
            &[],
            temp.path(),
        )
        .unwrap();
        let note = fs::read_to_string(temp.path().join("Corpus/LRITF/articulo-20.md")).unwrap();
        assert!(
            note.contains("^f-ii") && !note.contains("^f-i\n") && !note.contains("^f-i "),
            "{note}"
        );
        let from = fs::read_to_string(temp.path().join("Corpus/LRITF/articulo-5.md")).unwrap();
        assert!(from.contains("[[Corpus/LRITF/articulo-20|20]]"), "{from}");
        assert!(!from.contains("#^f-i|"), "{from}");
    }

    #[test]
    fn a_definition_link_names_only_an_anchor_the_note_emits() {
        // L3: LFCSQ article 2 defines terms inline, with no paragraph anchor.
        let mut glossary = sample_article("2");
        glossary.text = "Para efectos de esta Ley:\n\nXII. Desvío: el mal uso.\n\nXXVI. Polígono: el área, inline Z. XXVII. Otra: cosa.".to_owned();
        let term = |fraction: &str, name: &str, start: usize, end: usize| lex_core::DefinedTerm {
            schema_version: SCHEMA_VERSION.to_owned(),
            id: format!("{LRITF_INSTRUMENT_ID}:term:{name}"),
            term: name.to_owned(),
            instrument_id: LRITF_INSTRUMENT_ID.to_owned(),
            defining_provision_id: glossary.id.clone(),
            fraction: Some(fraction.to_owned()),
            inciso: None,
            start_char: start,
            end_char: end,
            basis: Basis::ExpressCrossReference,
        };
        let mut corpus = corpus_of(sample_instrument(), vec![glossary.clone()], Vec::new());
        corpus.terms = vec![
            term("XII", "desvio", 27, 52),
            term("XXVII", "otra", 90, 110),
        ];
        let targets = link_targets(&[(&corpus, "lritf")]);
        let terms = term_targets(&[(&corpus, "lritf")], &targets);
        assert_eq!(
            terms[&corpus.terms[0].id].anchor.as_deref(),
            Some("f-xii"),
            "a paragraph-leading fraction keeps its anchor"
        );
        assert_eq!(
            terms[&corpus.terms[1].id].anchor, None,
            "an inline fraction has no emitted anchor, so the link targets the note"
        );
    }

    #[test]
    fn a_qualified_fraction_definition_links_to_its_own_anchor() {
        // lgeepa 3o prints `XX BIS.`; its term's label is `XX Bis`. The anchor
        // had a space (`f-xx bis`) that no paragraph carried.
        let mut glossary = sample_article("3");
        glossary.text =
            "Para efectos de esta Ley:\n\nXX. Impacto: el efecto.\n\nXX BIS. Luz intrusa: la luz."
                .to_owned();
        let term = lex_core::DefinedTerm {
            schema_version: SCHEMA_VERSION.to_owned(),
            id: format!("{LRITF_INSTRUMENT_ID}:term:luz-intrusa"),
            term: "Luz intrusa".to_owned(),
            instrument_id: LRITF_INSTRUMENT_ID.to_owned(),
            defining_provision_id: glossary.id.clone(),
            fraction: Some("XX Bis".to_owned()),
            inciso: None,
            start_char: 46,
            end_char: 75,
            basis: Basis::ExpressDefinition,
        };
        let mut corpus = corpus_of(sample_instrument(), vec![glossary.clone()], Vec::new());
        corpus.terms = vec![term];
        let targets = link_targets(&[(&corpus, "lritf")]);
        assert!(targets[&glossary.id].fractions.contains("xx-bis"));
        let terms = term_targets(&[(&corpus, "lritf")], &targets);
        assert_eq!(
            terms[&corpus.terms[0].id].anchor.as_deref(),
            Some("f-xx-bis")
        );
    }

    #[test]
    fn an_inciso_definition_links_to_its_inciso_or_the_paragraph_that_holds_it() {
        // LIEPS 3 VIII, operator ruling 2026-10-19: each inciso links to its
        // own anchor; `c)` shares `b)`'s paragraph, so it links there.
        let mut glossary = sample_article("3");
        glossary.text = "Se entiende por:\n\nVIII. Tabacos labrados y otros:\n\na) Cigarros, los \
                         cigarros.\n\nb) Puros, los puros. c) Otros tabacos labrados, los demás."
            .to_owned();
        let chars: Vec<char> = glossary.text.chars().collect();
        let at = |needle: &str| {
            let byte = glossary.text.find(needle).unwrap();
            glossary.text[..byte].chars().count()
        };
        let term = |name: &str, inciso: Option<&str>, start: usize| lex_core::DefinedTerm {
            schema_version: SCHEMA_VERSION.to_owned(),
            id: format!("{LRITF_INSTRUMENT_ID}:term:{name}"),
            term: name.to_owned(),
            instrument_id: LRITF_INSTRUMENT_ID.to_owned(),
            defining_provision_id: glossary.id.clone(),
            fraction: Some("VIII".to_owned()),
            inciso: inciso.map(str::to_owned),
            start_char: start,
            end_char: chars.len(),
            basis: Basis::ExpressDefinition,
        };
        let mut corpus = corpus_of(sample_instrument(), vec![glossary.clone()], Vec::new());
        corpus.terms = vec![
            term("tabacos-labrados", None, at("VIII.")),
            term("cigarros", Some("a"), at("a)")),
            term("puros", Some("b"), at("b)")),
            term("otros-tabacos-labrados", Some("c"), at("c)")),
        ];
        let targets = link_targets(&[(&corpus, "lritf")]);
        let terms = term_targets(&[(&corpus, "lritf")], &targets);
        let anchor = |index: usize| terms[&corpus.terms[index].id].anchor.clone();
        assert_eq!(anchor(0).as_deref(), Some("f-viii"));
        assert_eq!(anchor(1).as_deref(), Some("f-viii-a"));
        assert_eq!(anchor(2).as_deref(), Some("f-viii-b"));
        assert_eq!(anchor(3).as_deref(), Some("f-viii-b"));
    }

    #[test]
    fn local_versus_cross_instrument_paths_follow_instrument_identity_not_short_name() {
        // L6: two different instruments sharing a display acronym.
        let mut other = sample_instrument();
        other.id = "urn:lex-mx:federal:statute:other".to_owned();
        let mut other_article = sample_article("20");
        other_article.id = format!("{}:article:20", other.id);
        other_article.instrument_id = other.id.clone();
        let mut source = sample_article("5");
        source.text = "Conforme al artículo 20.".to_owned();
        let mut reference = sample_reference("20", 20, Vec::new());
        reference.source_provision_id = source.id.clone();
        reference.target_instrument_id = other.id.clone();
        reference.target_provision_id = other_article.id.clone();
        reference.source_span = "20".to_owned();
        reference.start_char = 21;
        reference.end_char = 23;
        let local = corpus_of(
            sample_instrument(),
            vec![source.clone(), sample_article("20")],
            vec![reference],
        );
        let remote = corpus_of(other, vec![other_article], Vec::new());
        let targets = link_targets(&[(&local, "lritf"), (&remote, "other")]);
        let temp = tempdir().unwrap();
        write_markdown(
            &local,
            &targets,
            &term_targets(&[(&local, "lritf")], &targets),
            temp.path(),
        )
        .unwrap();
        let note = fs::read_to_string(temp.path().join("articulo-5.md")).unwrap();
        assert!(
            note.contains("[20](../../other/markdown/articulo-20.md)"),
            "{note}"
        );
    }

    #[test]
    fn military_slash_heading_exports_as_one_file_and_keeps_display_label() {
        let temp = tempdir().unwrap();
        let mut article = sample_article("1/o BIS");
        article.label = "Artículo 1/o BIS".to_owned();
        let corpus = Corpus {
            enactment_evidence: Vec::new(),
            instrument: sample_instrument(),
            provisions: vec![article],
            references: Vec::new(),
            terms: Vec::new(),
            term_usages: Vec::new(),
            amendment_references: Vec::new(),
        };
        let targets = link_targets(&[(&corpus, "lritf")]);
        write_obsidian(
            &corpus,
            &targets,
            &term_targets(&[(&corpus, "lritf")], &targets),
            &[],
            temp.path(),
        )
        .unwrap();
        let file = temp.path().join("Corpus/LRITF/articulo-1-o-BIS.md");
        assert!(file.is_file());
        assert!(
            fs::read_to_string(file)
                .unwrap()
                .contains("Artículo 1/o BIS")
        );
        assert_eq!(
            targets.values().next().unwrap().filename,
            "articulo-1-o-BIS.md"
        );
    }
    #[test]
    fn decree_evidence_exports_separately_and_is_removed_when_cleared() {
        let temp = tempdir().unwrap();
        let mut corpus = Corpus {
            instrument: sample_instrument(),
            provisions: vec![sample_provision()],
            enactment_evidence: Vec::new(),
            references: Vec::new(),
            terms: Vec::new(),
            term_usages: Vec::new(),
            amendment_references: Vec::new(),
        };
        let kind = lex_core::EnactmentEvidenceKind::Transitory;
        corpus.enactment_evidence.push(lex_core::EnactmentEvidence {
            schema_version: SCHEMA_VERSION.to_owned(),
            id: lex_core::enactment_evidence_id(
                &corpus.instrument.id,
                corpus.instrument.publication_date,
                &kind,
                "PRIMERO",
            ),
            instrument_id: corpus.instrument.id.clone(),
            owner: lex_core::EnactmentOwner::ContainingDecree,
            kind,
            containing_act_title: "DECRETO de expedición".to_owned(),
            section_heading: Some("TRANSITORIOS".to_owned()),
            source_heading: "PRIMERO.-".to_owned(),
            number: "PRIMERO".to_owned(),
            text: "Texto del decreto.".to_owned(),
            publication_date: corpus.instrument.publication_date,
            publication_period: None,
            source_sha256: corpus.instrument.source_sha256.clone(),
            extracted_text_sha256: corpus.instrument.extracted_text_sha256.clone(),
            text_sha256: lex_core::evidence_sha256("Texto del decreto."),
        });
        let targets = link_targets(&[(&corpus, "lritf")]);
        let terms = term_targets(&[(&corpus, "lritf")], &targets);
        let md = temp.path().join("markdown");
        let canonical = temp.path().join("canonical");
        super::write_canonical(&corpus, &canonical).unwrap();
        write_markdown(&corpus, &targets, &terms, &md).unwrap();
        write_obsidian(&corpus, &targets, &terms, &[], temp.path()).unwrap();
        let decree = fs::read_to_string(md.join("decreto-expedicion.md")).unwrap();
        assert!(decree.contains("Transitorio del decreto"));
        assert!(decree.contains("Texto del decreto."));
        let vault = temp.path().join("Corpus/LRITF");
        assert!(vault.join("enactment-evidence.json").exists());
        corpus.enactment_evidence.clear();
        super::write_canonical(&corpus, &canonical).unwrap();
        write_markdown(&corpus, &targets, &terms, &md).unwrap();
        write_obsidian(&corpus, &targets, &terms, &[], temp.path()).unwrap();
        assert_eq!(
            fs::read_to_string(canonical.join("enactment-evidence.json"))
                .unwrap()
                .trim(),
            "[]"
        );
        for directory in [&md, &vault] {
            assert!(!directory.join("decreto-expedicion.md").exists());
            assert!(!directory.join("enactment-evidence.json").exists());
        }
    }
    #[test]
    fn historical_instrument_notice_is_visible_without_changing_provision_state() {
        let mut instrument = sample_instrument();
        instrument.status = InstrumentStatus::Repealed;
        instrument.source_repeal_notice = Some(lex_core::SourceRepealNotice {
            source_note: "Ley Abrogada DOF 16-07-2025".to_owned(),
            source_sha256: instrument.source_sha256.clone(),
            formal_publication_url: "https://www.dof.gob.mx/nota_detalle.php?codigo=fixture"
                .parse()
                .unwrap(),
            formal_source_sha256: "b".repeat(64),
            successor_instrument_id: None,
            successor_clause: None,
            reviewer: "Fixture reviewer".to_owned(),
            captured_at: "2026-10-16T00:00:00Z".to_owned(),
            reviewer_supplied_at: None,
            rationale: "Fixture only".to_owned(),
            decision_source: "fixture".to_owned(),
            decision_source_sha256: "c".repeat(64),
        });
        let corpus = Corpus {
            instrument,
            provisions: vec![sample_article("1")],
            references: vec![],
            terms: vec![],
            term_usages: vec![],
            amendment_references: vec![],
            enactment_evidence: vec![],
        };
        let targets = link_targets(&[(&corpus, "lritf")]);
        let terms = term_targets(&[(&corpus, "lritf")], &targets);
        let index = super::markdown_index(&corpus, &targets, &terms, false);
        assert!(
            index.contains("Ley abrogada — texto histórico")
                && index.contains("Ley Abrogada DOF 16-07-2025")
        );
        let article = super::standard_markdown(&corpus, &targets, &terms, &corpus.provisions[0]);
        assert!(article.contains("Ley abrogada — texto histórico"));
        assert_eq!(
            corpus.provisions[0].temporal_status,
            TemporalStatus::Unknown
        );
        assert_eq!(
            corpus.provisions[0].review_status,
            ReviewStatus::NotAnalyzed
        );
    }
    #[test]
    fn successor_clause_notice_cites_abrogating_provision_not_a_header() {
        let mut instrument = sample_instrument();
        instrument.status = InstrumentStatus::Repealed;
        instrument.source_repeal_notice = Some(lex_core::SourceRepealNotice {
            source_note: "se abroga al Reglamento anterior".to_owned(),
            source_sha256: instrument.source_sha256.clone(),
            formal_publication_url: "https://dof.gob.mx/nota_detalle.php?codigo=fixture"
                .parse()
                .unwrap(),
            formal_source_sha256: "b".repeat(64),
            successor_instrument_id: Some("urn:lex-mx:federal:regulation:succ".to_owned()),
            successor_clause: Some(lex_core::SuccessorAbrogationClause {
                provision_id: "urn:lex-mx:federal:regulation:succ:transitory:segundo".to_owned(),
                provision_text_sha256: "d".repeat(64),
            }),
            reviewer: "Fixture reviewer".to_owned(),
            captured_at: "2026-10-16T00:00:00Z".to_owned(),
            reviewer_supplied_at: None,
            rationale: "Fixture only".to_owned(),
            decision_source: "fixture".to_owned(),
            decision_source_sha256: "c".repeat(64),
        });
        let corpus = Corpus {
            instrument,
            provisions: vec![sample_article("1")],
            references: vec![],
            terms: vec![],
            term_usages: vec![],
            amendment_references: vec![],
            enactment_evidence: vec![],
        };
        let targets = link_targets(&[(&corpus, "lritf")]);
        let terms = term_targets(&[(&corpus, "lritf")], &targets);
        let index = super::markdown_index(&corpus, &targets, &terms, false);
        assert!(
            index.contains("Instrumento abrogado — texto histórico")
                && index.contains("se abroga al Reglamento anterior")
                && index.contains("succ:transitory:segundo")
                && !index.contains("Ley abrogada")
        );
    }
}
