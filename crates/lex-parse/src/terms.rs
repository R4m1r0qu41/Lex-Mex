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
//! (`reg-lan` Article 1). Two further styles apply an auto-detection
//! grammar to one named provision: `auto_established` (the established
//! `se entenderá por:` pass) and `auto_whole` (the wider pass, accepted only
//! whole). An instrument may configure several glossary provisions, each
//! defining a different set of terms (`lss` Articles 5 A and 159).
//!
//! Usage linking is deterministic and case-sensitive: an occurrence matches
//! a term exactly or through one generated singular/plural variant, at word
//! boundaries, longest match first. A glossary may be additive to another
//! instrument's glossary (the DCG defines its terms "además de los términos
//! utilizados en la Ley…"), so resolution tries the instrument's own terms
//! before earlier instruments' terms.

use std::sync::LazyLock;

use anyhow::{Context, Result, bail};
use lex_core::{
    Basis, DefinedTerm, FRACTION_QUALIFIER_PATTERN, GLOSSARY_STRATEGY_VERSION,
    GlossaryEntryDisposition, GlossaryHoldCode, GlossaryProfile, GlossarySpan, Provision,
    ProvisionType, SCHEMA_VERSION, TermUsage, evidence_sha256, fraction_qualifier_rank,
    normalize_fraction_label,
};
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
    /// The established auto-detection grammar (`…se entenderá por:` and
    /// Roman fractions, delimiter taken from the first entry) applied to
    /// the configured provision, exactly as [`detect_glossary`] applies it.
    AutoEstablished,
    /// The wider auto-detection grammar applied to the configured
    /// provision. The glossary is accepted only whole; an incomplete one is
    /// an error, not a partial set.
    AutoWhole,
}

impl GlossaryStyle {
    pub fn from_config(value: &str) -> Result<Self> {
        match value {
            "fractions" => Ok(Self::Fractions),
            "colon_entries" => Ok(Self::ColonEntries),
            "roman_colon" => Ok(Self::RomanColon),
            "mixed_fractions" => Ok(Self::MixedFractions),
            "quoted_respectively" => Ok(Self::QuotedRespectively),
            "auto_established" => Ok(Self::AutoEstablished),
            "auto_whole" => Ok(Self::AutoWhole),
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
        GlossaryStyle::AutoEstablished => {
            let terms = established_glossary_terms(provision)?;
            if terms.is_empty() {
                bail!("no glossary detected at {}", provision.id);
            }
            return Ok(terms);
        }
        GlossaryStyle::AutoWhole => match wider_glossary_entries(provision) {
            Some(Ok(glossary)) => glossary.entries,
            Some(Err((_, reason))) => {
                bail!("glossary at {} is incomplete: {reason}", provision.id)
            }
            None => bail!("no glossary detected at {}", provision.id),
        },
    };
    terms_from_entries(provision, entries)
}

/// The repealed fractions a configured style skipped as whole placeholders,
/// when the style reports them (`auto_whole`); `None` for the other styles.
#[must_use]
pub fn repeal_placeholders(
    provision: &Provision,
    style: GlossaryStyle,
    profile: Option<&GlossaryProfile>,
) -> Option<Vec<String>> {
    if style != GlossaryStyle::AutoWhole {
        return None;
    }
    match profile {
        Some(profile) => profiled_glossary_entries(provision, profile).ok(),
        None => wider_glossary_entries(provision).and_then(std::result::Result::ok),
    }
    .map(|glossary| glossary.repeal_placeholders)
}

/// The fractions a configured `auto_established` glossary found but could
/// not read a term from; `None` for the other styles.
pub fn unread_fractions(
    provision: &Provision,
    style: GlossaryStyle,
) -> Result<Option<Vec<String>>> {
    if style != GlossaryStyle::AutoEstablished {
        return Ok(None);
    }
    Ok(Some(established_glossary(provision)?.unread_fractions))
}

/// Extract an instrument's configured glossary provisions, in order, each
/// with its style and, for `auto_whole`, an optional reviewed profile. Each
/// provision defines a different set of terms and the sets add to each
/// other, so a term defined in two of them is an error rather than a
/// precedence choice.
pub fn extract_glossaries(
    glossaries: &[(&Provision, GlossaryStyle, Option<&GlossaryProfile>)],
) -> Result<Vec<DefinedTerm>> {
    let mut terms: Vec<DefinedTerm> = Vec::new();
    for (provision, style, profile) in glossaries {
        let extracted = match profile {
            Some(profile) => {
                if *style != GlossaryStyle::AutoWhole {
                    bail!(
                        "glossary profile at {} requires style auto_whole",
                        provision.id
                    );
                }
                terms_from_entries(
                    provision,
                    profiled_glossary_entries(provision, profile)?.entries,
                )?
            }
            None => extract_terms(provision, *style)?,
        };
        for term in &extracted {
            if let Some(earlier) = terms.iter().find(|earlier| earlier.id == term.id) {
                bail!(
                    "term {} is defined in both {} and {}",
                    term.id,
                    earlier.defining_provision_id,
                    term.defining_provision_id
                );
            }
        }
        terms.extend(extracted);
    }
    Ok(terms)
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
///
/// When this finds nothing, the wider fallback pass of [`detect_glossary`]
/// runs; a glossary that pass holds back yields no terms here.
pub fn detect_glossary_terms(provisions: &[Provision]) -> Result<Vec<DefinedTerm>> {
    Ok(detect_glossary(provisions)?.terms)
}

/// Outcome of glossary auto-detection.
#[derive(Debug, Default)]
pub struct GlossaryDetection {
    pub terms: Vec<DefinedTerm>,
    /// The pass that produced `terms`: `established_pass` or `wider_pass`.
    pub method: Option<&'static str>,
    /// Repealed fractions the wider pass skipped as whole placeholders; the
    /// established pass does not report them.
    pub repeal_placeholders: Option<Vec<String>>,
    /// Fractions the established pass found but could not read a term
    /// from; the wider pass holds such a glossary instead.
    pub unread_fractions: Option<Vec<String>>,
    /// Set when the wider pass found a glossary it could not extract
    /// completely. Such a glossary yields no terms rather than a partial set.
    pub held: Option<GlossaryHold>,
}

/// A detected glossary held as incomplete: where, and why.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GlossaryHold {
    pub provision_id: String,
    pub code: GlossaryHoldCode,
    pub message: String,
}

/// Why the wider grammar could not extract a glossary whole.
type Incomplete = (GlossaryHoldCode, String);

/// A wider-pass glossary read whole.
struct WiderGlossary {
    entries: Vec<GlossaryEntry>,
    repeal_placeholders: Vec<String>,
}

/// Auto-detect a glossary: first the established `se entenderá por:` pass,
/// unchanged, then (only when it finds nothing) the wider pass for `se
/// entiende por:`, inserted phrases (`se entiende, en singular o plural,
/// por:`) and introductions that adopt another instrument's definitions
/// (`…las definiciones contenidas en el artículo 3 de la Ley, así como las
/// siguientes:`). Running the wider pass only as a fallback keeps every
/// glossary the established pass already finds byte for byte.
pub fn detect_glossary(provisions: &[Provision]) -> Result<GlossaryDetection> {
    for provision in provisions {
        if provision.provision_type != ProvisionType::Article {
            continue;
        }
        let glossary = established_glossary(provision)?;
        if !glossary.terms.is_empty() {
            return Ok(GlossaryDetection {
                terms: glossary.terms,
                method: Some("established_pass"),
                unread_fractions: Some(glossary.unread_fractions),
                ..GlossaryDetection::default()
            });
        }
    }
    detect_wider_glossary(provisions)
}

static ESTABLISHED_INTRO: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)se\s+entender[áa]\s+(?:por|lo\s+siguiente)\s*:")
        .expect("established intro regex must compile")
});

static ESTABLISHED_FRACTION: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(&format!(
        r"(?:^|[;:\n]|DOF\s+\d{{2}}-\d{{2}}-\d{{4}}[.,]?)[ \t]*([IVXLCDM]{{1,7}}(?:\.?\s+{FRACTION_QUALIFIER_PATTERN})?)\.\s+"
    ))
    .expect("established fraction regex must compile")
});

/// The established pass over one provision; empty when it finds no glossary.
fn established_glossary_terms(provision: &Provision) -> Result<Vec<DefinedTerm>> {
    Ok(established_glossary(provision)?.terms)
}

/// An established-pass glossary: its terms, and the fractions it found but
/// could not read a term from. Those are recorded rather than dropped, so a
/// partial glossary is never reported as a complete one.
struct EstablishedGlossary {
    terms: Vec<DefinedTerm>,
    unread_fractions: Vec<String>,
}

