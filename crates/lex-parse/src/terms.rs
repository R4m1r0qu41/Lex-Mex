//! Defined-term (glossary) extraction and exact usage linking.
//!
//! Mexican financial instruments commonly open with a glossary provision —
//! LRITF Article 4, DCG-IFPE-2021 Article 1 — though not always. Two layouts
//! are supported: `fractions` (`I. Término, a …`, continuation paragraphs
//! such as incisos belong to the preceding fraction) and `colon_entries`
//! (`Término: a …`, produced by the definition-layout reconstruction, with
//! non-entry paragraphs continuing the previous entry).
//!
//! Usage linking is deterministic and case-sensitive: an occurrence matches
//! a term exactly or through one generated singular/plural variant, at word
//! boundaries, longest match first. A glossary may be additive to another
//! instrument's glossary (the DCG defines its terms "además de los términos
//! utilizados en la Ley…"), so resolution tries the instrument's own terms
//! before earlier instruments' terms.

use anyhow::{Context, Result, bail};
use lex_core::{Basis, DefinedTerm, Provision, SCHEMA_VERSION, TermUsage};
use regex::Regex;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GlossaryStyle {
    /// `I. Término, a …` — the term runs to the first comma.
    Fractions,
    /// `Término: a …` — the term runs to the first colon.
    ColonEntries,
}

impl GlossaryStyle {
    pub fn from_config(value: &str) -> Result<Self> {
        match value {
            "fractions" => Ok(Self::Fractions),
            "colon_entries" => Ok(Self::ColonEntries),
            other => bail!("unsupported glossary style {other:?}"),
        }
    }
}

/// Longest term text an entry prefix may have in a colon-entries glossary;
/// longer prefixes mean the colon belongs to running definition text.
const COLON_TERM_MAX_CHARS: usize = 90;

/// Extract the defined terms of one glossary provision.
pub fn extract_terms(provision: &Provision, style: GlossaryStyle) -> Result<Vec<DefinedTerm>> {
    let entries = match style {
        GlossaryStyle::Fractions => fraction_entries(provision)?,
        GlossaryStyle::ColonEntries => colon_entries(provision),
    };
    let mut terms = Vec::with_capacity(entries.len());
    for entry in entries {
        if entry.term.trim().is_empty() {
            bail!("empty defined term in glossary provision {}", provision.id);
        }
        terms.push(DefinedTerm {
            schema_version: SCHEMA_VERSION.to_owned(),
            id: format!("{}:term:{}", provision.instrument_id, slug(&entry.term)),
            term: entry.term,
            instrument_id: provision.instrument_id.clone(),
            defining_provision_id: provision.id.clone(),
            fraction: entry.fraction,
            start_char: entry.start_char,
            end_char: entry.end_char,
            basis: Basis::ExpressDefinition,
        });
    }
    Ok(terms)
}

struct GlossaryEntry {
    term: String,
    fraction: Option<String>,
    start_char: usize,
    end_char: usize,
}

fn fraction_entries(provision: &Provision) -> Result<Vec<GlossaryEntry>> {
    let fraction_re = Regex::new(r"^([IVXLCDM]+)\.\s+(.*)$")?;
    let mut entries: Vec<GlossaryEntry> = Vec::new();
    for (start_char, paragraph) in paragraphs_with_offsets(&provision.text) {
        let end_char = start_char + paragraph.chars().count();
        if let Some(captures) = fraction_re.captures(paragraph) {
            let body = captures.get(2).expect("body capture").as_str();
            let term = body.split(',').next().unwrap_or("").trim().to_owned();
            entries.push(GlossaryEntry {
                term,
                fraction: Some(captures[1].to_owned()),
                start_char,
                end_char,
            });
        } else if let Some(entry) = entries.last_mut() {
            // Continuation paragraph (for example, incisos) extends the
            // previous fraction's definition span.
            entry.end_char = end_char;
        }
    }
    Ok(entries)
}

