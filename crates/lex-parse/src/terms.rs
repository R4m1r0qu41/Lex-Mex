//! Defined-term (glossary) extraction and exact usage linking.
//!
//! Mexican financial instruments commonly open with a glossary provision —
//! LRITF Article 4, DCG-IFPE-2021 Article 1 — though not always. Five
//! layouts are supported: `fractions` (`I. Término, a …`, continuation
//! paragraphs such as incisos belong to the preceding fraction),
//! `colon_entries` (`Término: a …`, produced by the definition-layout
//! reconstruction, with non-entry paragraphs continuing the previous
//! entry), and `roman_colon` (`I. Término: …`, scanned inline so a
//! glossary whose fractions share one paragraph still splits),
//! `mixed_fractions` (reviewed comma/colon entries), and
//! `quoted_respectively`, which reads the single sentence `…los vocablos "A",
//! "B" y "C", se entenderá que se refiere a X, a Y y a Z, respectivamente.`
//! (`reg-lan` Article 1).
//!
//! Usage linking is deterministic and case-sensitive: an occurrence matches
//! a term exactly or through one generated singular/plural variant, at word
//! boundaries, longest match first. A glossary may be additive to another
//! instrument's glossary (the DCG defines its terms "además de los términos
//! utilizados en la Ley…"), so resolution tries the instrument's own terms
//! before earlier instruments' terms.

use anyhow::{Context, Result, bail};
use lex_core::{Basis, DefinedTerm, Provision, ProvisionType, SCHEMA_VERSION, TermUsage};
use regex::Regex;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GlossaryStyle {
    MixedFractions,
    /// `I. Término, a …` — the term runs to the first comma.
    Fractions,
    /// `Término: a …` — the term runs to the first colon.
    ColonEntries,
    /// `I. Término: a …` — a Roman fraction, the term to the first colon,
    /// then the definition. Scanned inline, so a glossary whose fractions
    /// share one paragraph (no blank line between them) still splits.
    RomanColon,
    /// `…los vocablos "A", "B" y "C", se entenderá que se refiere a X, a Y y
    /// a Z, respectivamente.` — each quoted vocable is a term and the whole
    /// sentence is every term's definition span.
    QuotedRespectively,
}