fn established_glossary(provision: &Provision) -> Result<EstablishedGlossary> {
    let none = || EstablishedGlossary {
        terms: Vec::new(),
        unread_fractions: Vec::new(),
    };
    let head: String = provision.text.chars().take(400).collect();
    let Some(intro_match) = ESTABLISHED_INTRO.find(&head) else {
        return Ok(none());
    };
    // Scan entries starting after the introduction.
    let body_start = intro_match.end();
    let text = &provision.text;
    let starts: Vec<(usize, String)> = ESTABLISHED_FRACTION
        .captures_iter(text)
        .filter_map(|captures| {
            let whole = captures.get(0)?;
            let numeral = captures.get(1)?;
            (whole.start() >= body_start).then(|| (numeral.start(), numeral.as_str().to_owned()))
        })
        .collect();
    if starts.len() < 2 {
        return Ok(none());
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
        return Ok(none());
    };
    let mut entries = Vec::with_capacity(starts.len());
    let mut unread_fractions = Vec::new();
    for (index, (numeral_start, numeral)) in starts.iter().enumerate() {
        let entry_end = starts.get(index + 1).map_or(text.len(), |(next, _)| *next);
        let after_numeral = numeral_start + numeral.len() + 1; // past "N."
        let entry_body = &text[after_numeral.min(text.len())..entry_end];
        let glossary_term = entry_body
            .find(delimiter)
            .map(|offset| entry_body[..offset].trim())
            .filter(|term| !term.is_empty() && term.chars().count() <= COLON_TERM_MAX_CHARS);
        if glossary_term.is_some_and(is_repealed_fraction_placeholder)
            || starts_with_repeal_wording(entry_body)
        {
            continue;
        }
        // An entry the glossary's delimiter does not fit is read by its own
        // (`Auditor Interno, al responsable…` in a colon glossary); one that
        // fits neither is recorded as unread.
        let Some(term) = glossary_term.or_else(|| own_delimiter_term(entry_body)) else {
            unread_fractions.push(normalize_fraction_label(numeral));
            continue;
        };
        entries.push(GlossaryEntry {
            term: term.to_owned(),
            fraction: Some(normalize_fraction_label(numeral)),
            start_char: text[..*numeral_start].chars().count(),
            end_char: text[..entry_end].chars().count(),
        });
    }
    if entries.is_empty() {
        return Ok(none());
    }
    Ok(EstablishedGlossary {
        terms: terms_from_entries(provision, entries)?,
        unread_fractions,
    })
}

/// An entry's term read by its own delimiter, for an entry the glossary's
/// delimiter does not fit: the text before the first `:`, before a `,` that
/// a definition opener follows (`, al`, `, a la`, `, en singular`), or
/// before a `.` that a capital follows. The term must be a plain short name
/// that does not end in a connector (`Familiares derechohabientes a:` opens
/// a list, not a definition).
fn own_delimiter_term(body: &str) -> Option<&str> {
    static OPENER: LazyLock<Regex> = LazyLock::new(|| {
        Regex::new(r"^\s+(?:al?|el|las?|los|en|para)\b").expect("opener regex must compile")
    });
    static CAPITAL: LazyLock<Regex> =
        LazyLock::new(|| Regex::new(r"^\s+[A-ZÁÉÍÓÚÑ]").expect("capital regex must compile"));
    let (offset, _) =
        body.char_indices()
            .take(COLON_TERM_MAX_CHARS + 1)
            .find(|(offset, character)| match character {
                ':' => true,
                ',' => OPENER.is_match(&body[offset + 1..]),
                '.' => CAPITAL.is_match(&body[offset + 1..]),
                _ => false,
            })?;
    let term = body[..offset].trim();
    let last_word = term.rsplit(' ').next().unwrap_or_default();
    (!term.is_empty() && !term.contains([';', '\n', '"', '“', '”']) && !is_connector(last_word))
        .then_some(term)
}

/// The wider fallback pass. A glossary is accepted only whole: its fractions
/// run I, II, III… without a gap (`Bis`, `Ter`… may follow their base), every
/// fraction that is not a repeal placeholder yields a term before the
/// glossary's delimiter, no fraction swallows another fraction heading (a
/// running page header between two fractions does), and no two terms share
/// an identifier. Otherwise the glossary is held with its reason. Chapter-,
/// title- or article-scoped introductions are skipped: their terms do not
/// govern the whole instrument.
fn detect_wider_glossary(provisions: &[Provision]) -> Result<GlossaryDetection> {
    for provision in provisions {
        if provision.provision_type != ProvisionType::Article {
            continue;
        }
        let Some(outcome) = wider_glossary_entries(provision) else {
            continue;
        };
        return Ok(match outcome {
            Ok(glossary) => GlossaryDetection {
                terms: terms_from_entries(provision, glossary.entries)?,
                method: Some("wider_pass"),
                repeal_placeholders: Some(glossary.repeal_placeholders),
                unread_fractions: None,
                held: None,
            },
            Err((code, message)) => GlossaryDetection {
                held: Some(GlossaryHold {
                    provision_id: provision.id.clone(),
                    code,
                    message,
                }),
                ..GlossaryDetection::default()
            },
        });
    }
    Ok(GlossaryDetection::default())
}

static WIDER_DEFINES: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"(?i)\bse\s+(?:entiende|entienden|entender[áa]n?|deben?\s+entender|deber[áa]n?\s+entender)(?:\s*,\s*en\s+singular\s+o\s+plural)?\s*,?\s*(?:por|como)\s*:",
    )
    .expect("wider defines regex must compile")
});

static WIDER_ADOPTS: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"(?i)\bdefiniciones\b[^:]{0,320}?,?\s*(?:así\s+como|y,?)\s+(?:a\s+|en\s+)?(?:l[ao]s\s+)?(?:siguientes(?:\s+definiciones)?|definiciones\s+siguientes)\s*:",
    )
    .expect("wider adopts regex must compile")
});

static WIDER_LISTS: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"(?i)\bse\s+(?:aplican|observar[áa]n)\s+las\s+siguientes\s+definiciones\s*:|\bse\s+considera\s*:",
    )
    .expect("wider lists regex must compile")
});

static WIDER_SCOPED: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"(?i)efectos?\s+(?:de|del)\s+(?:este|esta|el\s+presente|la\s+presente|presente|dicho|dicha)\s+(?:capítulo|título|sección|apartado|artículo|párrafo)|artículo\s+anterior",
    )
    .expect("wider scoped regex must compile")
});

// A fraction starts the text or follows `;`, `:`, `.`, a line break, the
// conjunction `y` (the last entry of an inline glossary: `…de Economía, y
// X.- Servicio: …`) or an amendment note (`…Fracción recorrida DOF
// 28-01-2011 XX.- Impacto ambiental: …`). A spurious start breaks the
// numbering and holds the glossary rather than splitting it wrongly.
static WIDER_FRACTION: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(&format!(
        r"(?:^|[;:.\n]|\by|DOF\s+\d{{2}}-\d{{2}}-\d{{4}})[ \t]*([IVXLCDM]{{1,7}}(?:\.?\s+{FRACTION_QUALIFIER_PATTERN})?)\.-?\s+",
    ))
    .expect("wider fraction regex must compile")
});

/// The wider pass over one provision: `None` when it opens no glossary that
/// governs the whole instrument, otherwise every entry or the reason the
/// glossary is incomplete.
fn wider_glossary_entries(
    provision: &Provision,
) -> Option<std::result::Result<WiderGlossary, Incomplete>> {
    let starts = wider_fraction_starts(provision)?;
    Some(complete_entries(
        &provision.text,
        &starts,
        &std::collections::HashMap::new(),
    ))
}

/// Each fraction candidate of a wider-pass glossary: the numeral's byte
/// offset, its body's byte offset and its printed label. `None` when the
/// provision opens no glossary that governs the whole instrument.
fn wider_fraction_starts(provision: &Provision) -> Option<Vec<(usize, usize, String)>> {
    let text = &provision.text;
    let head_end = text
        .char_indices()
        .nth(500)
        .map_or(text.len(), |(index, _)| index);
    let head = &text[..head_end];
    let intro = WIDER_DEFINES
        .find(head)
        .or_else(|| WIDER_ADOPTS.find(head))
        .or_else(|| WIDER_LISTS.find(head))?;
    if WIDER_SCOPED.is_match(&head[..intro.end()]) {
        return None;
    }
    let body_start = intro.end();
    let starts: Vec<(usize, usize, String)> = WIDER_FRACTION
        .captures_iter(&text[body_start..])
        .map(|captures| {
            let numeral = captures.get(1).expect("numeral");
            (
                body_start + numeral.start(),
                body_start + captures.get(0).expect("fraction").end(),
                normalize_fraction_label(numeral.as_str()),
            )
        })
        .collect();
    (starts.len() >= 2).then_some(starts)
}

/// How a profile reads one fraction instead of the shared term rule.
enum EntryHint<'a> {
    Terms(Vec<&'a str>),
    NonDefinition,
}