fn colon_entries(provision: &Provision) -> Vec<GlossaryEntry> {
    let mut entries: Vec<GlossaryEntry> = Vec::new();
    let mut in_entries = false;
    for (start_char, paragraph) in paragraphs_with_offsets(&provision.text) {
        let end_char = start_char + paragraph.chars().count();
        if !in_entries {
            // Entries begin after the intro paragraph ending "…los
            // siguientes:"; everything before it is preamble.
            if paragraph.trim_end().ends_with("los siguientes:") {
                in_entries = true;
            }
            continue;
        }
        let term = paragraph
            .split(':')
            .next()
            .filter(|prefix| {
                prefix.chars().count() <= COLON_TERM_MAX_CHARS
                    && prefix.chars().count() < paragraph.chars().count()
            })
            .map(str::trim);
        match term {
            Some(term) => entries.push(GlossaryEntry {
                term: term.to_owned(),
                fraction: None,
                start_char,
                end_char,
            }),
            // A paragraph without an early colon continues the previous
            // entry (for example, "Para efectos de la presente definición…").
            None => {
                if let Some(entry) = entries.last_mut() {
                    entry.end_char = end_char;
                }
            }
        }
    }
    entries
}

/// Extract every exact defined-term occurrence across the provisions.
///
/// `term_sets` is ordered by resolution priority: the instrument's own
/// glossary first, then the glossaries it is expressly additive to. At any
/// text position the longest matching variant wins; on equal length the
/// earlier term set wins. A term never matches inside its own definition
/// entry, and the glossary entry prefix itself is the definition site, not
/// a usage.
pub fn extract_term_usages(
    provisions: &[Provision],
    term_sets: &[&[DefinedTerm]],
) -> Result<Vec<TermUsage>> {
    let mut candidates: Vec<(String, &DefinedTerm)> = Vec::new();
    for set in term_sets {
        for term in *set {
            for variant in term_variants(&term.term) {
                candidates.push((variant, term));
            }
        }
    }
    // Longest variants first; ties keep `term_sets` priority order because
    // the sort is stable.
    candidates.sort_by_key(|(variant, _)| std::cmp::Reverse(variant.chars().count()));

    let mut usages = Vec::new();
    for provision in provisions {
        usages.extend(provision_usages(provision, &candidates));
    }
    Ok(usages)
}

fn provision_usages(
    provision: &Provision,
    candidates: &[(String, &DefinedTerm)],
) -> Vec<TermUsage> {
    let chars: Vec<char> = provision.text.chars().collect();
    let mut usages = Vec::new();
    let mut position = 0;
    while position < chars.len() {
        if !starts_word(&chars, position) {
            position += 1;
            continue;
        }
        let mut matched = None;
        for (variant, term) in candidates {
            if matches_at(&chars, position, variant)
                && !inside_own_definition(term, provision, position)
                && capitalization_is_informative(&chars, position, variant)
            {
                matched = Some((variant.chars().count(), *term));
                break;
            }
        }
        let Some((length, term)) = matched else {
            position += 1;
            continue;
        };
        let end_char = position + length;
        let span: String = chars[position..end_char].iter().collect();
        usages.push(TermUsage {
            schema_version: SCHEMA_VERSION.to_owned(),
            id: format!("{}:term-usage:{position}-{end_char}", provision.id),
            provision_id: provision.id.clone(),
            term_id: term.id.clone(),
            span,
            start_char: position,
            end_char,
        });
        position = end_char;
    }
    usages
}

fn starts_word(chars: &[char], position: usize) -> bool {
    chars[position].is_alphabetic() && (position == 0 || !chars[position - 1].is_alphanumeric())
}

fn matches_at(chars: &[char], position: usize, variant: &str) -> bool {
    let mut end = position;
    for expected in variant.chars() {
        if chars.get(end) != Some(&expected) {
            return false;
        }
        end += 1;
    }
    // Word boundary after the match.
    chars.get(end).is_none_or(|next| !next.is_alphanumeric())
}

fn inside_own_definition(term: &DefinedTerm, provision: &Provision, position: usize) -> bool {
    provision.id == term.defining_provision_id
        && position >= term.start_char
        && position < term.end_char
}

