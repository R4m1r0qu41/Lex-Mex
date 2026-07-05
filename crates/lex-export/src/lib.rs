use std::{collections::HashMap, fmt::Write as _, fs, path::Path};

use anyhow::{Context, Result};
use lex_core::{
    Corpus, Provision, ProvisionType, ReferenceForm, ReferenceResolutionStatus, ReviewItem,
    ReviewItemStatus, TemporalBoundary, TemporalBoundaryType, ValidationReport,
};

/// Presentation location of a link target, possibly in another instrument.
#[derive(Debug, Clone)]
pub struct LinkTarget {
    pub instrument_short_name: String,
    /// Corpus directory slug used for standard-Markdown relative paths.
    pub instrument_slug: String,
    pub filename: String,
    /// Lowercase roman numerals of the fraction paragraphs the target
    /// provision has, each carrying a `^f-<numeral>` block anchor in its
    /// generated note.
    pub fractions: std::collections::HashSet<String>,
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
    /// definition, `t-<slug>` for a colon-style one.
    pub anchor: String,
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
    let fraction_re = regex::Regex::new(r"^([IVXLCDM]+)\.\s").expect("static regex");
    let mut targets = LinkTargets::new();
    for (corpus, slug) in corpora {
        for provision in &corpus.provisions {
            let fractions = provision
                .text
                .split("\n\n")
                .filter_map(|paragraph| {
                    fraction_re
                        .captures(paragraph)
                        .map(|captures| captures[1].to_lowercase())
                })
                .collect();
            targets.insert(
                provision.id.clone(),
                LinkTarget {
                    instrument_short_name: corpus.instrument.short_name.clone(),
                    instrument_slug: (*slug).to_owned(),
                    filename: markdown_filename(provision),
                    fractions,
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
            output.insert(
                term.id.clone(),
                TermTarget {
                    note: note.clone(),
                    anchor: term_anchor(term),
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
    let roman_re = regex::Regex::new(r"\b[IVXLCDM]+\b").expect("static regex");
    roman_re
        .find_iter(text)
        .map(|item| QualifierNumeral {
            text: item.as_str(),
            offset_chars: text[..item.start()].chars().count(),
        })
        .collect()
}

fn term_anchor(term: &lex_core::DefinedTerm) -> String {
    if let Some(fraction) = &term.fraction {
        format!("f-{}", fraction.to_lowercase())
    } else {
        let slug = term.id.rsplit(":term:").next().unwrap_or("term");
        format!("t-{slug}")
    }
}

pub fn write_canonical(corpus: &Corpus, output_dir: &Path) -> Result<()> {
    fs::create_dir_all(output_dir)?;
    write_json(&corpus.instrument, &output_dir.join("instrument.json"))?;
    write_json(&corpus.provisions, &output_dir.join("provisions.json"))?;
    write_json(&corpus.references, &output_dir.join("references.json"))?;
    write_json(&corpus.terms, &output_dir.join("terms.json"))?;
    write_json(&corpus.term_usages, &output_dir.join("term-usages.json"))
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
    fs::create_dir_all(&instrument_dir)?;
    let mut generated_files = Vec::with_capacity(corpus.provisions.len() + 1);
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
    format!(
        "---\nid: {}\ninstrument_id: {}\ninstrument: {}\nprovision_type: {}\nnumber: \"{}\"\naliases: [{}]\ngenerated: true\ntemporal_status: {}\nreview_status: {}\n{}source_url: {}\nsource_sha256: {}\n---\n\n",
        provision.id,
        provision.instrument_id,
        corpus.instrument.short_name,
        json_name(&provision.provision_type),
        provision.number,
        serde_json::to_string(&alias).expect("serializing alias cannot fail"),
        json_name(&provision.temporal_status),
        json_name(&provision.review_status),
        effect_front_matter,
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
    let _ = write!(
        output,
        "[[Corpus/{0}/{0}|← Índice {0}]]\n\n",
        corpus.instrument.short_name
    );
    let body = linked_string(corpus, targets, terms, &provision.text, &provision.id, true);
    let body = with_block_anchors(corpus, provision, &body);
    let _ = write!(output, "# {}\n\n{body}\n", provision.label);
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
    let fraction_re = regex::Regex::new(r"^([IVXLCDM]+)\.\s").expect("static regex");
    let mut anchors_by_offset: HashMap<usize, String> = HashMap::new();
    for term in &corpus.terms {
        if term.defining_provision_id == provision.id && term.fraction.is_none() {
            anchors_by_offset.insert(term.start_char, term_anchor(term));
        }
    }

    let mut offset = 0;
    let mut anchors = Vec::new();
    for paragraph in provision.text.split("\n\n") {
        let anchor = anchors_by_offset.get(&offset).cloned().or_else(|| {
            fraction_re
                .captures(paragraph)
                .map(|captures| format!("f-{}", captures[1].to_lowercase()))
        });
        anchors.push(anchor);
        offset += paragraph.chars().count() + 2;
    }

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
            let numeral = edge.source_span.to_lowercase();
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
                let lower = numeral.text.to_lowercase();
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
            anchor: Some(term_target.anchor.clone()),
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
        } else if target.instrument_short_name == corpus.instrument.short_name {
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
        ProvisionType::Article => format!("articulo-{}.md", provision.number.replace(' ', "-")),
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
            instrument: sample_instrument(),
            provisions: vec![sample_provision()],
            references: Vec::new(),
            terms: Vec::new(),
            term_usages: Vec::new(),
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
                section: None,
                apartado: None,
            },
            text: "Texto.".to_owned(),
            publication_date: chrono::NaiveDate::from_ymd_opt(2018, 3, 9).unwrap(),
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
        }
    }

    fn sample_article(number: &str) -> Provision {
        Provision {
            schema_version: SCHEMA_VERSION.to_owned(),
            id: format!("{LRITF_INSTRUMENT_ID}:article:{number}"),
            instrument_id: LRITF_INSTRUMENT_ID.to_owned(),
            provision_type: ProvisionType::Article,
            label: format!("Artículo {number}"),
            number: number.to_owned(),
            heading_context: HeadingContext {
                title: None,
                chapter: None,
                section: None,
                apartado: None,
            },
            text: format!("Texto del artículo {number}."),
            publication_date: chrono::NaiveDate::from_ymd_opt(2018, 3, 9).unwrap(),
            effective_from: None,
            effective_to: None,
            temporal_status: TemporalStatus::Unknown,
            temporal_basis: None,
            temporal_confidence: None,
            review_status: ReviewStatus::NotAnalyzed,
            transitory_effects: Vec::new(),
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
            issuing_authorities: Vec::new(),
            formal_publication_url: None,
            formal_publication_code: None,
            formal_source_sha256: None,
            formal_extracted_text_sha256: None,
        }
    }
}