/// Read an `auto_whole` glossary under its reviewed profile. The profile
/// must be bound to this exact text and strategy, and every hint must match
/// the text literally at its offset and be used; anything else fails rather
/// than falling back. The shared completeness checks still run on every
/// fraction the profile does not name.
fn profiled_glossary_entries(
    provision: &Provision,
    profile: &GlossaryProfile,
) -> Result<WiderGlossary> {
    let text = &provision.text;
    if profile.strategy_version != GLOSSARY_STRATEGY_VERSION {
        bail!(
            "glossary profile at {} targets strategy {}, not {GLOSSARY_STRATEGY_VERSION}",
            provision.id,
            profile.strategy_version
        );
    }
    if evidence_sha256(text) != profile.provision_text_sha256 {
        bail!(
            "glossary profile at {} was reviewed against different text",
            provision.id
        );
    }
    let byte_at = |span: &GlossarySpan| -> Result<usize> {
        let start = text
            .char_indices()
            .map(|(index, _)| index)
            .nth(span.start_char)
            .filter(|start| !span.text.is_empty() && text[*start..].starts_with(&span.text));
        start.with_context(|| {
            format!(
                "glossary profile span {:?} is not at character {} of {}",
                span.text, span.start_char, provision.id
            )
        })
    };
    let mut starts = wider_fraction_starts(provision)
        .with_context(|| format!("no glossary detected at {}", provision.id))?;
    for candidate in &profile.ignored_candidates {
        let at = byte_at(candidate)?;
        let index = starts
            .iter()
            .position(|(numeral_start, _, _)| *numeral_start == at)
            .with_context(|| {
                format!(
                    "ignored candidate {:?} at character {} is not a fraction candidate",
                    candidate.text, candidate.start_char
                )
            })?;
        starts.remove(index);
    }
    let mut hints = std::collections::HashMap::new();
    for entry in &profile.entries {
        let matching: Vec<usize> = (0..starts.len())
            .filter(|index| starts[*index].2 == entry.fraction)
            .collect();
        let [index] = matching[..] else {
            bail!(
                "glossary profile fraction {} does not name exactly one fraction of {}",
                entry.fraction,
                provision.id
            );
        };
        let body_start = starts[index].1;
        let end = starts.get(index + 1).map_or(text.len(), |next| next.0);
        let hint = match entry.disposition {
            GlossaryEntryDisposition::NonDefinition if entry.terms.is_empty() => {
                EntryHint::NonDefinition
            }
            GlossaryEntryDisposition::Definition if !entry.terms.is_empty() => {
                let terms = entry
                    .terms
                    .iter()
                    .map(|span| {
                        let at = byte_at(span)?;
                        if at < body_start
                            || at + span.text.len() > end
                            || span.text.trim() != span.text
                            || span.text.contains('\n')
                        {
                            bail!(
                                "profile term {:?} is not a plain span inside fraction {}",
                                span.text,
                                entry.fraction
                            );
                        }
                        Ok(span.text.as_str())
                    })
                    .collect::<Result<Vec<_>>>()?;
                EntryHint::Terms(terms)
            }
            _ => bail!(
                "glossary profile fraction {}: a definition lists its terms and a non-definition none",
                entry.fraction
            ),
        };
        if hints.insert(index, hint).is_some() {
            bail!("glossary profile names fraction {} twice", entry.fraction);
        }
    }
    complete_entries(text, &starts, &hints).map_err(|(_, reason)| {
        anyhow::anyhow!(
            "glossary at {} is incomplete under its profile: {reason}",
            provision.id
        )
    })
}

/// Each fraction's parsing key is its base value and qualifier rank (none,
/// `Bis`, `Ter`, `Quáter`…); the label, its qualifier in title case, is kept
/// as the entry's fraction. An unqualified fraction follows the previous base; a qualified one repeats
/// the base with the next qualifier, so `I, I Bis, I Bis` and `I, I Ter`
/// both break the sequence.
fn check_fraction_sequence(
    starts: &[(usize, usize, String)],
) -> std::result::Result<(), Incomplete> {
    let mut previous: Option<(u32, u32)> = None;
    let mut previous_label: Option<&str> = None;
    for (_, _, numeral) in starts {
        let (base, rank) = match numeral.split_once(char::is_whitespace) {
            Some((base, qualifier)) => match fraction_qualifier_rank(qualifier) {
                Some(rank) => (base, rank),
                None => {
                    return Err((
                        GlossaryHoldCode::UnknownQualifier,
                        format!("fraction {numeral} has an unknown qualifier"),
                    ));
                }
            },
            None => (numeral.as_str(), 0),
        };
        let value = roman_value(base)
            .filter(|value| canonical_roman(*value) == base)
            .ok_or_else(|| {
                (
                    GlossaryHoldCode::NoncanonicalNumeral,
                    format!("fraction {numeral} is not a canonical Roman numeral"),
                )
            })?;
        let expected = if rank == 0 {
            previous.map_or(1, |(value, _)| value + 1)
        } else {
            previous.map_or(0, |(value, _)| value)
        };
        let expected_rank = if rank == 0 {
            0
        } else {
            previous.map_or(0, |(_, rank)| rank + 1)
        };
        if value != expected || rank != expected_rank {
            return Err((
                GlossaryHoldCode::SequenceBreak,
                format!(
                    "fraction {numeral} breaks the I, II, III… sequence after {}",
                    previous_label.unwrap_or("the introduction")
                ),
            ));
        }
        previous = Some((value, rank));
        previous_label = Some(numeral);
    }
    Ok(())
}

/// A fraction heading inside an entry: furniture hid a fraction.
static SWALLOWED_FRACTION: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(&format!(
        r"(?:^|\s)[IVXLCDM]{{1,7}}(?:\.?\s+{FRACTION_QUALIFIER_PATTERN})?\.-?\s+[A-ZÁÉÍÓÚÑ]"
    ))
    .expect("swallowed fraction regex must compile")
});

fn no_term_delimiter() -> Incomplete {
    (
        GlossaryHoldCode::NoTermDelimiter,
        "no fraction has a term delimiter".to_owned(),
    )
}

/// Every entry of a wider-pass glossary, or the reason it is incomplete.
/// A profile's hints read the fractions they name instead of the shared
/// term rule.
fn complete_entries(
    text: &str,
    starts: &[(usize, usize, String)],
    hints: &std::collections::HashMap<usize, EntryHint>,
) -> std::result::Result<WiderGlossary, Incomplete> {
    check_fraction_sequence(starts)?;
    let bodies: Vec<(&str, &str, usize, usize)> = starts
        .iter()
        .enumerate()
        .map(|(index, (numeral_start, body_start, numeral))| {
            let end = starts
                .get(index + 1)
                .map_or(text.len(), |(next, _, _)| *next);
            (
                numeral.as_str(),
                &text[*body_start..end],
                *numeral_start,
                end,
            )
        })
        .collect();
    let last = bodies.len().saturating_sub(1);
    let placeholder = |index: usize, body: &str| {
        // The last entry runs to the end of the article, so its closing
        // paragraphs follow the placeholder; any other placeholder must be
        // the whole entry.
        let entry = if index == last {
            body.split("\n\n").next().unwrap_or(body)
        } else {
            body
        };
        REPEAL_PLACEHOLDER.is_match(entry.trim())
    };
    let delimiter = bodies
        .iter()
        .enumerate()
        .filter(|(index, (_, body, _, _))| !placeholder(*index, body) && !hints.contains_key(index))
        .map(|(_, entry)| entry)
        .find_map(|(_, body, _, _)| {
            body.chars()
                .take(COLON_TERM_MAX_CHARS + 5)
                .find(|character| matches!(character, ':' | '.' | ','))
        });
    if delimiter.is_none() && hints.is_empty() {
        return Err(no_term_delimiter());
    }
    let mut entries = Vec::with_capacity(bodies.len());
    let mut repeal_placeholders = Vec::new();
    let mut slugs = std::collections::HashSet::new();
    for (index, (numeral, body, start, end)) in bodies.into_iter().enumerate() {
        // A fraction heading inside any entry, a placeholder included, means
        // furniture hid a fraction (`II. Derogada. ENCABEZADO III. …`).
        if let Some(found) = SWALLOWED_FRACTION.find(body) {
            return Err((
                GlossaryHoldCode::SwallowedFraction,
                format!(
                    "fraction {numeral} contains another fraction heading {:?}",
                    found.as_str().trim()
                ),
            ));
        }
        let mut push_term = |term: &str| {
            if !slugs.insert(slug(term)) {
                return Err((
                    GlossaryHoldCode::RepeatedTerm,
                    format!("fraction {numeral} repeats the term {term:?}"),
                ));
            }
            entries.push(GlossaryEntry {
                term: term.to_owned(),
                fraction: Some(numeral.to_owned()),
                start_char: text[..start].chars().count(),
                end_char: text[..end].chars().count(),
            });
            Ok(())
        };
        match hints.get(&index) {
            Some(EntryHint::NonDefinition) => continue,
            Some(EntryHint::Terms(terms)) => {
                for term in terms {
                    push_term(term)?;
                }
                continue;
            }
            None => {}
        }
        if placeholder(index, body) {
            repeal_placeholders.push(numeral.to_owned());
            continue;
        }
        // Only a whole placeholder is a repealed fraction. An entry that
        // opens with the repeal wording and goes on (`Derogados: Los
        // permisos retirados`) is not read as either.
        if starts_with_repeal_wording(body) {
            return Err((
                GlossaryHoldCode::RepealWording,
                format!(
                    "fraction {numeral} opens with repeal wording but is not a repeal placeholder"
                ),
            ));
        }
        push_term(entry_term(
            numeral,
            body,
            delimiter.ok_or_else(no_term_delimiter)?,
        )?)?;
    }
    Ok(WiderGlossary {
        entries,
        repeal_placeholders,
    })
}