/// Defined-term matching is case-sensitive because capitalization is what
/// distinguishes the defined `Control` from the ordinary word `control`.
/// At a sentence, list-item, or table-cell start the capital is positional
/// and carries no signal, so a term whose only capital is its initial
/// letter does not match there (`I. Controles de acceso…` is not the
/// defined `Control`). Terms with capitals beyond the first character —
/// acronyms such as `CNBV`, multi-word terms such as `Autoridad
/// Financiera` — remain informative anywhere.
fn capitalization_is_informative(chars: &[char], position: usize, variant: &str) -> bool {
    if !only_initial_capital(variant) {
        return true;
    }
    let mut index = position;
    while index > 0 && chars[index - 1].is_whitespace() {
        index -= 1;
    }
    if index == 0 {
        return false;
    }
    !matches!(chars[index - 1], '.' | ':' | ';' | ')' | '|')
}

fn only_initial_capital(variant: &str) -> bool {
    let mut characters = variant.chars();
    characters.next().is_some_and(char::is_uppercase)
        && characters.all(|character| !character.is_uppercase())
}

/// Generate the deterministic singular/plural variants of a term. Glossary
/// provisions state that terms apply "en singular o plural", so a term
/// defined as `Operaciones` must match `Operación` and vice versa. Rules,
/// applied word by word (skipping short connector words):
/// `-ón` ↔ `-ones`; vowel ending ↔ `+s`; consonant ending ↔ `+es`.
fn term_variants(term: &str) -> Vec<String> {
    let mut variants = vec![term.to_owned()];
    for transformed in [
        transform_words(term, pluralize),
        transform_words(term, singularize),
    ] {
        if let Some(value) = transformed
            && !variants.contains(&value)
        {
            variants.push(value);
        }
    }
    variants
}

fn transform_words(term: &str, transform: fn(&str) -> Option<String>) -> Option<String> {
    let words: Vec<&str> = term.split(' ').collect();
    let mut output = Vec::with_capacity(words.len());
    let mut changed = false;
    for word in words {
        if is_connector(word) {
            output.push(word.to_owned());
            continue;
        }
        match transform(word) {
            Some(transformed) => {
                changed = true;
                output.push(transformed);
            }
            None => output.push(word.to_owned()),
        }
    }
    changed.then(|| output.join(" "))
}

fn is_connector(word: &str) -> bool {
    matches!(
        word.to_lowercase().as_str(),
        "de" | "del" | "la" | "el" | "los" | "las" | "y" | "o" | "en" | "a" | "una" | "un"
    )
}

fn pluralize(word: &str) -> Option<String> {
    let lower_end = word.chars().last()?;
    if word.chars().count() < 3 || word.ends_with('s') || word.ends_with('S') {
        return None;
    }
    if let Some(stem) = word.strip_suffix("ón") {
        return Some(format!("{stem}ones"));
    }
    if matches!(
        lower_end,
        'a' | 'e' | 'i' | 'o' | 'u' | 'á' | 'é' | 'í' | 'ó' | 'ú'
    ) {
        return Some(format!("{word}s"));
    }
    if lower_end.is_alphabetic() {
        return Some(format!("{word}es"));
    }
    None
}

fn singularize(word: &str) -> Option<String> {
    if word.chars().count() < 4 {
        return None;
    }
    if let Some(stem) = word.strip_suffix("ones") {
        return Some(format!("{stem}ón"));
    }
    if let Some(stem) = word.strip_suffix("es") {
        // Only strip `-es` after a consonant that takes it (`-dades`,
        // `-ores`…); vowel stems take plain `-s`.
        if stem
            .chars()
            .last()
            .is_some_and(|last| matches!(last, 'd' | 'l' | 'r' | 'n' | 'j'))
        {
            return Some(stem.to_owned());
        }
    }
    if let Some(stem) = word.strip_suffix('s')
        && stem.chars().last().is_some_and(|last| {
            matches!(
                last,
                'a' | 'e' | 'i' | 'o' | 'u' | 'á' | 'é' | 'í' | 'ó' | 'ú'
            )
        })
    {
        return Some(stem.to_owned());
    }
    None
}