impl GlossaryStyle {
    pub fn from_config(value: &str) -> Result<Self> {
        match value {
            "fractions" => Ok(Self::Fractions),
            "colon_entries" => Ok(Self::ColonEntries),
            "roman_colon" => Ok(Self::RomanColon),
            "mixed_fractions" => Ok(Self::MixedFractions),
            "quoted_respectively" => Ok(Self::QuotedRespectively),
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
        GlossaryStyle::RomanColon => roman_colon_entries(provision)?,
        GlossaryStyle::MixedFractions => mixed_fraction_entries(provision)?,
        GlossaryStyle::QuotedRespectively => quoted_respectively_entries(provision)?,
    };
    terms_from_entries(provision, entries)
}

fn terms_from_entries(
    provision: &Provision,
    entries: Vec<GlossaryEntry>,
) -> Result<Vec<DefinedTerm>> {
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

/// Auto-detect and extract a fraction glossary when an instrument does not
/// configure one. A glossary provision opens with a "…se entenderá por:"
/// (or "se entiende por:") introduction and lists Roman-fraction entries;
/// the term runs to the entry's first delimiter (`,`, `.`, or `:`, chosen
/// from the first entry), scanned inline so the entries may share one
/// paragraph or be separated. Returns an empty vector when no glossary is
/// found, so a law without defined terms simply yields none.
pub fn detect_glossary_terms(provisions: &[Provision]) -> Result<Vec<DefinedTerm>> {
    let intro = Regex::new(r"(?i)se\s+entender[áa]\s+(?:por|lo\s+siguiente)\s*:")?;
    let fraction = Regex::new(r"(?:^|[;:\n])[ \t]*([IVXLCDM]{1,7})\.\s+")?;
    for provision in provisions {
        if provision.provision_type != ProvisionType::Article {
            continue;
        }
        let head: String = provision.text.chars().take(400).collect();
        let Some(intro_match) = intro.find(&head) else {
            continue;
        };
        // Scan entries starting after the introduction.
        let body_start = intro_match.end();
        let text = &provision.text;
        let starts: Vec<(usize, String)> = fraction
            .captures_iter(text)
            .filter_map(|captures| {
                let whole = captures.get(0)?;
                let numeral = captures.get(1)?;
                (whole.start() >= body_start)
                    .then(|| (numeral.start(), numeral.as_str().to_owned()))
            })
            .collect();
        if starts.len() < 2 {
            continue;
        }
        // The delimiter is normally representative of every entry, so it is
        // established once and applied uniformly. But a repealed fraction's
        // placeholder text ("Se deroga.", "Derogada.") is a period, not the
        // glossary's real term/definition separator — deriving the
        // delimiter from `starts[0]` unconditionally meant a repealed
        // fraction 1 (period-terminated) silently starved every other,
        // colon-delimited entry of its delimiter, discarding the entire
        // glossary rather than just the placeholder. Skip past placeholder
        // entries when choosing which entry is representative.
        let Some(delimiter) = starts.iter().find_map(|(numeral_start, numeral)| {
            let delimiter = glossary_delimiter(text, *numeral_start)?;
            let after_numeral = numeral_start + numeral.len() + 1;
            let entry_body = text.get(after_numeral.min(text.len())..)?;
            let offset = entry_body
                .chars()
                .take(COLON_TERM_MAX_CHARS + 5)
                .collect::<String>()
                .find(delimiter)?;
            let term = entry_body[..offset].trim();
            (!term.is_empty() && !is_repealed_fraction_placeholder(term)).then_some(delimiter)
        }) else {
            continue;
        };
        let mut entries = Vec::with_capacity(starts.len());
        for (index, (numeral_start, numeral)) in starts.iter().enumerate() {
            let entry_end = starts.get(index + 1).map_or(text.len(), |(next, _)| *next);
            let after_numeral = numeral_start + numeral.len() + 1; // past "N."
            let entry_body = &text[after_numeral.min(text.len())..entry_end];
            let Some(delimiter_offset) = entry_body.find(delimiter) else {
                continue;
            };
            let term = entry_body[..delimiter_offset].trim().to_owned();
            if term.is_empty()
                || term.chars().count() > COLON_TERM_MAX_CHARS
                || is_repealed_fraction_placeholder(&term)
            {
                continue;
            }
            entries.push(GlossaryEntry {
                term,
                fraction: Some(numeral.clone()),
                start_char: text[..*numeral_start].chars().count(),
                end_char: text[..entry_end].chars().count(),
            });
        }
        if !entries.is_empty() {
            return terms_from_entries(provision, entries);
        }
    }
    Ok(Vec::new())
}

/// The delimiter separating term from definition in the first entry: the
/// earliest of `:` `.` `,` following the fraction numeral.
fn glossary_delimiter(text: &str, numeral_start: usize) -> Option<char> {
    let tail = &text[numeral_start..];
    // Skip the numeral's own period ("III.").
    let after_numeral = tail.find(". ").map_or(0, |index| index + 2);
    tail[after_numeral..]
        .chars()
        .take(COLON_TERM_MAX_CHARS + 5)
        .find(|character| matches!(character, ':' | '.' | ','))
}

/// A repealed glossary fraction keeps its numeral but replaces the
/// definition with a standard placeholder (`Se deroga.`, `(Se deroga)`,
/// `Derogada.`, `Derogado.`), often followed by `Fracción derogada DOF
/// ...`. Two fractions repealed years apart routinely carry the identical
/// placeholder wording, so treating it as a defined term collides two
/// unrelated fractions under one id — and a repealed fraction has no
/// definition to extract in the first place.
fn is_repealed_fraction_placeholder(term: &str) -> bool {
    let normalized = term.trim().trim_matches(['(', ')']).to_lowercase();
    matches!(normalized.as_str(), "se deroga" | "derogada" | "derogado")
}

struct GlossaryEntry {
    term: String,
    fraction: Option<String>,
    start_char: usize,
    end_char: usize,
}

/// Explicit adapter style for reviewed mixed comma/colon definitions. The
/// page-joined entry prefix is admitted only after an exact editorial
/// `(antes fracción ROMAN)` note, never an arbitrary roman prose citation.
fn mixed_fraction_entries(provision: &Provision) -> Result<Vec<GlossaryEntry>> {
    let entry_re =
        Regex::new(r"(?:^|[;\n]|\(antes fracción [IVXLCDM]+\) )[ \t]*([IVXLCDM]{1,7})\.\s+")?;
    let mut starts = Vec::new();
    for c in entry_re.captures_iter(&provision.text) {
        let n = c.get(1).expect("fraction");
        let end = c.get(0).expect("entry").end();
        starts.push((n.start(), end, n.as_str().to_owned()));
    }
    let mut result = Vec::new();
    for (index, (start, body_start, fraction)) in starts.iter().enumerate() {
        let end = starts.get(index + 1).map_or(provision.text.len(), |x| x.0);
        let body = &provision.text[*body_start..end];
        let colon = body.find(':').filter(|n| body[..*n].chars().count() <= 90);
        let comma = body.find(',');
        let comma_definition = comma.is_some_and(|n| {
            ["al ", "a ", "el ", "la "]
                .iter()
                .any(|p| body[n + 1..].trim_start().starts_with(p))
        });
        let delimiter = if comma_definition {
            comma
        } else {
            colon.or(comma)
        };
        let Some(delimiter) = delimiter else {
            bail!("mixed glossary entry {fraction} lacks a delimiter");
        };
        let term = body[..delimiter].trim().to_owned();
        result.push(GlossaryEntry {
            term,
            fraction: Some(fraction.clone()),
            start_char: provision.text[..*start].chars().count(),
            end_char: provision.text[..end].chars().count(),
        });
    }
    Ok(result)
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

/// `I. Término: definición; II. …` entries scanned inline. Each entry
/// begins at a Roman-numeral fraction that follows the intro colon or a
/// prior entry's `;`, and the term is the text up to that entry's first
/// colon (bounded, so a definition's own colon is not mistaken for the
/// term boundary). The span runs from the fraction to the next entry.
fn roman_colon_entries(provision: &Provision) -> Result<Vec<GlossaryEntry>> {
    let entry_re = Regex::new(r"(?:^|[:;])\s*([IVXLCDM]+)\.\s+([^:;]{1,90}?):\s")?;
    let text = &provision.text;
    let mut starts: Vec<(usize, String, String)> = Vec::new();
    for captures in entry_re.captures_iter(text) {
        let fraction = captures.get(1).expect("fraction capture");
        let term = captures.get(2).expect("term capture");
        starts.push((
            fraction.start(),
            captures[1].to_owned(),
            term.as_str().trim().to_owned(),
        ));
    }
    let mut entries = Vec::with_capacity(starts.len());
    for (index, (byte_start, fraction, term)) in starts.iter().enumerate() {
        let byte_end = starts
            .get(index + 1)
            .map_or(text.len(), |(next_start, _, _)| *next_start);
        entries.push(GlossaryEntry {
            term: term.clone(),
            fraction: Some(fraction.clone()),
            start_char: text[..*byte_start].chars().count(),
            end_char: text[..byte_end].chars().count(),
        });
    }
    Ok(entries)
}

/// Quoted vocables paired with their referents by `respectivamente`. The
/// sentence must list at least two quoted vocables and as many referents,
/// or extraction fails rather than guessing the pairing. Each entry's span is
/// the whole sentence, so the referents (`la Ley de Aguas Nacionales`) are
/// the definition, not usages.
fn quoted_respectively_entries(provision: &Provision) -> Result<Vec<GlossaryEntry>> {
    let sentence_re = Regex::new(
        r#"(?:vocablos|términos|expresiones|palabras)\s+(["“][^"”]+["”](?:\s*,\s*["“][^"”]+["”])*\s+y\s+["“][^"”]+["”]),?\s+se\s+entender[áa]\s+que\s+se\s+refieren?\s+([^.]+?),?\s+respectivamente\."#,
    )?;
    let quoted_re = Regex::new(r#"["“]([^"”]+)["”]"#)?;
    // Every comma and every `y` separates referents, and each referent opens
    // with its own `a`/`al` marker. A conjunction inside a referent
    // (`a la Secretaría de Educación y Cultura`) leaves a piece without a
    // marker, so the sentence is refused instead of paired by guesswork.
    let separator_re = Regex::new(r"\s*,\s+|\s+y\s+")?;
    let referent_re = Regex::new(r"^(?:al|a\s+(?:la|las|los|lo|el))\s+\S")?;
    let boundary_re = Regex::new(r"\.\s+|\n\s*\n")?;
    let text = &provision.text;
    let sentences: Vec<_> = sentence_re.captures_iter(text).collect();
    let captures = match sentences.as_slice() {
        [only] => only,
        [] => bail!(
            "glossary provision {} has no quoted `respectivamente` definition",
            provision.id
        ),
        _ => bail!(
            "glossary provision {} has {} quoted `respectivamente` definitions; this style reads exactly one",
            provision.id,
            sentences.len()
        ),
    };
    let sentence = captures.get(0).expect("sentence");
    let terms: Vec<String> = quoted_re
        .captures_iter(&captures[1])
        .map(|quoted| quoted[1].trim().to_owned())
        .collect();
    let referents = &captures[2];
    let pieces: Vec<&str> = separator_re.split(referents).collect();
    let separators: Vec<&str> = separator_re
        .find_iter(referents)
        .map(|separator| separator.as_str().trim())
        .collect();
    let paired = pieces.len() == terms.len()
        && pieces.iter().all(|piece| referent_re.is_match(piece))
        && separators.split_last().is_some_and(|(last, rest)| {
            *last == "y" && rest.iter().all(|separator| *separator == ",")
        });
    if !paired {
        bail!(
            "glossary provision {} pairs {} vocables with an ambiguous or unequal referent list",
            provision.id,
            terms.len()
        );
    }
    let sentence_start = boundary_re
        .find_iter(&text[..sentence.start()])
        .last()
        .map_or(0, |boundary| boundary.end());
    let start_char = text[..sentence_start].chars().count();
    let end_char = text[..sentence.end()].chars().count();
    Ok(terms
        .into_iter()
        .map(|term| GlossaryEntry {
            term,
            fraction: None,
            start_char,
            end_char,
        })
        .collect())
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

/// Extract usages written exactly as a glossary prints its vocables: the
/// term, case-sensitive and without singular/plural variants, between a
/// matched pair of straight (`"Ley"`) or curly (`“Ley”`) quotes. The usage
/// span is the term inside the quotes. An adapter opts in with
/// `usage_form: "quoted"` (`reg-lan`, whose Article 1 defines its vocables
/// in quotes), so a bare word, or the first word of another title such as
/// `Ley Federal de Derechos`, is not a usage. `term_sets` priority and the
/// own-definition exclusion are as in [`extract_term_usages`].
pub fn extract_quoted_term_usages(
    provisions: &[Provision],
    term_sets: &[&[DefinedTerm]],
) -> Result<Vec<TermUsage>> {
    let mut candidates: Vec<&DefinedTerm> = term_sets.iter().flat_map(|set| set.iter()).collect();
    // Longest terms first; ties keep `term_sets` priority (stable sort).
    candidates.sort_by_key(|term| std::cmp::Reverse(term.term.chars().count()));

    let mut usages = Vec::new();
    for provision in provisions {
        let chars: Vec<char> = provision.text.chars().collect();
        let mut position = 0;
        while position < chars.len() {
            let close = match chars[position] {
                '"' => '"',
                '“' => '”',
                _ => {
                    position += 1;
                    continue;
                }
            };
            let start = position + 1;
            let matched = candidates.iter().find(|term| {
                let end = start + term.term.chars().count();
                matches_at(&chars, start, &term.term)
                    && chars.get(end) == Some(&close)
                    && !inside_own_definition(term, provision, start)
            });
            let Some(term) = matched else {
                position += 1;
                continue;
            };
            let end_char = start + term.term.chars().count();
            usages.push(TermUsage {
                schema_version: SCHEMA_VERSION.to_owned(),
                id: format!("{}:term-usage:{start}-{end_char}", provision.id),
                provision_id: provision.id.clone(),
                term_id: term.id.clone(),
                span: term.term.clone(),
                start_char: start,
                end_char,
            });
            // Resume after the closing quote, so it never opens a new match.
            position = end_char + 1;
        }
    }
    Ok(usages)
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

    use super::{
        GlossaryStyle, detect_glossary_terms, extract_quoted_term_usages, extract_term_usages,
        extract_terms,
    };

    const STATUTE_ID: &str = "urn:lex-mx:test:statute";
    const REGULATION_ID: &str = "urn:lex-mx:test:regulation";

    fn provision(instrument_id: &str, suffix: &str, text: &str) -> Provision {
        Provision {
            numbering_basis: None,
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
            publication_date: Some(chrono::NaiveDate::from_ymd_opt(2021, 1, 28).unwrap()),
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
    fn extracts_roman_colon_terms_from_a_single_merged_paragraph() {
        // The whole glossary is one paragraph: no blank line separates the
        // fractions, so the entries must be split inline.
        let glossary = provision(
            REGULATION_ID,
            "article:5",
            "Para los efectos de la presente Ley, se entenderá por: \
             I. Cotizante: la persona física o moral que confirma su cotización \
             en el procedimiento de adjudicación directa; \
             II. Dependencias: las señaladas en las fracciones I y II del artículo 1 \
             de esta Ley; \
             III. Investigación de mercado: el proceso previo al inicio de los \
             procedimientos de contratación;",
        );
        let terms = extract_terms(&glossary, GlossaryStyle::RomanColon).unwrap();
        assert_eq!(
            terms
                .iter()
                .map(|term| (term.term.as_str(), term.fraction.as_deref().unwrap()))
                .collect::<Vec<_>>(),
            [
                ("Cotizante", "I"),
                ("Dependencias", "II"),
                ("Investigación de mercado", "III"),
            ]
        );
        // A definition's own "fracciones I y II" does not open a new entry,
        // and each span contains its term.
        let chars: Vec<char> = glossary.text.chars().collect();
        for term in &terms {
            let span: String = chars[term.start_char..term.end_char].iter().collect();
            assert!(
                span.contains(&term.term),
                "span must contain {:?}",
                term.term
            );
        }
    }

    #[test]
    fn auto_detects_a_period_delimited_fraction_glossary() {
        // No adapter config; the term runs to the first period, and the
        // Constitution reference in the last definition is not a new entry.
        let glossary = provision(
            REGULATION_ID,
            "article:2",
            "Para efectos de la presente Ley se entenderá por:\n\n\
             I. Dieta correcta. Aquella que es completa, equilibrada y suficiente;\n\n\
             II. Normas. A las normas oficiales mexicanas;\n\n\
             III. Secretaría. A la Secretaría del Trabajo y Previsión Social;\n\n\
             IV. Trabajadores. A quienes prestan un trabajo comprendido en el \
             apartado A del artículo 123 de la Constitución.",
        );
        let terms = detect_glossary_terms(std::slice::from_ref(&glossary)).unwrap();
        assert_eq!(
            terms
                .iter()
                .map(|term| term.term.as_str())
                .collect::<Vec<_>>(),
            ["Dieta correcta", "Normas", "Secretaría", "Trabajadores"]
        );
        let chars: Vec<char> = glossary.text.chars().collect();
        for term in &terms {
            let span: String = chars[term.start_char..term.end_char].iter().collect();
            assert!(span.contains(&term.term));
        }
    }

    #[test]
    fn repealed_fraction_placeholders_are_skipped_not_collided() {
        // A repealed glossary fraction keeps its numeral but replaces the
        // definition with a standard placeholder ("Se deroga.", "(Se
        // deroga)", "Derogada.", "Derogado."), often followed by "Fracción
        // derogada DOF ...". Two fractions repealed years apart routinely
        // carry the identical placeholder wording -- confirmed against
        // reg-lfprh's real committed source, whose article 2 has both
        // fraction I and fraction XIV read "Se deroga." -- so treating the
        // placeholder as the defined term collided two unrelated fractions
        // under one id (`duplicate defined-term identifier`). A repealed
        // fraction has no definition to extract in the first place.
        let glossary = provision(
            REGULATION_ID,
            "article:2",
            "Se entenderá por:\n\n\
             I. Se deroga. Fracción derogada DOF 08-08-2025\n\n\
             II. Aportaciones federales: los recursos transferidos a las entidades.\n\n\
             III. Derogada. Fracción derogada DOF 25-04-2014\n\n\
             XIV. Se deroga. Fracción derogada DOF 04-09-2009",
        );
        let terms = detect_glossary_terms(std::slice::from_ref(&glossary)).unwrap();
        assert_eq!(
            terms
                .iter()
                .map(|term| (term.term.as_str(), term.fraction.as_deref().unwrap()))
                .collect::<Vec<_>>(),
            [("Aportaciones federales", "II")]
        );
    }

    #[test]
    fn auto_detection_yields_nothing_without_a_glossary_intro() {
        let ordinary = provision(
            REGULATION_ID,
            "article:1",
            "Las disposiciones de esta Ley son de orden público. I. No es una \
             entrada de glosario porque no hay introducción.",
        );
        assert!(
            detect_glossary_terms(std::slice::from_ref(&ordinary))
                .unwrap()
                .is_empty()
        );
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

    #[test]
    fn quoted_respectively_terms_share_the_defining_sentence() {
        // reg-lan Article 1, verbatim.
        let glossary = provision(
            REGULATION_ID,
            "article:1",
            "El presente ordenamiento tiene por objeto reglamentar la Ley de Aguas \
             Nacionales. Cuando en el mismo se expresen los vocablos \"Ley\", \
             \"Reglamento\", \"La Comisión\" y \"Registro\", se entenderá que se \
             refiere a la Ley de Aguas Nacionales, al presente Reglamento, a la \
             Comisión Nacional del Agua y al Registro Público de Derechos de Agua, \
             respectivamente.",
        );
        let terms = extract_terms(&glossary, GlossaryStyle::QuotedRespectively).unwrap();
        assert_eq!(
            terms
                .iter()
                .map(|term| (term.term.as_str(), term.id.rsplit(':').next().unwrap()))
                .collect::<Vec<_>>(),
            [
                ("Ley", "ley"),
                ("Reglamento", "reglamento"),
                ("La Comisión", "la-comision"),
                ("Registro", "registro"),
            ]
        );
        let chars: Vec<char> = glossary.text.chars().collect();
        let span: String = chars[terms[0].start_char..terms[0].end_char]
            .iter()
            .collect();
        assert!(span.starts_with("Cuando en el mismo"));
        assert!(span.ends_with("respectivamente."));
        assert!(terms.iter().all(|term| term.fraction.is_none()
            && (term.start_char, term.end_char) == (terms[0].start_char, terms[0].end_char)));

        // The parent law named in the first sentence is a usage; the
        // referents inside the defining sentence are not.
        let usages = extract_term_usages(std::slice::from_ref(&glossary), &[&terms]).unwrap();
        assert_eq!(
            usages
                .iter()
                .map(|usage| (usage.span.as_str(), usage.start_char < terms[0].start_char))
                .collect::<Vec<_>>(),
            [("Ley", true)]
        );
    }

    #[test]
    fn quoted_respectively_refuses_an_unpaired_sentence() {
        let glossary = provision(
            REGULATION_ID,
            "article:1",
            "Cuando se expresen los vocablos \"Ley\" y \"Reglamento\", se entenderá \
             que se refiere a la Ley de Aguas Nacionales, respectivamente.",
        );
        assert!(extract_terms(&glossary, GlossaryStyle::QuotedRespectively).is_err());
        let prose = provision(REGULATION_ID, "article:1", "Sin definiciones.");
        assert!(extract_terms(&prose, GlossaryStyle::QuotedRespectively).is_err());
    }

    #[test]
    fn quoted_usage_form_matches_only_the_quoted_vocable() {
        let glossary = provision(
            REGULATION_ID,
            "article:1",
            "El presente ordenamiento tiene por objeto reglamentar la Ley de Aguas \
             Nacionales. Cuando en el mismo se expresen los vocablos \"Ley\", \
             \"Reglamento\", \"La Comisión\" y \"Registro\", se entenderá que se \
             refiere a la Ley de Aguas Nacionales, al presente Reglamento, a la \
             Comisión Nacional del Agua y al Registro Público de Derechos de Agua, \
             respectivamente.",
        );
        let terms = extract_terms(&glossary, GlossaryStyle::QuotedRespectively).unwrap();
        let body = provision(
            REGULATION_ID,
            "article:2",
            "\"La Comisión\" aplicará la \"Ley\" y el “Reglamento”. La Ley, la Ley \
             Federal de Derechos, las \"Leyes\", la \"ley\", el \"Registro” y el \
             “Registro\" no cuentan; la Comisión tampoco.",
        );
        let usages =
            extract_quoted_term_usages(&[glossary.clone(), body.clone()], &[&terms]).unwrap();
        // Article 1's own quoted vocables are the definition, not usages.
        assert!(usages.iter().all(|usage| usage.provision_id == body.id));
        assert_eq!(
            usages
                .iter()
                .map(|usage| usage.span.as_str())
                .collect::<Vec<_>>(),
            ["La Comisión", "Ley", "Reglamento"]
        );
        let chars: Vec<char> = body.text.chars().collect();
        for usage in &usages {
            let span: String = chars[usage.start_char..usage.end_char].iter().collect();
            assert_eq!(span, usage.span);
            assert!(matches!(chars[usage.start_char - 1], '"' | '“'));
            assert!(matches!(chars[usage.end_char], '"' | '”'));
        }
    }

    #[test]
    fn quoted_respectively_refuses_a_conjunction_inside_a_referent() {
        // Two referents, one of which contains `y`: counting `y` as a
        // separator would wrongly pair three vocables.
        let glossary = provision(
            REGULATION_ID,
            "article:1",
            "Los vocablos \"A\", \"B\" y \"C\", se entenderá que se refiere al \
             Registro, a la Secretaría de Educación y Cultura, respectivamente.",
        );
        let error = extract_terms(&glossary, GlossaryStyle::QuotedRespectively).unwrap_err();
        assert!(error.to_string().contains("ambiguous"), "{error}");
    }

    #[test]
    fn quoted_respectively_refuses_a_second_definition_sentence() {
        let glossary = provision(
            REGULATION_ID,
            "article:1",
            "Los vocablos \"A\" y \"B\", se entenderá que se refiere al X y a la Y, \
             respectivamente. Los vocablos \"C\" y \"D\", se entenderá que se \
             refiere al Z y a la W, respectivamente.",
        );
        let error = extract_terms(&glossary, GlossaryStyle::QuotedRespectively).unwrap_err();
        assert!(error.to_string().contains("has 2 quoted"), "{error}");
    }

    #[test]
    fn quoted_respectively_span_starts_after_a_newline_boundary_and_allows_line_breaks() {
        let glossary = provision(
            REGULATION_ID,
            "article:1",
            "Primera oración.\nCuando se expresen los vocablos \"A\", \"B\" y\n\"C\", \
             se entenderá que se refiere al X,\na la Y y\na la Z, respectivamente.",
        );
        let terms = extract_terms(&glossary, GlossaryStyle::QuotedRespectively).unwrap();
        assert_eq!(
            terms
                .iter()
                .map(|term| term.term.as_str())
                .collect::<Vec<_>>(),
            ["A", "B", "C"]
        );
        let chars: Vec<char> = glossary.text.chars().collect();
        let span: String = chars[terms[0].start_char..terms[0].end_char]
            .iter()
            .collect();
        assert!(span.starts_with("Cuando se expresen"), "{span:?}");
        assert!(span.ends_with("respectivamente."), "{span:?}");
    }
    #[test]
    fn reviewed_mixed_glossary_preserves_internal_commas_and_page_joined_entry() {
        let opts = crate::DiputadosOptions {
            instrument_id: "urn:lex-mx:federal:statute:lmigra".to_owned(),
            header_lines: vec!["LEY DE MIGRACIÓN".to_owned()],
            stop_markers: Vec::new(),
            annex_markers: Vec::new(),
            separate_enactment: false,
            nested_instrument: None,
        };
        let doc = crate::parse_diputados(
            include_str!("../../../fixtures/diputados/followup-lmigra-glossary.txt"),
            &opts,
            chrono::NaiveDate::from_ymd_opt(2011, 5, 25).unwrap(),
        )
        .unwrap();
        let terms = extract_terms(&doc.provisions[0], GlossaryStyle::MixedFractions).unwrap();
        let child_terms: Vec<_> = terms
            .iter()
            .filter(|t| t.term.starts_with("Niña,"))
            .collect();
        assert_eq!(child_terms.len(), 4);
        assert_eq!(
            terms
                .iter()
                .filter(|t| t.term == "Autoridad migratoria")
                .count(),
            1
        );
        assert!(
            child_terms
                .iter()
                .any(|t| t.term == "Niña, niño o adolescente migrante separado")
        );
        assert_eq!(
            terms.len(),
            terms
                .iter()
                .map(|t| &t.id)
                .collect::<std::collections::HashSet<_>>()
                .len()
        );
        let generic = super::detect_glossary_terms(&doc.provisions).unwrap();
        assert!(
            generic.iter().filter(|t| t.term == "Niña").count() > 1,
            "generic inference must not silently change"
        );
        let p = provision(
            "urn:lex-mx:sample",
            "article:3",
            "I. Nombre, al funcionario indicado.\n\nII. Niña, niño o adolescente: persona menor.\n\nUna cita XVI. No debe crear: otra definición.",
        );
        let terms = extract_terms(&p, GlossaryStyle::MixedFractions).unwrap();
        assert_eq!(terms.len(), 2);
        assert_eq!(terms[0].term, "Nombre");
    }
}