/// The plain term before an entry's delimiter, or why the entry has none.
fn entry_term<'a>(
    numeral: &str,
    body: &'a str,
    delimiter: char,
) -> std::result::Result<&'a str, Incomplete> {
    let term = body
        .find(delimiter)
        .map(|offset| body[..offset].trim())
        .filter(|term| !term.is_empty() && term.chars().count() <= COLON_TERM_MAX_CHARS)
        .ok_or_else(|| {
            (
                GlossaryHoldCode::MissingTerm,
                format!("fraction {numeral} has no term before `{delimiter}`"),
            )
        })?;
    // A term is a short name on one line. Punctuation inside it means
    // the fraction is not `Term<delimiter> definition`: a nested list
    // (`Tabacos labrados y otros: … a) Cigarros, …`) or a quoted name
    // that needs the quoted style.
    if term.contains([':', ';', '.', '\n', '"', '“', '”']) {
        return Err((
            GlossaryHoldCode::UnplainTerm,
            format!("fraction {numeral} has no plain term: {term:?}"),
        ));
    }
    Ok(term)
}

/// A repealed fraction's whole entry: `Derogada.`, `(Se deroga);` or `Se
/// deroga.`, optionally followed by its amendment notes (`Fracción derogada
/// DOF 12-01-2024`, `Fracción adicionada DOF … . Derogada DOF …`) and the
/// conjunction that introduces an inline glossary's last entry.
static REPEAL_PLACEHOLDER: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"(?i)^\(?\s*(?:derogad[ao]s?|se\s+deroga)\s*\)?\s*[.;,]?(?:\s*(?:fracci[óo]n\s+)?(?:derogad[ao]|adicionad[ao]|reformad[ao]|recorrid[ao])\s+(?:DOF\s+)?\d{2}-\d{2}-\d{4}(?:\s*(?:,|y)\s*\d{2}-\d{2}-\d{4})*\s*[.;,]?)*(?:\s+y)?$",
    )
    .expect("repeal placeholder regex must compile")
});

fn starts_with_repeal_wording(body: &str) -> bool {
    let normalized = body.trim_start().trim_start_matches('(').to_lowercase();
    normalized.starts_with("derogad") || normalized.starts_with("se deroga")
}

fn canonical_roman(mut value: u32) -> String {
    let mut numeral = String::new();
    for (step, symbol) in [
        (1000, "M"),
        (900, "CM"),
        (500, "D"),
        (400, "CD"),
        (100, "C"),
        (90, "XC"),
        (50, "L"),
        (40, "XL"),
        (10, "X"),
        (9, "IX"),
        (5, "V"),
        (4, "IV"),
        (1, "I"),
    ] {
        while value >= step {
            numeral.push_str(symbol);
            value -= step;
        }
    }
    numeral
}

fn roman_value(numeral: &str) -> Option<u32> {
    let mut total = 0;
    let mut previous = 0;
    for character in numeral.chars().rev() {
        let value = match character {
            'I' => 1,
            'V' => 5,
            'X' => 10,
            'L' => 50,
            'C' => 100,
            'D' => 500,
            'M' => 1000,
            _ => return None,
        };
        if value < previous {
            total -= value;
        } else {
            total += value;
            previous = value;
        }
    }
    Some(total)
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
    let entry_re = Regex::new(&format!(
        r"(?:^|[;\n]|\(antes fracción [IVXLCDM]+\) )[ \t]*([IVXLCDM]{{1,7}}(?:\.?\s+{FRACTION_QUALIFIER_PATTERN})?)\.\s+"
    ))?;
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
            fraction: Some(normalize_fraction_label(fraction)),
            start_char: provision.text[..*start].chars().count(),
            end_char: provision.text[..end].chars().count(),
        });
    }
    Ok(result)
}