/// Paragraphs of a provision's canonical text with the Unicode character
/// offset each paragraph starts at.
fn paragraphs_with_offsets(text: &str) -> Vec<(usize, &str)> {
    let mut output = Vec::new();
    let mut offset = 0;
    for paragraph in text.split("\n\n") {
        output.push((offset, paragraph));
        offset += paragraph.chars().count() + 2;
    }
    output
}

fn slug(value: &str) -> String {
    value
        .to_lowercase()
        .replace('á', "a")
        .replace('é', "e")
        .replace('í', "i")
        .replace('ó', "o")
        .replace(['ú', 'ü'], "u")
        .replace('ñ', "n")
        .replace(' ', "-")
}

pub fn find_glossary_provision<'a>(
    provisions: &'a [Provision],
    provision_suffix: &str,
) -> Result<&'a Provision> {
    provisions
        .iter()
        .find(|provision| provision.id.ends_with(provision_suffix))
        .with_context(|| format!("glossary provision {provision_suffix} not found"))
}

#[cfg(test)]
mod tests {
    use lex_core::{
        HeadingContext, Provision, ProvisionType, ReviewStatus, SCHEMA_VERSION, TemporalStatus,
    };
    use pretty_assertions::assert_eq;

    use super::{GlossaryStyle, extract_term_usages, extract_terms};

    const STATUTE_ID: &str = "urn:lex-mx:test:statute";
    const REGULATION_ID: &str = "urn:lex-mx:test:regulation";

    fn provision(instrument_id: &str, suffix: &str, text: &str) -> Provision {
        Provision {
            schema_version: SCHEMA_VERSION.to_owned(),
            id: format!("{instrument_id}:{suffix}"),
            instrument_id: instrument_id.to_owned(),
            provision_type: ProvisionType::Article,
            label: suffix.to_owned(),
            number: "1".to_owned(),
            heading_context: HeadingContext {
                libro: None,
                title: None,
                chapter: None,
                section: None,
                apartado: None,
            },
            text: text.to_owned(),
            publication_date: chrono::NaiveDate::from_ymd_opt(2021, 1, 28).unwrap(),
            effective_from: None,
            effective_to: None,
            temporal_status: TemporalStatus::Unknown,
            temporal_basis: None,
            temporal_confidence: None,
            review_status: ReviewStatus::NotAnalyzed,
            transitory_effects: Vec::new(),
            amendment_marks: Vec::new(),
        }
    }

    fn statute_glossary() -> Provision {
        provision(
            STATUTE_ID,
            "article:4",
            "Para efectos de esta Ley, en singular o plural, se entenderá por:\n\n\
             II. Cliente, a la persona física o moral que contrata una Operación;\n\n\
             III. CNBV, a la Comisión Nacional Bancaria y de Valores;\n\n\
             X. Control, a la capacidad de imponer decisiones:\n\n\
             a) En las asambleas generales de accionistas;\n\n\
             XVIII. Operaciones, a las actividades reguladas por esta Ley;",
        )
    }

    fn regulation_glossary() -> Provision {
        provision(
            REGULATION_ID,
            "article:1",
            "Para efectos de las presentes Disposiciones, se entenderá, además de los \
             términos utilizados en la Ley, los siguientes:\n\n\
             Cuenta: a aquel registro contable en el que se anotan abonos del Cliente.\n\n\
             Para efectos de la presente definición se entenderá un registro por Cliente.\n\n\
             Usuario de la Infraestructura Tecnológica: a la persona autorizada.",
        )
    }

    #[test]
    fn extracts_fraction_style_terms_with_continuation_spans() {
        let glossary = statute_glossary();
        let terms = extract_terms(&glossary, GlossaryStyle::Fractions).unwrap();
        assert_eq!(
            terms
                .iter()
                .map(|term| (term.term.as_str(), term.fraction.as_deref().unwrap()))
                .collect::<Vec<_>>(),
            [
                ("Cliente", "II"),
                ("CNBV", "III"),
                ("Control", "X"),
                ("Operaciones", "XVIII"),
            ]
        );
        // Control's span extends over its inciso continuation paragraph.
        let control = &terms[2];
        let chars: Vec<char> = glossary.text.chars().collect();
        let span: String = chars[control.start_char..control.end_char].iter().collect();
        assert!(span.starts_with("X. Control"));
        assert!(span.ends_with("asambleas generales de accionistas;"));
    }

    #[test]
    fn extracts_colon_style_terms_with_continuation_spans() {
        let glossary = regulation_glossary();
        let terms = extract_terms(&glossary, GlossaryStyle::ColonEntries).unwrap();
        assert_eq!(
            terms
                .iter()
                .map(|term| term.term.as_str())
                .collect::<Vec<_>>(),
            ["Cuenta", "Usuario de la Infraestructura Tecnológica"]
        );
        let chars: Vec<char> = glossary.text.chars().collect();
        let cuenta: String = chars[terms[0].start_char..terms[0].end_char]
            .iter()
            .collect();
        assert!(cuenta.ends_with("un registro por Cliente."));
    }

    #[test]
    fn resolves_usages_with_variants_priority_and_positional_capitals() {
        let statute_terms = extract_terms(&statute_glossary(), GlossaryStyle::Fractions).unwrap();
        let regulation_glossary = regulation_glossary();
        let regulation_terms =
            extract_terms(&regulation_glossary, GlossaryStyle::ColonEntries).unwrap();
        let body = provision(
            REGULATION_ID,
            "article:9",
            "Los Clientes que ejerzan Control sobre una Cuenta lo notificarán a la CNBV. \
             Cada Operación quedará registrada:\n\n\
             I. Controles de acceso lógico a las Cuentas. La CNBV supervisará.",
        );
        let provisions = vec![body, regulation_glossary];
        let usages =
            extract_term_usages(&provisions, &[&regulation_terms, &statute_terms]).unwrap();

        let spans: Vec<(&str, &str)> = usages
            .iter()
            .filter(|usage| usage.provision_id.ends_with(":article:9"))
            .map(|usage| (usage.span.as_str(), usage.term_id.as_str()))
            .collect();
        // Plural and accent-shifting singular variants resolve; `Cuenta`
        // resolves to the regulation's own glossary; `Cliente`, `Control`,
        // `CNBV`, and `Operación` fall through to the statute's.
        assert!(spans.contains(&("Clientes", "urn:lex-mx:test:statute:term:cliente")));
        assert!(spans.contains(&("Control", "urn:lex-mx:test:statute:term:control")));
        assert!(spans.contains(&("Cuenta", "urn:lex-mx:test:regulation:term:cuenta")));
        assert!(spans.contains(&("Operación", "urn:lex-mx:test:statute:term:operaciones")));
        // Item-start `Controles` is positional capitalization, not the
        // defined term; the acronym CNBV still matches after a period.
        assert!(!spans.iter().any(|(span, _)| *span == "Controles"));
        assert_eq!(spans.iter().filter(|(span, _)| *span == "CNBV").count(), 2);
        // Inside the glossary itself, other terms' definitions still use
        // terms (`Cliente` in Cuenta's definition), but a term never
        // matches inside its own definition entry.
        let glossary_spans: Vec<&str> = usages
            .iter()
            .filter(|usage| usage.provision_id.ends_with(":article:1"))
            .map(|usage| usage.span.as_str())
            .collect();
        assert!(glossary_spans.contains(&"Cliente"));
        assert!(!glossary_spans.contains(&"Cuenta"));
    }

    #[test]
    fn longest_match_wins_over_embedded_terms() {
        let regulation_glossary = regulation_glossary();
        let regulation_terms =
            extract_terms(&regulation_glossary, GlossaryStyle::ColonEntries).unwrap();
        let body = provision(
            REGULATION_ID,
            "article:12",
            "El Usuario de la Infraestructura Tecnológica accederá con su Cuenta.",
        );
        let usages = extract_term_usages(&[body], &[&regulation_terms]).unwrap();
        assert_eq!(
            usages
                .iter()
                .map(|usage| usage.span.as_str())
                .collect::<Vec<_>>(),
            ["Usuario de la Infraestructura Tecnológica", "Cuenta"]
        );
    }
}