fn fraction_entries(provision: &Provision) -> Result<Vec<GlossaryEntry>> {
    let fraction_re = Regex::new(&format!(
        r"^([IVXLCDM]+(?:\.?\s+{FRACTION_QUALIFIER_PATTERN})?)\.\s+(.*)$"
    ))?;
    let mut entries: Vec<GlossaryEntry> = Vec::new();
    for (start_char, paragraph) in paragraphs_with_offsets(&provision.text) {
        let end_char = start_char + paragraph.chars().count();
        if let Some(captures) = fraction_re.captures(paragraph) {
            let body = captures.get(2).expect("body capture").as_str();
            let term = body.split(',').next().unwrap_or("").trim().to_owned();
            entries.push(GlossaryEntry {
                term,
                fraction: Some(normalize_fraction_label(&captures[1])),
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
    let entry_re = Regex::new(&format!(
        r"(?:^|[:;])\s*([IVXLCDM]+(?:\.?\s+{FRACTION_QUALIFIER_PATTERN})?)\.\s+([^:;]{{1,90}}?):\s"
    ))?;
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
            fraction: Some(normalize_fraction_label(fraction)),
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
        GlossaryHoldCode, HeadingContext, Provision, ProvisionType, ReviewStatus, SCHEMA_VERSION,
        TemporalStatus,
    };
    use pretty_assertions::assert_eq;

    use super::{
        GlossaryStyle, detect_glossary_terms, extract_glossaries, extract_quoted_term_usages,
        extract_term_usages, extract_terms,
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

    fn wider(text: &str) -> super::GlossaryDetection {
        super::detect_glossary(&[provision(STATUTE_ID, "article:3", text)]).unwrap()
    }

    fn wider_terms(text: &str) -> Vec<String> {
        let detection = wider(text);
        assert_eq!(detection.held, None);
        detection.terms.into_iter().map(|term| term.term).collect()
    }

    #[test]
    fn wider_pass_reads_se_entiende_por_with_dash_numerals() {
        // lgeepa 3o, verbatim opening (definitions shortened).
        assert_eq!(
            wider_terms(
                "Para los efectos de esta Ley se entiende por:\n\nI.- Ambiente: El conjunto \
                 de elementos naturales y artificiales;\n\nII.- Áreas naturales protegidas: \
                 Las zonas del territorio nacional;"
            ),
            ["Ambiente", "Áreas naturales protegidas"]
        );
    }

    #[test]
    fn wider_pass_reads_inserted_phrases_and_inline_fractions() {
        // lsh 5: the fractions share one paragraph.
        assert_eq!(
            wider_terms(
                "Para los efectos de esta Ley se entiende, en singular o plural, por: I. \
                 Agencia: Agencia Nacional de Seguridad Industrial; II. Almacenamiento: \
                 Depósito y resguardo de Hidrocarburos;"
            ),
            ["Agencia", "Almacenamiento"]
        );
        // lbogm 3 and lgeepa 3o shapes: an entry ends with a period or with an
        // amendment note, on the same line as the next fraction.
        assert_eq!(
            wider_terms(
                "Para los efectos de esta Ley, se entiende por: I. Actividades: Las \
                 actividades de liberación. II. Bioseguridad: Las acciones; Fracción recorrida \
                 DOF 28-01-2011 III.- Impacto ambiental: Modificación del ambiente;"
            ),
            ["Actividades", "Bioseguridad", "Impacto ambiental"]
        );
        // reg-lfppi 2 shape: the last inline entry follows `, y`.
        assert_eq!(
            wider_terms(
                "Para efectos de este Reglamento se entiende por: I.- Ley: la Ley Federal; \
                 II.- Secretaría: la Secretaría de Economía, y III.- Servicio: el servicio \
                 público."
            ),
            ["Ley", "Secretaría", "Servicio"]
        );
        // lotfja 2: `se entenderá, por:`.
        assert_eq!(
            wider_terms(
                "Para efectos de esta Ley se entenderá, por:\n\nI. Junta: La Junta de \
                 Gobierno y Administración;\n\nII. Ley: La Ley Orgánica del Tribunal \
                 Federal de Justicia Administrativa;"
            ),
            ["Junta", "Ley"]
        );
    }

    #[test]
    fn wider_pass_reads_introductions_that_adopt_other_definitions() {
        // lcne 6 (`y, las siguientes:`) and a regulation importing its parent's.
        assert_eq!(
            wider_terms(
                "Para efectos de la interpretación y aplicación de la presente Ley, se deben \
                 entender los conceptos y las definiciones, en singular o plural, previstos \
                 en la Ley del Sector Hidrocarburos, la Ley del Sector Eléctrico, la Ley de \
                 Planeación y Transición Energética y, las siguientes:\n\nI. Actividades en \
                 materia energética: Las actividades relacionadas con el sector eléctrico;\n\n\
                 II. Beneficio: La ventaja obtenida;"
            ),
            ["Actividades en materia energética", "Beneficio"]
        );
        assert_eq!(
            wider_terms(
                "Para los efectos del presente Reglamento, serán aplicables las definiciones \
                 establecidas en el artículo 5 A de la Ley del Seguro Social, así como las \
                 siguientes:\n\nI. Consejo Técnico: El Consejo Técnico del Instituto;\n\nII. \
                 Derogada.\n\nIII. Reserva General: La reserva prevista;"
            ),
            ["Consejo Técnico", "Reserva General"]
        );
    }

    #[test]
    fn wider_pass_reads_the_remaining_introduction_forms() {
        let entries = "\n\nI. Agencia: La Agencia;\n\nII. Banco: El Banco;";
        for intro in [
            // reg-lpeam 4
            "Para efectos del presente reglamento, además de las definiciones previstas en el \
             artículo 5 de la Ley, se debe entender por:",
            // reg-laam 5
            "Para efectos de lo establecido en el presente Reglamento, se entenderá como:",
            // lgeo 2
            "Para efectos de la interpretación y aplicación de la presente Ley, se deben \
             entender los conceptos y las definiciones, en singular o plural, previstos en la \
             Ley de Planeación y Transición Energética, la Ley del Sector Eléctrico y en las \
             siguientes definiciones:",
            // reg-lpte 5
            "Para efectos de la interpretación y aplicación del presente Reglamento se deben \
             entender los conceptos y las definiciones, en singular o plural, previstas en los \
             artículos 3 de la Ley de Planeación y Transición Energética, así como las \
             definiciones siguientes:",
            // reg-lgeepa-anp 3
            "Para los efectos de este Reglamento se estará a las definiciones que se contienen \
             en la Ley General del Equilibrio Ecológico y la Protección al Ambiente, así como a \
             las siguientes:",
            // reg-lic-calidad 2
            "Para efectos de este Reglamento, además de las establecidas en la Ley de \
             Infraestructura de la Calidad, se aplican las siguientes definiciones:",
            // lgn 2
            "Para efectos de la presente Ley se observarán las siguientes definiciones:",
            // ladua 2
            "Para los efectos de esta Ley se considera:",
        ] {
            assert_eq!(
                wider_terms(&format!("{intro}{entries}")),
                ["Agencia", "Banco"],
                "{intro}"
            );
        }
    }

    #[test]
    fn wider_pass_holds_an_incomplete_glossary_instead_of_extracting_part() {
        // reg-lgcc-rne 2: a running page header precedes fraction VI, so V
        // would swallow VI.
        let leaked = wider(
            "Para los efectos del presente Reglamento, se considerarán las definiciones \
             contenidas en el artículo 3 de la Ley, así como las siguientes:\n\nI. Cédula: \
             Instrumento de reporte;\n\nII. Emisiones Indirectas: Son los Gases; REGLAMENTO DE \
             LA LEY GENERAL DE CAMBIO CLIMÁTICO III. Establecimiento Sujeto a Reporte: El \
             conjunto de Fuentes;",
        );
        assert!(leaked.terms.is_empty());
        let hold = leaked.held.unwrap();
        assert!(hold.provision_id.ends_with("article:3"));
        assert_eq!(hold.code, GlossaryHoldCode::SwallowedFraction);
        assert!(
            hold.message.contains("contains another fraction heading"),
            "{}",
            hold.message
        );

        // reg-lss-maeba 2 shape: III is followed by V.
        // lieps 3o shape: fraction II opens a nested list instead of a
        // definition.
        let nested = wider(
            "Para los efectos de esta Ley se entiende por:\n\nI. Cerveza, la bebida \
             fermentada.\n\nII. Tabacos labrados y otros: Párrafo reformado DOF \
             07-11-2025\n\na) Cigarros, los cigarros con o sin filtro.",
        );
        assert!(nested.terms.is_empty());
        assert!(nested.held.unwrap().message.contains("no plain term"));

        // lsct 2 shape: quoted names.
        let quoted = wider(
            "Para los efectos de la presente Ley se entenderá por:\n\nI.- “Tratado”: el \
             convenio regido por el derecho internacional público;\n\nII.- “Reserva”: la \
             declaración formulada al firmar.",
        );
        assert!(quoted.terms.is_empty());
        assert!(quoted.held.unwrap().message.contains("no plain term"));

        let gap = wider(
            "Para los efectos de esta Ley, se entiende por:\n\nI. Adquirente: Persona;\n\nII. \
             Avalúo: Dictamen;\n\nIII. Bien: Mueble;\n\nV. Comité Técnico: Órgano;",
        );
        assert!(gap.terms.is_empty());
        assert!(gap.held.unwrap().message.contains("breaks the I, II, III"));
    }

    #[test]
    fn wider_pass_holds_repeal_wording_that_is_not_a_whole_placeholder() {
        // Codex review 2026-10-18, P1: furniture after a placeholder hid
        // fraction III, and the whole glossary was accepted as `Agencia`.
        let hidden = wider(
            "Para los efectos de esta Ley se entiende por:\nI. Agencia: La agencia;\nII. \
             Derogada. ENCABEZADO DE PAGINA III. Secretaría: La secretaría.",
        );
        assert!(hidden.terms.is_empty());
        let reason = hidden.held.unwrap().message;
        assert!(reason.contains("another fraction heading"), "{reason}");

        // A genuine entry that opens with the repeal wording is not dropped.
        let genuine = wider(
            "Para los efectos de esta Ley se entiende por:\nI. Derogados: Los permisos \
             retirados;\nII. Vigentes: Los permisos activos.",
        );
        assert!(genuine.terms.is_empty());
        let reason = genuine.held.unwrap().message;
        assert!(reason.contains("repeal wording"), "{reason}");

        // Whole placeholders, with their amendment notes, are still skipped.
        assert_eq!(
            wider_terms(
                "Para los efectos de esta Ley, se entiende por:\n\nI. Agencia: La agencia;\n\n\
                 II. Derogada. Fracción adicionada DOF 12-01-2015. Derogada DOF 01-02-2020\n\n\
                 III. (Se deroga); Fracción derogada DOF 03-04-2021\n\nIV. Se deroga \
                 Fracción reformada DOF 05-06-2010, 07-08-2012. Derogada DOF 09-10-2019\n\nV. \
                 Banco: El banco.\n\nVI. Derogada.\n\nArtículo reformado DOF 11-12-2022",
            ),
            ["Agencia", "Banco"]
        );
    }

    #[test]
    fn detection_reports_its_pass_skipped_placeholders_and_hold_code() {
        let detected = wider(
            "Para los efectos de esta Ley, se entiende por:\n\nI. Agencia: La agencia;\n\nII. \
             Derogada. Fracción derogada DOF 01-02-2003\n\nIII. Banco: El banco.",
        );
        assert_eq!(detected.method, Some("wider_pass"));
        assert_eq!(detected.repeal_placeholders, Some(vec!["II".to_owned()]));
        assert_eq!(
            super::repeal_placeholders(
                &provision(
                    STATUTE_ID,
                    "article:3",
                    "Para los efectos de esta Ley, se entiende \
                 por:\n\nI. Agencia: La agencia;\n\nII. Derogada.\n\nIII. Banco: El banco."
                ),
                GlossaryStyle::AutoWhole,
                None
            ),
            Some(vec!["II".to_owned()])
        );

        let established = wider(
            "Para los efectos de esta Ley se entenderá por:\n\nI. Agencia: La agencia;\n\nII. \
             Banco: El banco.",
        );
        assert_eq!(established.method, Some("established_pass"));
        assert_eq!(established.repeal_placeholders, None);

        for (fractions, code) in [
            (
                "I. Agencia: La agencia;\n\nIII. Banco: El banco.",
                GlossaryHoldCode::SequenceBreak,
            ),
            (
                "I. Agencia: La agencia;\n\nII. Banco: El banco;\n\nIIII. Caja: La caja.",
                GlossaryHoldCode::NoncanonicalNumeral,
            ),
            (
                "I. Agencia: La agencia;\n\nII. Agencia: Otra agencia.",
                GlossaryHoldCode::RepeatedTerm,
            ),
            (
                "I. Derogados: Los permisos;\n\nII. Vigentes: Los activos.",
                GlossaryHoldCode::RepealWording,
            ),
        ] {
            let held = wider(&format!(
                "Para los efectos de esta Ley, se entiende por:\n\n{fractions}"
            ));
            assert!(held.terms.is_empty(), "{fractions}");
            assert_eq!(held.method, None, "{fractions}");
            assert_eq!(held.held.map(|hold| hold.code), Some(code), "{fractions}");
        }
    }

    #[test]
    fn wider_pass_skips_every_surveyed_repeal_placeholder_shape() {
        // The 13 placeholder shapes in committed glossary provisions
        // (2026-10-18 survey), verbatim but for the dates.
        for placeholder in [
            "Derogada. Fracción derogada DOF 01-02-2003",
            "(Se deroga); Fracción derogada DOF 01-02-2003",
            "Se deroga. Fracción derogada DOF 01-02-2003",
            "Derogada.",
            "Se deroga. Fracción adicionada DOF 01-02-2003. Derogada DOF 04-05-2006",
            "Derogada. Fracción adicionada DOF 01-02-2003. Derogada DOF 04-05-2006",
            "Derogada Fracción derogada DOF 01-02-2003",
            "Se deroga Fracción reformada DOF 01-02-2003. Derogada DOF 04-05-2006",
            "(Se deroga). Fracción derogada DOF 01-02-2003",
            "(Se deroga) Fracción derogada DOF 01-02-2003",
            "Derogada; Fracción derogada DOF 01-02-2003",
            "Derogada. Fracción reformada DOF 01-02-2003. Derogada DOF 04-05-2006",
            "Se deroga. Fracción recorrida DOF 01-02-2003. Derogada DOF 04-05-2006",
        ] {
            assert_eq!(
                wider_terms(&format!(
                    "Para los efectos de esta Ley, se entiende por:\n\nI. Agencia: La \
                     agencia;\n\nII. {placeholder}\n\nIII. Banco: El banco."
                )),
                ["Agencia", "Banco"],
                "{placeholder}"
            );
        }
    }

    #[test]
    fn wider_pass_checks_qualified_fraction_order_and_spelling() {
        let glossary = |fractions: &str| {
            wider(&format!(
                "Para los efectos de esta Ley, se entiende por:\n\n{fractions}"
            ))
        };
        // Codex review 2026-10-18, P2: a repeated `Bis` with a new name.
        let repeated = glossary(
            "I. Agencia: La agencia;\n\nI Bis. Banco: El banco;\n\nI Bis. Caja: La \
             caja;\n\nII. Deuda: La deuda.",
        );
        assert!(repeated.terms.is_empty());
        assert!(repeated.held.unwrap().message.contains("I Bis breaks"));

        let ter_first = glossary("I. Agencia: La agencia;\n\nI Ter. Banco: El banco;");
        assert!(ter_first.held.unwrap().message.contains("I Ter breaks"));

        let spelled = glossary(
            "I. Agencia: La agencia;\n\nII. Banco: El banco;\n\nIII. Caja: La \
             caja;\n\nIIII. Deuda: La deuda.",
        );
        assert!(
            spelled
                .held
                .unwrap()
                .message
                .contains("not a canonical Roman numeral")
        );

        assert_eq!(
            wider_terms(
                "Para los efectos de esta Ley, se entiende por:\n\nI. Agencia: La \
                 agencia;\n\nI Bis. Banco: El banco;\n\nI Ter. Caja: La caja;\n\nII. \
                 Deuda: La deuda;\n\nIII. Ente: El ente.",
            ),
            ["Agencia", "Banco", "Caja", "Deuda", "Ente"]
        );
    }

    #[test]
    fn wider_pass_skips_scoped_and_enumerating_introductions() {
        for text in [
            // cff 16: a single definition enumerated, not a glossary.
            "Se entenderá por actividades empresariales las siguientes:\n\nI. Las comerciales \
             que son las que tienen ese carácter.\n\nII. Las industriales entendidas como la \
             extracción.",
            // lisr 142
            "Se entiende que, entre otros, son ingresos en los términos de este Capítulo los \
             siguientes:\n\nI. El importe de las deudas perdonadas.\n\nII. La ganancia cambiaria.",
            // lasoc 12
            "Se entienden como servicios básicos de salud en materia de asistencia social los \
             siguientes:\n\nI. Los señalados en el Artículo 168:\n\nII. Los demás.",
            // cpf 401: chapter-scoped.
            "Para los efectos de este Capítulo, se entiende por:\n\nI. Servidores Públicos, las \
             personas;\n\nII. Funcionarios electorales, quienes integren los órganos;",
            // ccom 1416: title-scoped.
            "Para los efectos del presente título se entenderá por:\n\nI.- Acuerdo de arbitraje, \
             el acuerdo;\n\nII.- Arbitraje, cualquier procedimiento arbitral;",
            // ladua 137 bis 2: article-scoped.
            "Para efecto del artículo anterior y de los siguientes, se entiende por:\n\nI.- \
             Persona Física: El ciudadano.\n\nII.- Franja Fronteriza Norte: La comprendida.",
        ] {
            let detection = wider(text);
            assert!(detection.terms.is_empty(), "{text}");
            assert_eq!(detection.held, None, "{text}");
        }
    }

    #[test]
    fn the_established_pass_wins_over_the_wider_pass() {
        let wider_first = provision(
            STATUTE_ID,
            "article:2",
            "Para los efectos de esta Ley se entiende por:\n\nI. Agencia: La Agencia;\n\nII. \
             Banco: El Banco;",
        );
        let established = provision(
            STATUTE_ID,
            "article:9",
            "Para efectos de esta Ley se entenderá por:\n\nI. Cliente: La persona;\n\nII. \
             Operación: El acto;",
        );
        let detection = super::detect_glossary(&[wider_first, established]).unwrap();
        assert_eq!(
            detection
                .terms
                .iter()
                .map(|term| term.term.as_str())
                .collect::<Vec<_>>(),
            ["Cliente", "Operación"]
        );
    }

    fn lss_5_a() -> Provision {
        provision(
            STATUTE_ID,
            "article:5-a",
            "Para los efectos de esta Ley, se entiende por:\n\nI. Ley: la Ley del Seguro \
             Social;\n\nII. Código: el Código Fiscal de la Federación;\n\nIII. Patrones, \
             patrón o persona empleadora: la persona física o moral; Fracción reformada DOF \
             16-11-2022\n\nIV. Trabajador permanente: aquél que tenga una relación de trabajo \
             por tiempo indeterminado;",
        )
    }

    fn lss_159() -> Provision {
        provision(
            STATUTE_ID,
            "article:159",
            "Para efectos de esta Ley, se entenderá por:\n\nI. Cuenta individual, aquella que \
             se abrirá para cada asegurado.\n\nRespecto de la subcuenta de vivienda.\n\nII. \
             Individualizar, el proceso mediante el cual se identifica la parte.\n\nIII. \
             Pensión, la renta vitalicia o el retiro programado.",
        )
    }

    fn term_records(terms: &[lex_core::DefinedTerm]) -> String {
        format!("{terms:?}")
    }

    #[test]
    fn configured_auto_styles_apply_one_grammar_to_a_named_provision() {
        // The established grammar gives exactly what auto-detection gives.
        let established = extract_terms(&lss_159(), GlossaryStyle::AutoEstablished).unwrap();
        assert_eq!(
            term_records(&established),
            term_records(&detect_glossary_terms(&[lss_159()]).unwrap())
        );
        // The wider grammar reads fractions after amendment notes.
        let whole = extract_terms(&lss_5_a(), GlossaryStyle::AutoWhole).unwrap();
        assert_eq!(
            whole
                .iter()
                .map(|term| term.term.as_str())
                .collect::<Vec<_>>(),
            [
                "Ley",
                "Código",
                "Patrones, patrón o persona empleadora",
                "Trabajador permanente"
            ]
        );

        // A configured glossary is never partial, and a provision without
        // the grammar fails instead of yielding no terms.
        let gap = provision(
            STATUTE_ID,
            "article:3",
            "Para los efectos de esta Ley, se entiende por:\n\nI. Adquirente: Persona;\n\nII. \
             Avalúo: Dictamen;\n\nIV. Comité Técnico: Órgano;",
        );
        let error = extract_terms(&gap, GlossaryStyle::AutoWhole)
            .unwrap_err()
            .to_string();
        assert!(error.contains("is incomplete"), "{error}");
        let error = extract_terms(&lss_5_a(), GlossaryStyle::AutoEstablished)
            .unwrap_err()
            .to_string();
        assert!(error.contains("no glossary detected"), "{error}");
    }

    #[test]
    fn configured_glossaries_add_their_term_sets_and_refuse_a_shared_term() {
        let (glossary, add_on) = (lss_5_a(), lss_159());
        let terms = extract_glossaries(&[
            (&glossary, GlossaryStyle::AutoWhole, None),
            (&add_on, GlossaryStyle::AutoEstablished, None),
        ])
        .unwrap();
        assert_eq!(
            terms
                .iter()
                .map(|term| (term.term.as_str(), term.defining_provision_id.as_str()))
                .collect::<Vec<_>>(),
            [
                ("Ley", "urn:lex-mx:test:statute:article:5-a"),
                ("Código", "urn:lex-mx:test:statute:article:5-a"),
                (
                    "Patrones, patrón o persona empleadora",
                    "urn:lex-mx:test:statute:article:5-a"
                ),
                (
                    "Trabajador permanente",
                    "urn:lex-mx:test:statute:article:5-a"
                ),
                ("Cuenta individual", "urn:lex-mx:test:statute:article:159"),
                ("Individualizar", "urn:lex-mx:test:statute:article:159"),
                ("Pensión", "urn:lex-mx:test:statute:article:159"),
            ]
        );

        let redefines = provision(
            STATUTE_ID,
            "article:200",
            "Para efectos de esta Ley, se entenderá por:\n\nI. Ley, la Ley del Seguro \
             Social.\n\nII. Pensión, la renta.",
        );
        let error = extract_glossaries(&[
            (&glossary, GlossaryStyle::AutoWhole, None),
            (&redefines, GlossaryStyle::AutoEstablished, None),
        ])
        .unwrap_err()
        .to_string();
        assert!(error.contains("is defined in both"), "{error}");
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
            separatorless_article_headings: Vec::new(),
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

    use lex_core::{
        GlossaryEntryDisposition::{Definition, NonDefinition},
        GlossaryProfile, GlossaryProfileEntry, GlossarySpan,
    };

    /// A profile bound to `text`.
    fn profile(
        text: &str,
        entries: Vec<GlossaryProfileEntry>,
        ignored_candidates: Vec<GlossarySpan>,
    ) -> GlossaryProfile {
        GlossaryProfile {
            strategy_version: lex_core::GLOSSARY_STRATEGY_VERSION.to_owned(),
            provision_text_sha256: lex_core::evidence_sha256(text),
            rationale: "test".to_owned(),
            evidence: "test".to_owned(),
            entries,
            ignored_candidates,
        }
    }

    /// `literal` at its offset inside the unique `anchor`.
    fn span(text: &str, anchor: &str, literal: &str) -> GlossarySpan {
        assert_eq!(text.matches(anchor).count(), 1, "{anchor}");
        let byte = text.find(anchor).unwrap() + anchor.find(literal).unwrap();
        GlossarySpan {
            text: literal.to_owned(),
            start_char: text[..byte].chars().count(),
        }
    }

    fn entry(
        fraction: &str,
        disposition: lex_core::GlossaryEntryDisposition,
        terms: Vec<GlossarySpan>,
    ) -> GlossaryProfileEntry {
        GlossaryProfileEntry {
            fraction: fraction.to_owned(),
            disposition,
            terms,
        }
    }

    fn profiled(text: &str, profile: &GlossaryProfile) -> anyhow::Result<Vec<String>> {
        let glossary = provision(STATUTE_ID, "article:2", text);
        Ok(
            extract_glossaries(&[(&glossary, GlossaryStyle::AutoWhole, Some(profile))])?
                .into_iter()
                .map(|term| format!("{} {}", term.fraction.unwrap(), term.term))
                .collect(),
        )
    }

    // lfvv 2 VII: `Secretaría;` where every other fraction uses a colon.
    const SEMICOLON: &str = "Para los efectos de esta ley se entenderá por:\n\nI.- Comité: El \
                             Comité Calificador; II.- Secretaría; La Secretaría de \
                             Agricultura.\n\nIII.- Registro: El Registro Nacional.";

    fn semicolon_entry() -> GlossaryProfileEntry {
        entry(
            "II",
            Definition,
            vec![span(SEMICOLON, "II.- Secretaría;", "Secretaría")],
        )
    }

    #[test]
    fn profile_term_span_reads_a_fraction_the_shared_rule_holds() {
        assert_eq!(
            wider(SEMICOLON).held.unwrap().code,
            GlossaryHoldCode::MissingTerm
        );
        let hint = profile(SEMICOLON, vec![semicolon_entry()], Vec::new());
        assert_eq!(
            profiled(SEMICOLON, &hint).unwrap(),
            ["I Comité", "II Secretaría", "III Registro"]
        );
    }

    #[test]
    fn profile_fails_closed_on_any_drift_or_unused_hint() {
        let hint = |entries| profile(SEMICOLON, entries, Vec::new());
        // One changed character in the bound text.
        let drifted = SEMICOLON.replace("Nacional.", "Nacional;");
        let error = profiled(&drifted, &hint(vec![semicolon_entry()])).unwrap_err();
        assert!(error.to_string().contains("different text"), "{error}");
        // A span whose literal is not at its offset.
        let mut shifted = semicolon_entry();
        shifted.terms[0].start_char = 3;
        assert!(profiled(SEMICOLON, &hint(vec![shifted])).is_err());
        // A span inside a different fraction than the one it names.
        let mut misplaced = semicolon_entry();
        misplaced.fraction = "III".to_owned();
        assert!(profiled(SEMICOLON, &hint(vec![misplaced])).is_err());
        // A fraction the text does not have.
        let mut absent = semicolon_entry();
        absent.fraction = "IV".to_owned();
        assert!(profiled(SEMICOLON, &hint(vec![absent])).is_err());
        // The same fraction named twice.
        assert!(profiled(SEMICOLON, &hint(vec![semicolon_entry(), semicolon_entry()])).is_err());
        // A definition without terms, and a non-definition with them.
        assert!(profiled(SEMICOLON, &hint(vec![entry("II", Definition, Vec::new())])).is_err());
        let mut with_terms = semicolon_entry();
        with_terms.disposition = NonDefinition;
        assert!(profiled(SEMICOLON, &hint(vec![with_terms])).is_err());
        // An ignored candidate that is not a fraction candidate.
        let unused = profile(
            SEMICOLON,
            vec![semicolon_entry()],
            vec![span(SEMICOLON, "El Registro", "Registro")],
        );
        assert!(profiled(SEMICOLON, &unused).is_err());
        // A profile on any style other than auto_whole.
        let glossary = provision(STATUTE_ID, "article:2", SEMICOLON);
        let bound = hint(vec![semicolon_entry()]);
        assert!(
            extract_glossaries(&[(&glossary, GlossaryStyle::RomanColon, Some(&bound))]).is_err()
        );
    }

    #[test]
    fn profile_keeps_the_shared_checks_on_every_other_fraction() {
        // lfca 2: `A. C. y` reads as fraction C. Ignoring that exact
        // candidate releases the glossary; it does not relax the sequence,
        // so a genuinely missing fraction still holds.
        let abbreviation = "Para los efectos de esta Ley se entenderá por: I. Centro: el Centro \
                            de Capacitación A. C. y Estudios S. A., y II. Temporada: el conjunto.";
        assert_eq!(
            wider(abbreviation).held.unwrap().code,
            GlossaryHoldCode::SequenceBreak
        );
        let ignore = |text: &str| profile(text, Vec::new(), vec![span(text, "A. C. y", "C.")]);
        assert_eq!(
            profiled(abbreviation, &ignore(abbreviation)).unwrap(),
            ["I Centro", "II Temporada"]
        );
        let gap = abbreviation.replace("II. Temporada", "III. Temporada");
        let error = profiled(&gap, &ignore(&gap)).unwrap_err();
        assert!(
            error.to_string().contains("breaks the I, II, III"),
            "{error}"
        );
    }

    #[test]
    fn profile_reads_non_definitions_and_several_terms_per_fraction() {
        // lfra 2 V–VI and lsct 2 V.
        let text = "Para los efectos de la presente Ley se entenderá por:\n\nI.- “Tratado”: el \
                    convenio;\n\nII.- “Ratificación”, “adhesión” o “aceptación”: el acto;\n\nIII. \
                    Se entiende por cadena causal la secuencia;\n\nIV. No se considerará que existe \
                    un daño;\n\nV.- Fondo: El Fondo.";
        let hint = profile(
            text,
            vec![
                entry("I", Definition, vec![span(text, "“Tratado”", "Tratado")]),
                entry(
                    "II",
                    Definition,
                    vec![
                        span(text, "“Ratificación”", "Ratificación"),
                        span(text, "“adhesión”", "adhesión"),
                        span(text, "“aceptación”", "aceptación"),
                    ],
                ),
                entry(
                    "III",
                    Definition,
                    vec![span(text, "por cadena causal", "cadena causal")],
                ),
                entry("IV", NonDefinition, Vec::new()),
            ],
            Vec::new(),
        );
        assert_eq!(
            profiled(text, &hint).unwrap(),
            [
                "I Tratado",
                "II Ratificación",
                "II adhesión",
                "II aceptación",
                "III cadena causal",
                "V Fondo"
            ]
        );
        // One fraction listing a term twice is still a repeated term.
        let mut twice = hint.clone();
        twice.entries[1].terms[1] = twice.entries[1].terms[0].clone();
        let error = profiled(text, &twice).unwrap_err();
        assert!(error.to_string().contains("repeats the term"), "{error}");
    }

    fn terms_and_fractions(terms: &[lex_core::DefinedTerm]) -> Vec<(String, String)> {
        terms
            .iter()
            .map(|term| (term.term.clone(), term.fraction.clone().unwrap_or_default()))
            .collect()
    }

    fn owned(pairs: &[(&str, &str)]) -> Vec<(String, String)> {
        pairs
            .iter()
            .map(|(term, fraction)| ((*term).to_owned(), (*fraction).to_owned()))
            .collect()
    }

    #[test]
    fn established_pass_reads_every_qualifier_spelling_and_dof_boundary() {
        // 2026-10-19 census: the established grammar read no qualifier, so
        // `III Bis` (lcm), `VI BIS` (lan), `V bis` (lsar) and `I Quáter`
        // (lrsf) were folded into the previous term's span; a fraction after
        // an amendment note (`…DOF 16-07-2025 XIII Bis.`, lfpiorpi) was too.
        let text = "Para los efectos de esta Ley, se entenderá por:\n\n\
             I. Agencia: La agencia;\n\n\
             I Bis. Banco: El banco;\n\n\
             I Ter. Caja: La caja;\n\n\
             I Quáter. Deuda: La deuda;\n\n\
             II. Ente: El ente; Fracción reformada DOF 16-07-2025 II BIS. Fondo: El fondo;\n\n\
             II ter. Grupo: El grupo;\n\n\
             III. Hogar: El hogar.";
        let glossary = provision(STATUTE_ID, "article:3", text);
        let detection = super::detect_glossary(std::slice::from_ref(&glossary)).unwrap();
        assert_eq!(detection.method, Some("established_pass"));
        assert_eq!(
            terms_and_fractions(&detection.terms),
            owned(&[
                ("Agencia", "I"),
                ("Banco", "I Bis"),
                ("Caja", "I Ter"),
                ("Deuda", "I Quáter"),
                ("Ente", "II"),
                ("Fondo", "II Bis"),
                ("Grupo", "II Ter"),
                ("Hogar", "III"),
            ])
        );
        assert_eq!(detection.unread_fractions, Some(Vec::new()));
        // No entry swallows the next one.
        let chars: Vec<char> = text.chars().collect();
        for term in &detection.terms {
            let span: String = chars[term.start_char..term.end_char].iter().collect();
            assert_eq!(span.matches(": ").count(), 1, "{span:?}");
        }
    }

    #[test]
    fn established_pass_reads_an_entry_by_its_own_delimiter_and_records_unread() {
        // socap-sofipo-dcg-2006 opens with a colon entry and then defines
        // almost every term with `, al` or `, a la`; the established pass
        // dropped those silently. An entry neither delimiter fits is recorded.
        let text = "Para efectos de las presentes disposiciones, se entenderá por:\n\n\
             I. Agencia: La agencia;\n\n\
             II. Auditor Interno, al responsable del área de auditoría;\n\n\
             III. Calificaciones, en singular o plural, a las calificaciones;\n\n\
             IV. Interculturalidad. El principio que asegura la inclusión;\n\n\
             V. Se deroga. Fracción derogada DOF 04-09-2009\n\n\
             VI. Mínima intervención cuando intervienen en procedimientos judiciales con \
             el fin de reducir su participación tanto como sea posible y siempre\n\n\
             VII. Programa Integral de Supervisión, Acompañamiento y Evaluación: El programa.";
        let glossary = provision(REGULATION_ID, "article:1", text);
        let detection = super::detect_glossary(std::slice::from_ref(&glossary)).unwrap();
        assert_eq!(
            terms_and_fractions(&detection.terms),
            owned(&[
                ("Agencia", "I"),
                ("Auditor Interno", "II"),
                ("Calificaciones", "III"),
                ("Interculturalidad", "IV"),
                (
                    "Programa Integral de Supervisión, Acompañamiento y Evaluación",
                    "VII"
                ),
            ])
        );
        assert_eq!(detection.unread_fractions, Some(vec!["VI".to_owned()]));
        assert_eq!(
            super::unread_fractions(&glossary, GlossaryStyle::AutoEstablished).unwrap(),
            Some(vec!["VI".to_owned()])
        );
        assert_eq!(
            super::unread_fractions(&glossary, GlossaryStyle::AutoWhole).unwrap(),
            None
        );
        // lissste's comma glossary: `a:` opens a list of incisos, so the
        // fallback reads no term ending in a connector.
        assert_eq!(
            super::own_delimiter_term("Familiares derechohabientes a: Párrafo reformado"),
            None
        );
        assert_eq!(
            super::own_delimiter_term("Consejos de Disciplina; al Consejo de Comisarios"),
            None
        );
    }

    #[test]
    fn wider_pass_reads_upper_case_and_higher_qualifiers_in_order() {
        // lgeepa 3o prints `VI BIS` and `VI TER`; lgmdfp prints `I Quater`
        // without the accent through `I Septies`.
        let detection = wider(
            "Para los efectos de esta Ley, se entiende por:\n\nI. Agencia: La agencia;\n\n\
             I Bis. Banco: El banco;\n\nI TER. Caja: La caja;\n\nI Quater. Deuda: La \
             deuda;\n\nI Quinquies. Ente: El ente;\n\nII. Fondo: El fondo.",
        );
        assert_eq!(detection.held, None);
        assert_eq!(
            terms_and_fractions(&detection.terms),
            owned(&[
                ("Agencia", "I"),
                ("Banco", "I Bis"),
                ("Caja", "I Ter"),
                ("Deuda", "I Quater"),
                ("Ente", "I Quinquies"),
                ("Fondo", "II"),
            ])
        );
        let skipped = wider(
            "Para los efectos de esta Ley, se entiende por:\n\nI. Agencia: La agencia;\n\n\
             I Bis. Banco: El banco;\n\nI Quáter. Caja: La caja;",
        );
        assert!(
            skipped
                .held
                .unwrap()
                .message
                .contains("I Quáter breaks the I, II, III… sequence after I Bis")
        );
        // A qualified heading behind furniture is still caught as swallowed.
        let swallowed = wider(
            "Para los efectos de esta Ley, se entiende por:\n\nI. Agencia: La agencia \
             ENCABEZADO I BIS. Banco: El banco;\n\nII. Caja: La caja.",
        );
        assert_eq!(
            swallowed.held.unwrap().code,
            GlossaryHoldCode::SwallowedFraction
        );
    }

    #[test]
    fn configured_fraction_styles_read_qualified_fractions() {
        let fractions = provision(
            STATUTE_ID,
            "article:2",
            "Para los efectos de esta Ley se entenderá por:\n\nI. Agencia, la agencia.\n\n\
             I BIS. Banco, el banco.\n\nII. Caja, la caja.",
        );
        assert_eq!(
            terms_and_fractions(&extract_terms(&fractions, GlossaryStyle::Fractions).unwrap()),
            owned(&[("Agencia", "I"), ("Banco", "I Bis"), ("Caja", "II")])
        );
        let roman_colon = provision(
            STATUTE_ID,
            "article:2",
            "Para los efectos de esta Ley se entenderá por: I. Agencia: la agencia; \
             I bis. Banco: el banco; II. Caja: la caja.",
        );
        assert_eq!(
            terms_and_fractions(&extract_terms(&roman_colon, GlossaryStyle::RomanColon).unwrap()),
            owned(&[("Agencia", "I"), ("Banco", "I Bis"), ("Caja", "II")])
        );
        let mixed = provision(
            STATUTE_ID,
            "article:2",
            "Para los efectos de esta Ley se entenderá por:\nI. Agencia, a la agencia;\n\
             I Ter. Banco: El banco;\nII. Caja, a la caja.",
        );
        assert_eq!(
            terms_and_fractions(&extract_terms(&mixed, GlossaryStyle::MixedFractions).unwrap()),
            owned(&[("Agencia", "I"), ("Banco", "I Ter"), ("Caja", "II")])
        );
    }

    #[test]
    fn established_pass_reads_a_period_between_numeral_and_qualifier() {
        // reg-lfjs 3 prints `I. BIS. Azar:` (read as the term `BIS. Azar`)
        // and `XII. Bis. Derogada.` (read as the term `Bis`).
        let glossary = provision(
            REGULATION_ID,
            "article:3",
            "Para los efectos del presente Reglamento, se entenderá por:\n\nI. Apuesta: La \
             apuesta;\n\nI. BIS. Azar: Casualidad;\n\nII. Ley: La Ley;\n\nII. Bis. Derogada. \
             Fracción adicionada DOF 19-10-2012. Derogada DOF 23-10-2013\n\nIII. Sorteo: El sorteo.",
        );
        let detection = super::detect_glossary(std::slice::from_ref(&glossary)).unwrap();
        assert_eq!(
            terms_and_fractions(&detection.terms),
            owned(&[
                ("Apuesta", "I"),
                ("Azar", "I Bis"),
                ("Ley", "II"),
                ("Sorteo", "III"),
            ])
        );
        assert_eq!(detection.unread_fractions, Some(Vec::new()));
        assert_eq!(lex_core::normalize_fraction_label("XII. Bis"), "XII Bis");
    }

    #[test]
    fn fraction_labels_normalize_their_qualifier_case_only() {
        assert_eq!(lex_core::normalize_fraction_label("XX BIS"), "XX Bis");
        assert_eq!(lex_core::normalize_fraction_label("III  bis"), "III Bis");
        assert_eq!(lex_core::normalize_fraction_label("X QUÁTER"), "X Quáter");
        assert_eq!(lex_core::normalize_fraction_label("I Quater"), "I Quater");
        assert_eq!(lex_core::normalize_fraction_label("XIV"), "XIV");
        assert_eq!(lex_core::fraction_qualifier_rank("QUÁTER"), Some(3));
        assert_eq!(lex_core::fraction_qualifier_rank("Decies"), Some(9));
        assert_eq!(lex_core::fraction_qualifier_rank("Quintus"), None);
    }
}
