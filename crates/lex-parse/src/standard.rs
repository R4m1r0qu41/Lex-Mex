use std::{collections::HashSet, sync::LazyLock};

use anyhow::{Result, bail};
use lex_core::{
    SCHEMA_VERSION, Severity, StandardClause, StandardClauseAmendment, StandardKind,
    StandardMetadata, StandardModificationAction, StandardStatus, StandardSupplement,
    StandardSupplementLegalCharacter, StandardTextBasis, StandardTransitory,
    StandardValidationReport, ValidationIssue,
};
use regex::Regex;

use crate::{
    collapse_whitespace,
    diputados::{parse_transitory_start, transitory_ordinals},
    slug, spanish_date,
};

/// Leading whitespace admissible before any line-anchored heading or marker.
///
/// `\x0c` (form feed) belongs here because `pdftotext` emits a page break
/// immediately before the first line of a page, with no intervening newline,
/// so a heading, ordinal, or section marker landing on a page boundary reads
/// as `"\x0c   TRANSITORIOS"`. Every line-anchored pattern in this module
/// must interpolate this one fragment: when only some of them admitted the
/// form feed, a page-break TRANSITORIOS was visible to the heading finder but
/// invisible to span bounding, so the last clause silently absorbed the whole
/// transitorios section -- and a page-break ordinal line made the heading
/// finder reject the genuine section altogether, re-admitting post-transitorios
/// table rows as clauses.
const LINE_LEAD: &str = r"[ \t\x0c]*";

static CLAUSE_HEADING: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(&format!(
        r"(?m)^{LINE_LEAD}(\d+(?:\.\d+)*)(?:(\.)[ \t]+|[ \t]+)([^\r\n].*?)[ \t]*\r?$"
    ))
    .expect("clause-heading regex must compile")
});

static LINE_START: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(&format!(r"(?m)^{LINE_LEAD}(\S.*)$")).expect("line-start regex must compile")
});

static TRANSITORIOS_HEADING: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(&format!(
        r"(?mi)^{LINE_LEAD}(?:ART[ÍI]CULOS?[ \t]+)?TRANSITORIOS?\b.*\r?$"
    ))
    .expect("transitorios heading regex must compile")
});

static CLAUSE_END: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(&format!(
        r"(?mi)^{LINE_LEAD}(?:{SIGNATURE_MARKERS}|(?:ART[ÍI]CULOS?[ \t]+)?TRANSITORIOS?\b|AP[ÉE]NDICE\b|ANEXO\b)"
    ))
    .expect("standard end-marker regex must compile")
});

static BIBLIOGRAPHY_HEADING: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)^bibliograf[ií]a\b").expect("bibliography-heading regex must compile")
});

// `\s+` (not `[ \t]+`) between tokens: `pdftotext -layout` output wraps
// long lines, and a date phrase can fall across that wrap (".. el 1 de
// octubre\nde 2023." is a real occurrence, not a contrived one).
static DATE_PHRASE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)(\d{1,2})[oº]?\s+de\s+([a-zá-úñ]+)\s+de\s+(\d{4})")
        .expect("date-phrase regex must compile")
});

/// Parse the numbered body of a NOM/NMX without treating its clauses as
/// statute articles. Character offsets address the unchanged extracted text.
pub fn parse_standard_clauses(
    source_text: &str,
    metadata: &StandardMetadata,
) -> Result<Vec<StandardClause>> {
    // The heading regex admits a form feed via `LINE_LEAD` -- the defect that
    // hid NOM-052-SEMARNAT-2005's entire body, leaving only its índice to be
    // selected. DOF running headers ("6 (Edición Vespertina) DIARIO OFICIAL
    // ...") also sit after a form feed, but they carry no ordinal period and
    // their label opens with `(`, so `plausible_top_level` already rejects
    // them.
    //
    // A standard's normative numbered body ends at TRANSITORIOS. What follows
    // -- apéndices, anexos, tablas, listados, and explicitly non-binding
    // "Guía de Referencia" material -- is not clause-structured, but is often
    // numbered, and continuing the run into it absorbs those rows as clauses
    // (744 phantom clauses from an exposure-limit table in NOM-010-STPS-2014).
    // Some of that trailing material is normative and simply is not modeled
    // yet; `validate_standard` reports its presence rather than letting the
    // record imply the standard ends where the clause body does.
    let body_limit = real_transitorios_heading(source_text)
        .map_or(source_text.len(), |(heading_start, _)| heading_start);
    let matches = CLAUSE_HEADING
        .captures_iter(source_text)
        .filter_map(|captures| {
            let whole = captures.get(0)?;
            let number = captures.get(1)?.as_str();
            let order = clause_order(number)?;
            let label = captures.get(3)?.as_str().trim_start();
            let plausible_top_level =
                captures.get(2).is_some() || label.chars().next().is_some_and(char::is_uppercase);
            let terminal_heading = order.len() == 1 && is_bibliography_heading(label);
            Some((
                whole.start(),
                number.to_owned(),
                order,
                plausible_top_level,
                terminal_heading,
            ))
        })
        .collect::<Vec<_>>();
    // Run selection sees the FULL match list; the TRANSITORIOS boundary is
    // applied to the winning run afterwards. Truncating the candidates first
    // shortens only body-side runs (the índice sits before the boundary and
    // never loses a row to it), so a body whose headings are partially
    // regex-invisible could lose the length comparison to its own índice --
    // the NOM-052 failure mode reintroduced through run selection. A run must
    // still *start* before the boundary: a real body begins before its own
    // transitorios, and admitting later starts would let a post-transitorios
    // guía or apéndice numbering that restarts at 1 compete for selection.
    let Some((selected, body_end)) = matches
        .iter()
        .enumerate()
        .filter(|(_, (start, _, order, plausible, _))| {
            *plausible && order.len() == 1 && matches!(order[0], 0 | 1) && *start < body_limit
        })
        .map(|(start, _)| numbered_body_run(&matches, start))
        .max_by_key(|(selected, _)| selected.len())
    else {
        bail!("standard text has no numbered body beginning with clause 0 or 1");
    };
    let numbering_end = matches
        .get(body_end)
        .map_or(source_text.len(), |(start, _, _, _, _)| *start);
    let matches = selected
        .into_iter()
        .filter(|&index| matches[index].0 < body_limit)
        .map(|index| matches[index].clone())
        .collect::<Vec<_>>();
    let structural_end = matches
        .last()
        .map_or(source_text.len(), |(start, _, _, _, _)| {
            standard_clause_end(source_text, *start, numbering_end)
        });

    let mut clauses = Vec::with_capacity(matches.len());
    for (index, (start, number, _, _, _)) in matches.iter().enumerate() {
        let natural_end = matches.get(index + 1).map_or(structural_end, |next| next.0);
        let end = standard_clause_end(source_text, *start, natural_end);
        let (trimmed_start, trimmed_end) = trim_span(source_text, *start, end);
        let text = source_text[trimmed_start..trimmed_end].to_owned();
        clauses.push(StandardClause {
            schema_version: SCHEMA_VERSION.to_owned(),
            id: format!("{}:clause:{}", metadata.id, clause_slug(number)),
            standard_id: metadata.id.clone(),
            number: number.clone(),
            label: first_line(&text).to_owned(),
            text,
            start_char: source_text[..trimmed_start].chars().count(),
            end_char: source_text[..trimmed_end].chars().count(),
            amended_by: Vec::new(),
        });
    }
    // Amendment marks are derived here rather than authored, so a committed
    // `clauses.json` stays under the same reparse-and-compare determinism check
    // that already guards clause spans: a change to title parsing shows up as
    // stale committed data instead of silently diverging.
    apply_amendment_marks(&mut clauses, metadata);
    Ok(clauses)
}

/// Parse a standard's TRANSITORIOS section into addressable, ordinal-labeled
/// blocks without attempting to understand their internal structure.
/// Deliberately lightweight: some standards' transitorios are long and
/// structurally complex (phased criteria, tables, cross-references), and a
/// full clause-style structural parse of that would risk exactly the
/// runaway complexity this stays clear of. Each block's raw text is scanned
/// for "N de MES de AAAA" date phrases; those are recorded as
/// `asserted_dates` without any claim about what they mean (entry into
/// force, phase boundary, deadline, ...) -- reading the surrounding text is
/// still required for that. Returns an empty vector for a standard with no
/// recognizable TRANSITORIOS section, which is not an error: absence is
/// reported by the caller if it matters.
///
/// # Panics
///
/// Panics only if the line-start regex match group used internally fails to
/// capture, which cannot happen given the regex's own construction.
pub fn parse_standard_transitories(
    source_text: &str,
    metadata: &StandardMetadata,
) -> Result<Vec<StandardTransitory>> {
    let ordinals = transitory_ordinals();
    let Some(layout) = standard_tail_layout(source_text, metadata)? else {
        return Ok(Vec::new());
    };
    let (section, section_offset) = (
        &source_text[layout.heading_end..layout.transitory_end],
        layout.heading_end,
    );

    let starts = LINE_START
        .captures_iter(section)
        .filter_map(|captures| {
            let line = captures
                .get(1)
                .expect("group 1 always matches with the line");
            parse_transitory_start(line.as_str(), &ordinals)
                .map(|(ordinal, _)| (line.start(), ordinal.to_owned()))
        })
        .collect::<Vec<_>>();

    let mut transitories = Vec::with_capacity(starts.len());
    for (index, (start, ordinal)) in starts.iter().enumerate() {
        let natural_end = starts.get(index + 1).map_or(section.len(), |next| next.0);
        let (trimmed_start, trimmed_end) = trim_span(section, *start, natural_end);
        let text = section[trimmed_start..trimmed_end].to_owned();
        let asserted_dates = DATE_PHRASE
            .captures_iter(&text)
            .filter_map(|captures| {
                spanish_date(&captures[1], &captures[2].to_lowercase(), &captures[3])
            })
            .collect();
        let absolute_start = section_offset + trimmed_start;
        let absolute_end = section_offset + trimmed_end;
        transitories.push(StandardTransitory {
            schema_version: SCHEMA_VERSION.to_owned(),
            id: format!("{}:transitory:{}", metadata.id, slug(ordinal)),
            standard_id: metadata.id.clone(),
            ordinal: ordinal.clone(),
            text,
            start_char: source_text[..absolute_start].chars().count(),
            end_char: source_text[..absolute_end].chars().count(),
            asserted_dates,
        });
    }
    Ok(transitories)
}

/// Parse configured top-level material following a genuine TRANSITORIOS
/// section. Anchors are input; spans, headings, sequence and legal character
/// are deterministic derived data.
pub fn parse_standard_supplements(
    source_text: &str,
    metadata: &StandardMetadata,
) -> Result<Vec<StandardSupplement>> {
    let Some(layout) = standard_tail_layout(source_text, metadata)? else {
        if metadata.supplement_starts.is_empty() {
            return Ok(Vec::new());
        }
        bail!("supplement anchors require a genuine TRANSITORIOS section");
    };
    let mut supplements = Vec::with_capacity(layout.supplements.len());
    for (index, span) in layout.supplements.iter().enumerate() {
        let configured = &metadata.supplement_starts[index];
        let (start, end) = trim_span(source_text, span.start, span.end);
        let text = source_text[start..end].to_owned();
        supplements.push(StandardSupplement {
            schema_version: SCHEMA_VERSION.to_owned(),
            id: format!("{}:supplement:{}", metadata.id, index + 1),
            standard_id: metadata.id.clone(),
            sequence: index + 1,
            kind: configured.kind,
            heading: collapse_whitespace(&configured.anchor),
            legal_character: derive_supplement_legal_character(&text).0,
            text,
            start_char: source_text[..start].chars().count(),
            end_char: source_text[..end].chars().count(),
        });
    }
    Ok(supplements)
}

#[derive(Debug)]
struct SupplementSpan {
    start: usize,
    end: usize,
}

#[derive(Debug)]
struct StandardTailLayout {
    heading_end: usize,
    transitory_end: usize,
    supplements: Vec<SupplementSpan>,
}

/// Resolve every standard-tail boundary once so clauses, transitories,
/// supplements and validation cannot disagree about the same source layout.
fn standard_tail_layout(
    source_text: &str,
    metadata: &StandardMetadata,
) -> Result<Option<StandardTailLayout>> {
    let Some((_, heading_end)) = real_transitorios_heading(source_text) else {
        return Ok(None);
    };
    let mut starts = Vec::with_capacity(metadata.supplement_starts.len());
    let mut previous = heading_end;
    let mut configured_anchors = HashSet::new();
    for configured in &metadata.supplement_starts {
        if configured.anchor.is_empty() {
            bail!("supplement anchor cannot be empty");
        }
        if !configured_anchors.insert(&configured.anchor) {
            bail!(
                "duplicate configured supplement anchor {:?}",
                configured.anchor
            );
        }
        let matches = source_text[heading_end..]
            .match_indices(&configured.anchor)
            .map(|(offset, _)| heading_end + offset)
            .collect::<Vec<_>>();
        if matches.len() != 1 {
            bail!(
                "supplement anchor {:?} must occur exactly once after TRANSITORIOS; found {}",
                configured.anchor,
                matches.len()
            );
        }
        let start = matches[0];
        if start < previous {
            bail!("supplement anchors are not in declared source order");
        }
        starts.push(start);
        previous = start + configured.anchor.len();
    }

    let first_anchor = starts.first().copied().unwrap_or(source_text.len());
    let signature = closing_signature_start(source_text, heading_end, source_text.len());
    let transitory_end = signature.map_or(first_anchor, |start| start.min(first_anchor));
    let supplements = starts
        .iter()
        .enumerate()
        .map(|(index, &start)| {
            let natural_end = starts.get(index + 1).copied().unwrap_or(source_text.len());
            let end =
                closing_signature_start(source_text, start, natural_end).unwrap_or(natural_end);
            SupplementSpan { start, end }
        })
        .collect();
    Ok(Some(StandardTailLayout {
        heading_end,
        transitory_end,
        supplements,
    }))
}

/// Locate the standard's real TRANSITORIOS section, returning
/// `(heading_start, heading_end)` byte offsets of the heading line itself.
///
/// A standard's índice repeats every section heading, including TRANSITORIOS,
/// before the real body -- the same false-first-match hazard the bibliography
/// heading has. Candidates are tried from the last occurrence backwards, and
/// the first one whose following section actually contains a recognized
/// ordinal start wins; an índice mention never does, since it is a bare
/// heading with no transitory text after it.
///
/// Shared by the transitory parser (which needs the section) and the clause
/// parser (which needs it as the end of the normative numbered body), so the
/// two can never disagree about where a standard's body stops.
fn real_transitorios_heading(source_text: &str) -> Option<(usize, usize)> {
    let ordinals = transitory_ordinals();
    let found = TRANSITORIOS_HEADING
        .find_iter(source_text)
        .map(|found| (found.start(), found.end()))
        .collect::<Vec<_>>();
    found
        .iter()
        .enumerate()
        .rev()
        .find_map(|(index, &(start, heading_end))| {
            let section_end = found
                .get(index + 1)
                .map_or(source_text.len(), |next| next.0);
            LINE_START
                .captures_iter(&source_text[heading_end..section_end])
                .any(|captures| parse_transitory_start(&captures[1], &ordinals).is_some())
                .then_some((start, heading_end))
        })
}

/// A closing dateline ("México, D.F., a ..." pre-2016; "Ciudad de México, a
/// ..." after the Distrito Federal/CDMX renaming) or a `SUFRAGIO EFECTIVO`
/// line, either of which opens a decree's signature block.
const SIGNATURE_MARKERS: &str = r"SUFRAGIO[ \t]+EFECTIVO\b|DADO[ \t]+EN\b|SE[ \t]+EXPIDE[ \t]+EN\b|PROV[ÉE]ASE[ \t]+LA[ \t]+PUBLICACI[ÓO]N\b|(?:CIUDAD[ \t]+DE[ \t]+M[ÉE]XICO|M[ÉE]XICO(?:,[ \t]+D\.?[ \t]*F\.?)?),[ \t]+A\b";

fn closing_signature_start(source_text: &str, start: usize, end: usize) -> Option<usize> {
    let pattern = Regex::new(&format!(r"(?mi)^{LINE_LEAD}(?:{SIGNATURE_MARKERS})"))
        .expect("signature-marker regex must compile");
    pattern
        .find(&source_text[start..end])
        .map(|found| start + found.start())
}

fn standard_clause_end(source_text: &str, clause_start: usize, natural_end: usize) -> usize {
    CLAUSE_END
        .find(&source_text[clause_start..natural_end])
        .map_or(natural_end, |marker| clause_start + marker.start())
}

fn numbered_body_run(
    matches: &[(usize, String, Vec<u32>, bool, bool)],
    start: usize,
) -> (Vec<usize>, usize) {
    let mut selected = vec![start];
    let mut previous = &matches[start].2;
    let mut current_top = previous[0];
    let mut top_level_closed = false;
    for (index, (_, _, order, plausible_top_level, terminal_heading)) in
        matches.iter().enumerate().skip(start + 1)
    {
        if order.len() == 1 {
            if top_level_closed || !plausible_top_level {
                continue;
            }
            if order[0] <= current_top {
                continue;
            }
            if order[0] != current_top + 1 {
                continue;
            }
            current_top = order[0];
        } else if order[0] != current_top || order <= previous {
            continue;
        }
        selected.push(index);
        previous = order;
        // A bibliography/references heading's own entries are legitimate
        // nested clauses when they continue its number (e.g. `11.1`,
        // `11.2`, ...), but some sources instead give the reference list
        // its own independent numbering that restarts at 1. That
        // restarted count can later coincidentally reach the value that
        // would follow this heading in the outer top-level sequence, so
        // only close the top-level run when the very next candidate is
        // such a restart; nested `current_top.N` sub-clauses and any
        // genuine following section keep parsing normally.
        if *terminal_heading
            && matches
                .get(index + 1)
                .is_some_and(|(.., next_order, _, _)| next_order.as_slice() == [1])
        {
            top_level_closed = true;
        }
    }
    (selected, matches.len())
}

/// Whether a top-level heading label names the standard's bibliography or
/// references section (`Bibliografía`, any accent/case variant). Its
/// internal reference list is a numbered enumeration of sources, not
/// sub-clauses, even when formatted as a numbered list.
fn is_bibliography_heading(label: &str) -> bool {
    BIBLIOGRAPHY_HEADING.is_match(label.trim_start())
}

/// One unit a modifying decree names in its own DOF publication title.
///
/// Derived from [`StandardModificationSource::title`], never authored. It
/// records *what a decree says it touches*, not any applied change; `clause`
/// is the addressed token exactly as the title writes it, including the
/// non-clause case (an `Apéndice normativo`, which the corpus does not model
/// and which therefore never resolves).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StandardModificationTarget {
    /// Index into [`StandardMetadata::modifications`].
    pub modification_index: usize,
    pub clause: String,
    pub action: StandardModificationAction,
    /// Whether `clause` matches a committed clause number exactly.
    ///
    /// Exact match only, deliberately: no resolution to a nearest committed
    /// ancestor. An unmatched target is a real signal (an *adición* of a
    /// numeral the base text does not contain, an annex the corpus does not
    /// model, or a numeral that should exist and does not), and collapsing it
    /// onto a parent clause would attach the mark to text the decree never
    /// addressed.
    pub resolved: bool,
}

/// Parse each recorded modification's DOF title into the units it names.
///
/// Title-only by design. Nothing here reads a decree's body, applies a text
/// change, or produces a consolidated text -- those are Scope 2 Stage C.
#[must_use]
pub fn parse_standard_modification_targets(
    metadata: &StandardMetadata,
    clauses: &[StandardClause],
) -> Vec<StandardModificationTarget> {
    let numbers = clauses
        .iter()
        .map(|clause| clause.number.as_str())
        .collect::<HashSet<_>>();
    let mut targets = Vec::new();
    for (modification_index, source) in metadata.modifications.iter().enumerate() {
        let Some(title) = source.title.as_deref() else {
            continue;
        };
        for (clause, action) in title_targets(title) {
            targets.push(StandardModificationTarget {
                modification_index,
                resolved: numbers.contains(clause.as_str()),
                clause,
                action,
            });
        }
    }
    targets
}

/// Split a decree title into the units it names, in order of appearance.
///
/// Conservative on purpose. The title is segmented at its own action verbs
/// (`Modificación` / `adición` / `eliminación` ...), each segment must carry a
/// target noun (`numeral`, `apéndice`, ...) before any token in it counts as a
/// target, and everything from the standard's own identity onward is discarded
/// first so a designation's digits ("NOM-247-SSA1-2008") can never be read as
/// a numeral. A title that names nothing yields an empty vector rather than a
/// guess -- STPS publishes "ACUERDO de Modificación a la Norma Oficial
/// Mexicana NOM-020-STPS-2011, ..." with no numerals at all, and the caller
/// must be able to tell that apart from a title that was never recorded.
fn title_targets(title: &str) -> Vec<(String, StandardModificationAction)> {
    static IDENTITY: LazyLock<Regex> = LazyLock::new(|| {
        Regex::new(r"(?i)\bnormas?\s+(?:oficiales?\s+)?mexicanas?\b|\bNO?MX?-")
            .expect("standard-identity regex must compile")
    });
    static CUE: LazyLock<Regex> = LazyLock::new(|| {
        Regex::new(r"(?i)\b(numeral(?:es)?|ap[eé]ndices?|anexos?|incisos?|cap[ií]tulos?|puntos?|tablas?|figuras?)\b")
            .expect("target-noun regex must compile")
    });
    // Case-insensitivity is scoped to the keyword groups only. A global `(?i)`
    // case-folds the identifier class too, so `[A-Z0-9]` matched a lowercase
    // `d` and "eliminación del Anexo de la ..." produced the bogus target
    // "Anexo de"; the identifier of a real annex ("Apéndice normativo A",
    // "Anexo 1") starts with a genuine capital or digit.
    static ANNEX: LazyLock<Regex> = LazyLock::new(|| {
        Regex::new(
            r"\b((?i:ap[eé]ndice|anexo)(?:\s+(?i:normativo|informativo))?\s+[A-Z0-9][A-Za-z0-9]*)",
        )
        .expect("annex-target regex must compile")
    });
    static NUMERAL: LazyLock<Regex> = LazyLock::new(|| {
        Regex::new(r"\b\d+(?:\.[0-9A-Za-z]+)*\)?").expect("numeral-target regex must compile")
    });

    let operative = match IDENTITY.find(title) {
        Some(found) => &title[..found.start()],
        // Without the boundary the digits of the standard's own designation are
        // indistinguishable from targets, so name nothing rather than guess.
        None => return Vec::new(),
    };

    let verbs = VERB.find_iter(operative).collect::<Vec<_>>();
    let mut targets: Vec<(String, StandardModificationAction)> = Vec::new();
    let mut seen = HashSet::new();
    for (position, found) in verbs.iter().enumerate() {
        let segment_end = verbs
            .get(position + 1)
            .map_or(operative.len(), regex::Match::start);
        let segment = &operative[found.start()..segment_end];
        if !CUE.is_match(segment) {
            continue;
        }
        let action = modification_action(found.as_str());
        // A date inside the segment ("del diverso publicado el 30 de junio de
        // 2011") is prose, but its day and year are bare integers the numeral
        // regex would otherwise capture -- and "30" or "2011" can collide with
        // a real top-level clause number, stamping a false amendment mark on
        // an unrelated clause. Date-phrase spans are excluded before any
        // numeral in them can count.
        let date_spans = DATE_PHRASE
            .find_iter(segment)
            .map(|found| (found.start(), found.end()))
            .collect::<Vec<_>>();
        let mut spans = ANNEX
            .find_iter(segment)
            .map(|found| (found.start(), found.end(), found.as_str().to_owned()))
            .collect::<Vec<_>>();
        for found in NUMERAL.find_iter(segment) {
            let covered = spans
                .iter()
                .any(|(start, end, _)| found.start() < *end && *start < found.end());
            let inside_date = date_spans
                .iter()
                .any(|(start, end)| found.start() < *end && *start < found.end());
            if !covered && !inside_date {
                spans.push((found.start(), found.end(), found.as_str().to_owned()));
            }
        }
        spans.sort_by_key(|(start, _, _)| *start);
        for (_, _, clause) in spans {
            let clause = collapse_whitespace(&clause);
            if seen.insert((clause.clone(), action)) {
                targets.push((clause, action));
            }
        }
    }
    targets
}

/// The single source of truth for decree action verbs: each family's regex
/// alternatives paired with the action they classify to.
///
/// Both the segmenting regex ([`VERB`]) and the classifier
/// ([`modification_action`]) are generated from this table, so a family
/// cannot be added to one and not the other. When they were two
/// hand-maintained lists with an `else => Modified` fallback, adding a verb
/// family to the regex alone compiled silently and labelled a repeal as a
/// modification -- the exact mislabeling `StandardModificationAction` exists
/// to prevent. Every family must cover the same grammatical forms, nominal
/// ("eliminación de los numerales") and conjugated ("se eliminan los
/// numerales"): segments are cut at these matches, so an unseen form is not
/// skipped -- its targets are absorbed by the preceding family.
const VERB_FAMILIES: &[(&str, StandardModificationAction)] = &[
    (
        r"modificaci[oó]n(?:es)?|modifican?",
        StandardModificationAction::Modified,
    ),
    (r"reformas?|reforman", StandardModificationAction::Modified),
    (
        r"adici[oó]n(?:es)?|adicionan?",
        StandardModificationAction::Added,
    ),
    (
        r"eliminaci[oó]n(?:es)?|eliminan?",
        StandardModificationAction::Eliminated,
    ),
    (
        r"derogaci[oó]n(?:es)?|derogan?",
        StandardModificationAction::Eliminated,
    ),
    (
        r"cancelaci[oó]n(?:es)?|cancelan?",
        StandardModificationAction::Eliminated,
    ),
    (
        r"supresi[oó]n(?:es)?|suprimen?",
        StandardModificationAction::Eliminated,
    ),
];

static VERB: LazyLock<Regex> = LazyLock::new(|| {
    let alternatives = VERB_FAMILIES
        .iter()
        .map(|(pattern, _)| *pattern)
        .collect::<Vec<_>>()
        .join("|");
    Regex::new(&format!(r"(?i)\b({alternatives})\b")).expect("modification-verb regex must compile")
});

static VERB_CLASSIFIERS: LazyLock<Vec<(Regex, StandardModificationAction)>> = LazyLock::new(|| {
    VERB_FAMILIES
        .iter()
        .map(|(pattern, action)| {
            (
                Regex::new(&format!(r"(?i)^(?:{pattern})$"))
                    .expect("verb-family classifier regex must compile"),
                *action,
            )
        })
        .collect()
});

fn modification_action(verb: &str) -> StandardModificationAction {
    VERB_CLASSIFIERS
        .iter()
        .find(|(family, _)| family.is_match(verb))
        .map(|(_, action)| *action)
        // Unreachable by construction: `verb` is a match of the alternation
        // built from the same table the classifiers are built from.
        .expect("a verb matched by the segmenting regex must classify to its own family")
}

/// Stamp each clause with the recorded decrees whose titles name it.
fn apply_amendment_marks(clauses: &mut [StandardClause], metadata: &StandardMetadata) {
    let targets = parse_standard_modification_targets(metadata, clauses);
    for clause in clauses.iter_mut() {
        clause.amended_by = targets
            .iter()
            .filter(|target| target.resolved && target.clause == clause.number)
            .map(|target| StandardClauseAmendment {
                modification_index: target.modification_index,
                action: target.action,
            })
            .collect();
    }
}

/// Validate the standards-specific identity, lifecycle, review separation,
/// clause ordering, uniqueness, hashes, and exact source spans.
#[must_use]
pub fn validate_standard(
    metadata: &StandardMetadata,
    clauses: &[StandardClause],
    transitories: &[StandardTransitory],
    supplements: &[StandardSupplement],
    source_text: &str,
) -> StandardValidationReport {
    let mut issues = Vec::new();
    // Derived once here and shared: `apply_amendment_marks` already consumed
    // the same derivation during parsing, and re-deriving it per validator
    // would invite the two to drift.
    let targets = parse_standard_modification_targets(metadata, clauses);
    validate_metadata(metadata, &mut issues);
    validate_standard_sources(metadata, &targets, &mut issues);
    validate_clauses(metadata, clauses, source_text, &mut issues);
    validate_transitories(metadata, transitories, source_text, &mut issues);
    validate_supplements(
        metadata,
        transitories,
        supplements,
        source_text,
        &mut issues,
    );
    validate_trailing_material(metadata, source_text, &mut issues);
    StandardValidationReport {
        schema_version: SCHEMA_VERSION.to_owned(),
        standard_id: metadata.id.clone(),
        valid: !issues.iter().any(|issue| issue.severity == Severity::Error),
        clause_count: clauses.len(),
        supplement_count: supplements.len(),
        issues,
    }
}

fn validate_metadata(metadata: &StandardMetadata, issues: &mut Vec<ValidationIssue>) {
    let prefix = match metadata.kind {
        StandardKind::Nom => "NOM-",
        StandardKind::Nmx => "NMX-",
    };
    if !metadata.designation.starts_with(prefix) {
        issues.push(error(
            "standard_designation",
            format!("{:?} designation must start {prefix}", metadata.kind),
            None,
        ));
    }
    if let Some(published) = metadata.published_designation.as_deref() {
        if !published.starts_with(prefix) {
            issues.push(error(
                "standard_published_designation",
                format!(
                    "{:?} published designation must start {prefix}",
                    metadata.kind
                ),
                None,
            ));
        }
        if published == metadata.designation {
            issues.push(error(
                "standard_published_designation",
                "published designation is only recorded when it differs from the current \
                 designation"
                    .to_owned(),
                None,
            ));
        } else {
            issues.push(warning(
                "standard_redesignated",
                format!(
                    "retained text was published as {published}; the registry now designates it \
                     {}",
                    metadata.designation
                ),
            ));
        }
    }
    if metadata.issuing_authorities.is_empty() {
        issues.push(error(
            "standard_issuing_authority",
            "at least one issuing authority is required".to_owned(),
            None,
        ));
    }
    if metadata.regulatory_domains.is_empty() {
        issues.push(error(
            "standard_regulatory_domain",
            "at least one regulatory domain is required".to_owned(),
            None,
        ));
    }
    if metadata.source_sha256.len() != 64 || metadata.extracted_text_sha256.len() != 64 {
        issues.push(error(
            "standard_source_hash",
            "source and extracted-text SHA-256 values must have 64 hex characters".to_owned(),
            None,
        ));
    }
    if metadata.status == StandardStatus::Replaced && metadata.replaced_by.is_empty() {
        issues.push(error(
            "standard_replacement",
            "a replaced standard must identify at least one replacement".to_owned(),
            None,
        ));
    }
    if metadata.status == StandardStatus::Cancelled && metadata.cancellation_date.is_none() {
        issues.push(error(
            "standard_cancellation_date",
            "a cancelled standard must record its cancellation date".to_owned(),
            None,
        ));
    }
    if metadata
        .effective_date
        .is_some_and(|date| date < metadata.publication_date)
    {
        issues.push(error(
            "standard_effective_date",
            "effective date cannot precede publication".to_owned(),
            None,
        ));
    }
    if metadata
        .cancellation_date
        .is_some_and(|date| date < metadata.publication_date)
    {
        issues.push(error(
            "standard_cancellation_date",
            "cancellation date cannot precede publication".to_owned(),
            None,
        ));
    }
}

fn validate_standard_sources(
    metadata: &StandardMetadata,
    targets: &[StandardModificationTarget],
    issues: &mut Vec<ValidationIssue>,
) {
    let included_modifications = metadata
        .modifications
        .iter()
        .filter(|source| source.included_in_source)
        .count();
    match metadata.text_basis {
        StandardTextBasis::AsPublished if included_modifications > 0 => {
            issues.push(error(
                "standard_text_basis",
                "an as-published source cannot include later modifications".to_owned(),
                None,
            ));
        }
        _ => {}
    }
    for (index, source) in metadata.modifications.iter().enumerate() {
        if source.publication_date <= metadata.publication_date {
            issues.push(error(
                "standard_modification_date",
                "a modification must postdate the standard publication".to_owned(),
                None,
            ));
        }
        let named = targets
            .iter()
            .filter(|target| target.modification_index == index)
            .collect::<Vec<_>>();
        if !source.included_in_source {
            let scope = if named.is_empty() {
                // Deliberately not "affects nothing": the decree's scope is
                // unknown, which is a weaker and more accurate claim.
                "; affected clauses unknown from its title".to_owned()
            } else {
                let marked = named
                    .iter()
                    .filter(|target| target.resolved)
                    .map(|target| target.clause.as_str())
                    .collect::<Vec<_>>();
                format!(
                    "; its title names {} unit(s), {} matching committed clauses ({})",
                    named.len(),
                    marked.len(),
                    if marked.is_empty() {
                        "none".to_owned()
                    } else {
                        marked.join(", ")
                    }
                )
            };
            issues.push(warning(
                "standard_unconsolidated_modification",
                format!(
                    "source text does not incorporate modification published {}{scope}",
                    source.publication_date
                ),
            ));
        }
        // Title diagnostics are raised only where the staleness they describe
        // is real. A modification the retained text already incorporates is not
        // made less current by an unrecorded title, and warning about it would
        // bury the two records where the gap actually matters.
        if !source.included_in_source {
            if source.title.is_none() {
                issues.push(warning(
                    "standard_modification_title_absent",
                    format!(
                        "modification published {} has no recorded DOF title, so the clauses it \
                         addresses cannot be located",
                        source.publication_date
                    ),
                ));
            } else if named.is_empty() {
                issues.push(warning(
                    "standard_modification_scope_unknown",
                    format!(
                        "modification published {} records a title that names no numeral, \
                         apéndice, or anexo; its scope stays at instrument level",
                        source.publication_date
                    ),
                ));
            }
        }
        for target in named.iter().filter(|target| !target.resolved) {
            issues.push(warning(
                "standard_modification_target_unresolved",
                format!(
                    "modification published {} names {} ({}), which matches no committed clause",
                    source.publication_date,
                    target.clause,
                    target.action.as_str(),
                ),
            ));
        }
    }
    if metadata
        .systematic_review
        .as_ref()
        .is_some_and(|review| review.review_date < metadata.publication_date)
    {
        issues.push(error(
            "standard_systematic_review_date",
            "systematic review cannot predate the standard publication".to_owned(),
            None,
        ));
    }
}

fn validate_clauses(
    metadata: &StandardMetadata,
    clauses: &[StandardClause],
    source_text: &str,
    issues: &mut Vec<ValidationIssue>,
) {
    if clauses.is_empty() {
        issues.push(error(
            "standard_clause_count",
            "standard has no numbered clauses".to_owned(),
            None,
        ));
        return;
    }
    let source_chars = source_text.chars().collect::<Vec<_>>();
    let mut ids = HashSet::new();
    let mut numbers = HashSet::new();
    let mut previous: Option<Vec<u32>> = None;
    let mut previous_top_level: Option<u32> = None;
    for clause in clauses {
        if clause.standard_id != metadata.id {
            issues.push(error(
                "standard_clause_instrument",
                "clause points to a different standard".to_owned(),
                Some(clause.id.clone()),
            ));
        }
        if !ids.insert(&clause.id) || !numbers.insert(&clause.number) {
            issues.push(error(
                "standard_clause_duplicate",
                "duplicate clause identifier or number".to_owned(),
                Some(clause.id.clone()),
            ));
        }
        let Some(order) = clause_order(&clause.number) else {
            issues.push(error(
                "standard_clause_number",
                "clause number is not a dot-separated numeric identifier".to_owned(),
                Some(clause.id.clone()),
            ));
            continue;
        };
        if previous.as_ref().is_some_and(|prior| order <= *prior) {
            issues.push(error(
                "standard_clause_order",
                "clause numbers do not increase in source order".to_owned(),
                Some(clause.id.clone()),
            ));
        }
        if order.len() == 1 {
            if previous_top_level.is_some_and(|prior| order[0] != prior + 1) {
                issues.push(error(
                    "standard_clause_gap",
                    "top-level clause numbers must be consecutive".to_owned(),
                    Some(clause.id.clone()),
                ));
            }
            previous_top_level = Some(order[0]);
        } else if previous_top_level != Some(order[0]) {
            issues.push(error(
                "standard_clause_parent",
                "nested clause appears outside its current top-level clause".to_owned(),
                Some(clause.id.clone()),
            ));
        }
        previous = Some(order);
        if clause.start_char >= clause.end_char || clause.end_char > source_chars.len() {
            issues.push(error(
                "standard_clause_span",
                "clause character span is outside the extracted text".to_owned(),
                Some(clause.id.clone()),
            ));
            continue;
        }
        let anchored = source_chars[clause.start_char..clause.end_char]
            .iter()
            .collect::<String>();
        if anchored != clause.text {
            issues.push(error(
                "standard_clause_span",
                "clause text does not match its exact extracted-text span".to_owned(),
                Some(clause.id.clone()),
            ));
        }
    }
}

/// Identity, uniqueness, and exact-span checks for transitorios. No absence
/// or ordering check: an empty list is not itself an error (some retained
/// texts genuinely lack a recognizable TRANSITORIOS section), and ordinal
/// sequencing is deliberately not enforced -- this stays a lightweight
/// inspection, not a structural parse.
fn validate_transitories(
    metadata: &StandardMetadata,
    transitories: &[StandardTransitory],
    source_text: &str,
    issues: &mut Vec<ValidationIssue>,
) {
    let source_chars = source_text.chars().collect::<Vec<_>>();
    let mut ids = HashSet::new();
    let mut ordinals = HashSet::new();
    for transitory in transitories {
        if transitory.standard_id != metadata.id {
            issues.push(error(
                "standard_transitory_instrument",
                "transitory points to a different standard".to_owned(),
                Some(transitory.id.clone()),
            ));
        }
        if !ids.insert(&transitory.id) || !ordinals.insert(&transitory.ordinal) {
            issues.push(error(
                "standard_transitory_duplicate",
                "duplicate transitory identifier or ordinal".to_owned(),
                Some(transitory.id.clone()),
            ));
        }
        if transitory.start_char >= transitory.end_char || transitory.end_char > source_chars.len()
        {
            issues.push(error(
                "standard_transitory_span",
                "transitory character span is outside the extracted text".to_owned(),
                Some(transitory.id.clone()),
            ));
            continue;
        }
        let anchored = source_chars[transitory.start_char..transitory.end_char]
            .iter()
            .collect::<String>();
        if anchored != transitory.text {
            issues.push(error(
                "standard_transitory_span",
                "transitory text does not match its exact extracted-text span".to_owned(),
                Some(transitory.id.clone()),
            ));
        }
    }
}

fn validate_supplements(
    metadata: &StandardMetadata,
    transitories: &[StandardTransitory],
    supplements: &[StandardSupplement],
    source_text: &str,
    issues: &mut Vec<ValidationIssue>,
) {
    let source_chars = source_text.chars().collect::<Vec<_>>();
    let mut ids = HashSet::new();
    let mut previous_end = transitories.last().map_or(0, |item| item.end_char);
    if supplements.len() != metadata.supplement_starts.len() {
        issues.push(error(
            "standard_supplement_count",
            format!(
                "metadata configures {} supplement starts but {} supplements were provided",
                metadata.supplement_starts.len(),
                supplements.len()
            ),
            None,
        ));
    }
    for (index, supplement) in supplements.iter().enumerate() {
        let expected_sequence = index + 1;
        if supplement.standard_id != metadata.id {
            issues.push(error(
                "standard_supplement_instrument",
                "supplement points to a different standard".to_owned(),
                Some(supplement.id.clone()),
            ));
        }
        if !ids.insert(&supplement.id) || supplement.sequence != expected_sequence {
            issues.push(error(
                "standard_supplement_identity",
                "supplement identifiers and one-based sequence must be unique and source-ordered"
                    .to_owned(),
                Some(supplement.id.clone()),
            ));
        }
        if metadata
            .supplement_starts
            .get(index)
            .is_some_and(|configured| configured.kind != supplement.kind)
        {
            issues.push(error(
                "standard_supplement_kind",
                "supplement kind differs from its configured anchor".to_owned(),
                Some(supplement.id.clone()),
            ));
        }
        if supplement.start_char < previous_end
            || supplement.start_char >= supplement.end_char
            || supplement.end_char > source_chars.len()
        {
            issues.push(error(
                "standard_supplement_span",
                "supplement spans must be non-overlapping, source-ordered, and inside the extracted text"
                    .to_owned(),
                Some(supplement.id.clone()),
            ));
            continue;
        }
        let anchored = source_chars[supplement.start_char..supplement.end_char]
            .iter()
            .collect::<String>();
        if anchored != supplement.text {
            issues.push(error(
                "standard_supplement_span",
                "supplement text does not match its exact extracted-text span".to_owned(),
                Some(supplement.id.clone()),
            ));
        }
        if metadata
            .supplement_starts
            .get(index)
            .is_some_and(|configured| {
                !supplement.text.starts_with(&configured.anchor)
                    || supplement.heading != collapse_whitespace(&configured.anchor)
            })
        {
            issues.push(error(
                "standard_supplement_anchor",
                "supplement does not begin at its exact configured anchor".to_owned(),
                Some(supplement.id.clone()),
            ));
        }
        validate_supplement_character(supplement, issues);
        previous_end = supplement.end_char;
    }
}

fn validate_supplement_character(
    supplement: &StandardSupplement,
    issues: &mut Vec<ValidationIssue>,
) {
    let (derived, conflict) = derive_supplement_legal_character(&supplement.text);
    if conflict {
        issues.push(error(
            "standard_supplement_character_conflict",
            "supplement contains conflicting explicit normative and non-normative signals"
                .to_owned(),
            Some(supplement.id.clone()),
        ));
    } else if supplement.legal_character != derived {
        issues.push(error(
            "standard_supplement_character",
            "supplement legal character is not the deterministic explicit-source derivation"
                .to_owned(),
            Some(supplement.id.clone()),
        ));
    } else if derived == StandardSupplementLegalCharacter::Unspecified {
        issues.push(warning(
            "standard_supplement_character_unspecified",
            format!(
                "supplement {} states no explicit normative or non-normative character",
                supplement.sequence
            ),
        ));
    }
}

fn derive_supplement_legal_character(text: &str) -> (StandardSupplementLegalCharacter, bool) {
    static NON_NORMATIVE_HEADING: LazyLock<Regex> = LazyLock::new(|| {
        Regex::new(r"(?i)\bNO[ \t\r\n]+NORMATIV[OA]S?\b")
            .expect("non-normative heading regex must compile")
    });
    static NON_BINDING_STATEMENT: LazyLock<Regex> = LazyLock::new(|| {
        Regex::new(r"(?i)\bNO[ \t\r\n]+ES[ \t\r\n]+DE[ \t\r\n]+CUMPLIMIENTO[ \t\r\n]+OBLIGATORIO\b")
            .expect("non-binding statement regex must compile")
    });
    static NORMATIVE: LazyLock<Regex> = LazyLock::new(|| {
        Regex::new(r"(?i)\bNORMATIV[OA]S?\b").expect("normative signal regex must compile")
    });
    // "Normativo" classifies only a heading. Scanning the entire opaque
    // supplement would misread ordinary prose about a normativa as a legal-
    // character declaration. The explicit non-binding sentence is allowed
    // anywhere because the source often places it below a multi-line title.
    let heading_prefix = text.chars().take(512).collect::<String>();
    let non_normative =
        NON_NORMATIVE_HEADING.is_match(&heading_prefix) || NON_BINDING_STATEMENT.is_match(text);
    let scrubbed = NON_NORMATIVE_HEADING.replace_all(&heading_prefix, "");
    let normative = NORMATIVE.is_match(&scrubbed);
    let conflict = normative && non_normative;
    let character = match (normative, non_normative) {
        (true, false) => StandardSupplementLegalCharacter::ExplicitlyNormative,
        (false, true) => StandardSupplementLegalCharacter::ExplicitlyNonNormative,
        _ => StandardSupplementLegalCharacter::Unspecified,
    };
    (character, conflict)
}

fn clause_order(number: &str) -> Option<Vec<u32>> {
    number.split('.').map(|part| part.parse().ok()).collect()
}

fn trim_span(text: &str, start: usize, end: usize) -> (usize, usize) {
    let candidate = &text[start..end];
    let leading = candidate.len() - candidate.trim_start().len();
    let trailing = candidate.len() - candidate.trim_end().len();
    (start + leading, end - trailing)
}

fn first_line(text: &str) -> &str {
    text.lines().next().unwrap_or(text).trim()
}

fn clause_slug(number: &str) -> String {
    number.replace('.', "-")
}

fn error(code: &str, message: String, provision_id: Option<String>) -> ValidationIssue {
    ValidationIssue {
        severity: Severity::Error,
        code: code.to_owned(),
        message,
        provision_id,
    }
}

/// Report substantive material following the transitorios section.
///
/// The clause parser deliberately stops at TRANSITORIOS, because what follows
/// is not clause-structured. But some of it is *normative* -- NOM-052's
/// Tablas, Listados and Anexo 1 carry the hazardous-waste classifications the
/// standard exists to establish, and NOM-010's Apéndice I carries its exposure
/// limits -- while other trailing material ("Guía de Referencia ... no es de
/// cumplimiento obligatorio") is explicitly non-binding. Lex-Mex does not yet
/// model either kind.
///
/// Without this warning a compiled standard would present a complete-looking
/// clause body while silently omitting operative content. The warning does not
/// distinguish normative from non-binding trailing material; making that call
/// requires reading it, which is the point of surfacing it to a reviewer.
///
/// Detection is structural first, byte-count last. When the transitorios
/// section ends at an APÉNDICE/ANEXO heading, that heading *is* the evidence
/// -- unmodeled material follows, and it is named unconditionally: a compact
/// single-formula apéndice below any byte threshold is exactly the silent
/// omission this warning exists for. When the section ends at the signature
/// block instead, the remainder is searched for a later annex-like heading
/// (GUÍA, LISTADO, TABLA, APÉNDICE, ANEXO -- annexes are sometimes laid out
/// after the signatures). Only a headingless remainder falls back to a size
/// heuristic, measured after stripping signature furniture (lines through the
/// last "Rúbrica"), because a long multi-signatory block is a closing
/// formality, not omitted content, and a false warning here trains reviewers
/// to dismiss the code.
fn validate_trailing_material(
    metadata: &StandardMetadata,
    source_text: &str,
    issues: &mut Vec<ValidationIssue>,
) {
    const TRAILING_MATERIAL_BYTES: usize = 2_000;
    static TRAILING_HEADING: LazyLock<Regex> = LazyLock::new(|| {
        Regex::new(&format!(
            r"(?mi)^{LINE_LEAD}((?:AP[ÉE]NDICE|ANEXOS?|GU[ÍI]A|LISTADOS?|TABLAS?)\b[^\r\n]*)"
        ))
        .expect("trailing-heading regex must compile")
    });
    static RUBRICA: LazyLock<Regex> =
        LazyLock::new(|| Regex::new(r"(?i)r[úu]brica").expect("rúbrica regex must compile"));

    if !metadata.supplement_starts.is_empty() {
        return;
    }
    let Some((_, heading_end)) = real_transitorios_heading(source_text) else {
        return;
    };
    let section_end = closing_signature_start(source_text, heading_end, source_text.len())
        .unwrap_or(source_text.len());
    let named_heading = TRAILING_HEADING
        .captures(&source_text[heading_end..])
        .map(|captures| collapse_whitespace(&captures[1]));
    if let Some(heading) = named_heading {
        issues.push(warning(
            "standard_trailing_material",
            format!(
                "material after the transitorios section begins at \"{heading}\"; it is not \
                 represented in clauses.json and may be normative"
            ),
        ));
        return;
    }
    // Headingless remainder: measure what is left after the signature block.
    let trailing = source_text[section_end..].trim();
    let unsigned = RUBRICA
        .find_iter(trailing)
        .last()
        .map_or(trailing, |last| {
            let after = &trailing[last.end()..];
            after.split_once('\n').map_or("", |(_, rest)| rest)
        })
        .trim();
    if unsigned.len() > TRAILING_MATERIAL_BYTES {
        issues.push(warning(
            "standard_trailing_material",
            format!(
                "{} bytes of unheaded material follow the transitorios section and its signature \
                 block; this material is not represented in clauses.json and may be normative",
                unsigned.len()
            ),
        ));
    }
}

fn warning(code: &str, message: String) -> ValidationIssue {
    ValidationIssue {
        severity: Severity::Warning,
        code: code.to_owned(),
        message,
        provision_id: None,
    }
}

#[cfg(test)]
mod tests {
    use super::{
        parse_standard_clauses, parse_standard_modification_targets, parse_standard_supplements,
        parse_standard_transitories, title_targets, validate_metadata, validate_standard,
    };
    use chrono::{NaiveDate, Utc};
    use lex_core::{
        ReviewStatus, SCHEMA_VERSION, Severity, StandardClauseAmendment, StandardKind,
        StandardMetadata, StandardModificationAction, StandardModificationSource, StandardStatus,
        StandardSupplementKind, StandardSupplementLegalCharacter, StandardSupplementStart,
        StandardTextBasis, TechnicalReviewStatus,
    };

    const SAMPLE: &str = include_str!("../../../fixtures/standards/numbered-standard-sample.txt");
    const INDEX_AND_APPENDIX_SAMPLE: &str =
        include_str!("../../../fixtures/standards/index-and-appendix-sample.txt");
    const BIBLIOGRAPHY_CONTINUATION_SAMPLE: &str =
        include_str!("../../../fixtures/standards/bibliography-continuation-sample.txt");
    const TRANSITORIOS_WITH_DATES_SAMPLE: &str =
        include_str!("../../../fixtures/standards/transitorios-with-dates-sample.txt");
    const INDEXED_TRANSITORIOS_SAMPLE: &str =
        include_str!("../../../fixtures/standards/indexed-transitorios-sample.txt");
    const CDMX_SIGNATURE_SAMPLE: &str =
        include_str!("../../../fixtures/standards/cdmx-signature-sample.txt");
    const PAGE_BREAK_HEADING_SAMPLE: &str =
        include_str!("../../../fixtures/standards/page-break-heading-sample.txt");
    const POST_TRANSITORIOS_ANNEX_SAMPLE: &str =
        include_str!("../../../fixtures/standards/post-transitorios-annex-sample.txt");
    const FORM_FEED_ORDINAL_SAMPLE: &str =
        include_str!("../../../fixtures/standards/form-feed-ordinal-sample.txt");
    const FORM_FEED_SECTION_SAMPLE: &str =
        include_str!("../../../fixtures/standards/form-feed-section-heading-sample.txt");
    const INDEX_OUTNUMBERS_BODY_SAMPLE: &str =
        include_str!("../../../fixtures/standards/index-outnumbers-body-sample.txt");

    #[test]
    fn a_redesignated_standard_records_its_published_designation() {
        // Mexican norm prefixes track the issuing authority, which is
        // occasionally reorganized. NOM-002-SEMARNAT-1996's retained text is
        // titled NOM-002-ECOL-1996 (SEMARNAP era); the registry redesignated
        // it. The current designation is canonical, but a record asserting a
        // designation that appears nowhere in its own source text must say so.
        let mut metadata = metadata();
        metadata.designation = "NOM-002-SEMARNAT-1996".to_owned();
        metadata.published_designation = Some("NOM-002-ECOL-1996".to_owned());
        let mut issues = Vec::new();
        validate_metadata(&metadata, &mut issues);
        assert!(
            issues
                .iter()
                .any(|issue| issue.code == "standard_redesignated"),
            "expected a redesignation warning, got {issues:?}"
        );
        assert!(
            !issues.iter().any(|issue| issue.severity == Severity::Error),
            "a recorded redesignation is not an error: {issues:?}"
        );

        // Recording it when nothing was redesignated is a mistake, not a
        // no-op: it would assert a discrepancy that does not exist.
        metadata.published_designation = Some("NOM-002-SEMARNAT-1996".to_owned());
        let mut issues = Vec::new();
        validate_metadata(&metadata, &mut issues);
        assert!(
            issues.iter().any(|issue| {
                issue.code == "standard_published_designation" && issue.severity == Severity::Error
            }),
            "expected an error when published equals current, got {issues:?}"
        );
    }

    #[test]
    fn body_heading_after_a_page_break_is_not_hidden_by_the_index() {
        // `pdftotext` emits a form feed immediately before the first line of
        // a page, with no intervening newline, so a heading landing on a page
        // boundary reads as "\x0c   1. Introducción". The leading-whitespace
        // class did not admit `\x0c`, so that heading never matched and the
        // real body became invisible -- leaving the índice as the only
        // candidate run. Reproduces NOM-052-SEMARNAT-2005, whose compiled
        // output was 11 índice lines covering 1.1% of the document while
        // every substantive provision was absent.
        let metadata = metadata();
        let clauses = parse_standard_clauses(PAGE_BREAK_HEADING_SAMPLE, &metadata).unwrap();
        assert_eq!(clauses.len(), 3);
        // The índice lists the same three numbers; the body is the run whose
        // clauses carry actual provision text.
        assert!(
            clauses[0].text.contains("manejo especial"),
            "expected the real body, got {:?}",
            clauses[0].text
        );
        assert!(
            clauses[2].text.contains("observancia obligatoria"),
            "expected the real body, got {:?}",
            clauses[2].text
        );
    }

    #[test]
    fn clause_body_stops_at_the_transitorios_section() {
        // A standard's normative numbered body ends at TRANSITORIOS. Trailing
        // apéndices, anexos, tablas and non-binding guías are frequently
        // numbered, and continuing the run into them absorbs those rows as
        // clauses -- 744 phantom clauses from an exposure-limit table in
        // NOM-010-STPS-2014, and a lone numeric table row (`12.5  0.024
        // 0.025`) in NOM-024-STPS-2001.
        let mut metadata = metadata();
        metadata
            .supplement_starts
            .push(lex_core::StandardSupplementStart {
                anchor: "APENDICE A".to_owned(),
                kind: lex_core::StandardSupplementKind::Appendix,
            });
        let clauses = parse_standard_clauses(POST_TRANSITORIOS_ANNEX_SAMPLE, &metadata).unwrap();
        assert_eq!(
            clauses
                .iter()
                .map(|clause| clause.number.as_str())
                .collect::<Vec<_>>(),
            ["1", "2", "3", "4"],
            "apéndice rows numbered 5-7 must not be absorbed as clauses"
        );
    }

    #[test]
    fn trailing_material_after_transitorios_is_reported() {
        // Dropping the apéndice from the clause body is correct, but some
        // trailing material is normative (NOM-052's Listados, NOM-010's
        // exposure limits). Nothing models it yet, so a compiled standard
        // would otherwise present a complete-looking body while omitting
        // operative content.
        let metadata = metadata();
        let long_annex = format!(
            "{POST_TRANSITORIOS_ANNEX_SAMPLE}{}",
            "8.   Sustancia adicional        Efecto documentado\n".repeat(60)
        );
        let clauses = parse_standard_clauses(&long_annex, &metadata).unwrap();
        let transitories = parse_standard_transitories(&long_annex, &metadata).unwrap();
        let report = validate_standard(&metadata, &clauses, &transitories, &[], &long_annex);
        assert!(
            report
                .issues
                .iter()
                .any(|issue| issue.code == "standard_trailing_material"),
            "expected a trailing-material warning, got {:?}",
            report.issues
        );
    }

    #[test]
    fn a_compact_annex_below_any_byte_threshold_is_still_reported_by_name() {
        // Detection is structural, not size-based: the transitorios section
        // ending at an APÉNDICE heading is itself the evidence that unmodeled,
        // possibly normative material follows. Under the old flat 2000-byte
        // threshold this sample's short annex produced no warning at all --
        // the exact silent omission the warning exists to prevent.
        let metadata = metadata();
        let clauses = parse_standard_clauses(POST_TRANSITORIOS_ANNEX_SAMPLE, &metadata).unwrap();
        let transitories =
            parse_standard_transitories(POST_TRANSITORIOS_ANNEX_SAMPLE, &metadata).unwrap();
        let report = validate_standard(
            &metadata,
            &clauses,
            &transitories,
            &[],
            POST_TRANSITORIOS_ANNEX_SAMPLE,
        );
        let trailing = report
            .issues
            .iter()
            .find(|issue| issue.code == "standard_trailing_material")
            .unwrap_or_else(|| panic!("expected a named warning, got {:?}", report.issues));
        assert!(
            trailing.message.contains("APÉNDICE I"),
            "the warning must name the heading it found: {}",
            trailing.message
        );
    }

    #[test]
    fn a_transitory_ordinal_after_a_page_break_is_still_an_ordinal() {
        // `pdftotext` emits "\x0c   PRIMERO.- ..." when the first ordinal
        // lands on a new page. The ordinal-confirmation scan did not admit the
        // form feed, so the genuine TRANSITORIOS heading failed confirmation:
        // body_limit fell back to the end of the text (re-admitting
        // post-transitorios table rows as clauses -- the 744-phantom-clause
        // defect returning through a different door), the transitories came
        // back empty, and no warning fired anywhere.
        let metadata = metadata();
        let clauses = parse_standard_clauses(FORM_FEED_ORDINAL_SAMPLE, &metadata).unwrap();
        assert_eq!(
            clauses
                .iter()
                .map(|clause| clause.number.as_str())
                .collect::<Vec<_>>(),
            ["1", "2"],
            "table rows after the transitorios must not be absorbed as clauses"
        );
        let transitories =
            parse_standard_transitories(FORM_FEED_ORDINAL_SAMPLE, &metadata).unwrap();
        assert_eq!(transitories.len(), 1, "the form-fed PRIMERO must be found");
        assert_eq!(
            transitories[0].asserted_dates,
            [NaiveDate::from_ymd_opt(2027, 1, 1).unwrap()]
        );
    }

    #[test]
    fn form_fed_section_headings_bound_clause_and_transitory_spans() {
        // The heading finder admitted "\x0cTRANSITORIOS" but the span-bounding
        // regexes did not, so the last clause absorbed the entire transitorios
        // section whenever it started a new PDF page -- and a form-fed
        // APÉNDICE was likewise swallowed by the last transitory, harvesting
        // decoy dates into asserted_dates and suppressing the
        // trailing-material warning exactly when the annex began a new page.
        // One shared LINE_LEAD fragment now feeds every line-anchored pattern.
        let mut metadata = metadata();
        metadata
            .supplement_starts
            .push(lex_core::StandardSupplementStart {
                anchor: "APENDICE A".to_owned(),
                kind: lex_core::StandardSupplementKind::Appendix,
            });
        let clauses = parse_standard_clauses(FORM_FEED_SECTION_SAMPLE, &metadata).unwrap();
        let last = clauses.last().expect("sample has clauses");
        assert!(
            !last.text.contains("TRANSITORIOS"),
            "the last clause must not absorb the form-fed transitorios section: {:?}",
            last.text
        );
        let transitories =
            parse_standard_transitories(FORM_FEED_SECTION_SAMPLE, &metadata).unwrap();
        assert_eq!(transitories.len(), 1);
        assert!(
            !transitories[0].text.contains("APENDICE"),
            "the transitory must not absorb the form-fed annex: {:?}",
            transitories[0].text
        );
        assert_eq!(
            transitories[0].asserted_dates,
            [NaiveDate::from_ymd_opt(2027, 3, 15).unwrap()],
            "the annex's decoy date must not be harvested as an asserted_date"
        );
        let supplements = parse_standard_supplements(FORM_FEED_SECTION_SAMPLE, &metadata).unwrap();
        assert_eq!(supplements.len(), 1);
        assert!(supplements[0].text.starts_with("APENDICE A"));
        let report = validate_standard(
            &metadata,
            &clauses,
            &transitories,
            &supplements,
            FORM_FEED_SECTION_SAMPLE,
        );
        assert!(
            !report
                .issues
                .iter()
                .any(|issue| issue.code == "standard_trailing_material"),
            "the represented form-fed annex must not remain unmodeled: {:?}",
            report.issues
        );
    }

    #[test]
    fn configured_supplements_partition_the_tail_and_keep_nested_tables_opaque() {
        let text = "1. Objetivo\nTexto.\n\nTRANSITORIOS\nPRIMERO. Vigencia.\n\n\
                    Dado en Ciudad de México, a 1 de julio de 2026.\nRúbrica\n\n\
                    APÉNDICE NORMATIVO\nTabla 1\n1. fila interna\n\n\
                    GUÍA DE REFERENCIA I\nEsta guía no es de cumplimiento obligatorio.\nTabla 2\n";
        let mut metadata = metadata();
        metadata.supplement_starts = vec![
            StandardSupplementStart {
                anchor: "APÉNDICE NORMATIVO".to_owned(),
                kind: StandardSupplementKind::Appendix,
            },
            StandardSupplementStart {
                anchor: "GUÍA DE REFERENCIA I".to_owned(),
                kind: StandardSupplementKind::ReferenceGuide,
            },
        ];
        let transitories = parse_standard_transitories(text, &metadata).unwrap();
        assert_eq!(transitories.len(), 1);
        assert_eq!(transitories[0].text, "PRIMERO. Vigencia.");
        let supplements = parse_standard_supplements(text, &metadata).unwrap();
        assert_eq!(
            supplements.len(),
            2,
            "nested tables stay inside their parent"
        );
        assert!(supplements[0].text.contains("Tabla 1"));
        assert_eq!(
            supplements[0].legal_character,
            StandardSupplementLegalCharacter::ExplicitlyNormative
        );
        assert_eq!(
            supplements[1].legal_character,
            StandardSupplementLegalCharacter::ExplicitlyNonNormative
        );
    }

    #[test]
    fn multiline_anchors_disambiguate_duplicate_headings_and_order_is_enforced() {
        let text = "1. Objetivo\nTexto.\nTRANSITORIOS\nÚNICO. Vigencia.\n\n\
                    GUÍA DE REFERENCIA I\nPrimera guía\nContenido.\n\n\
                    GUÍA DE REFERENCIA I\nSegunda guía\nContenido.\n";
        let mut metadata = metadata();
        metadata.supplement_starts = vec![
            StandardSupplementStart {
                anchor: "GUÍA DE REFERENCIA I\nPrimera guía".to_owned(),
                kind: StandardSupplementKind::ReferenceGuide,
            },
            StandardSupplementStart {
                anchor: "GUÍA DE REFERENCIA I\nSegunda guía".to_owned(),
                kind: StandardSupplementKind::ReferenceGuide,
            },
        ];
        assert_eq!(
            parse_standard_supplements(text, &metadata).unwrap().len(),
            2
        );

        metadata.supplement_starts.swap(0, 1);
        assert!(
            parse_standard_supplements(text, &metadata)
                .unwrap_err()
                .to_string()
                .contains("source order")
        );
        metadata.supplement_starts = vec![StandardSupplementStart {
            anchor: "GUÍA DE REFERENCIA I".to_owned(),
            kind: StandardSupplementKind::ReferenceGuide,
        }];
        assert!(
            parse_standard_supplements(text, &metadata)
                .unwrap_err()
                .to_string()
                .contains("exactly once")
        );
    }

    #[test]
    fn inline_table_references_are_not_tail_boundaries_and_conflicts_fail_validation() {
        let text = "1. Objetivo\nTexto.\nTRANSITORIOS\nPRIMERO. Véase la Tabla 1 para la vigencia.\n\n\
                    APÉNDICE NORMATIVO NO NORMATIVO\nContenido.\n";
        let mut metadata = metadata();
        metadata.supplement_starts = vec![StandardSupplementStart {
            anchor: "APÉNDICE NORMATIVO NO NORMATIVO".to_owned(),
            kind: StandardSupplementKind::Appendix,
        }];
        let transitories = parse_standard_transitories(text, &metadata).unwrap();
        assert!(transitories[0].text.contains("Tabla 1"));
        let supplements = parse_standard_supplements(text, &metadata).unwrap();
        let clauses = parse_standard_clauses(text, &metadata).unwrap();
        let report = validate_standard(&metadata, &clauses, &transitories, &supplements, text);
        assert!(!report.valid);
        assert!(
            report
                .issues
                .iter()
                .any(|issue| issue.code == "standard_supplement_character_conflict")
        );
    }

    #[test]
    fn body_run_selection_sees_matches_beyond_the_transitorios_boundary() {
        // Truncating candidate headings at the TRANSITORIOS boundary before
        // run selection shortens only body-side runs -- the índice sits
        // entirely before the boundary and never loses a row to it -- so a
        // body whose trailing headings are regex-invisible could lose the
        // length comparison to its own índice (the NOM-052 failure mode
        // returning through run selection). Selection now sees the full match
        // list and the boundary is applied to the winning run afterwards.
        let metadata = metadata();
        let clauses = parse_standard_clauses(INDEX_OUTNUMBERS_BODY_SAMPLE, &metadata).unwrap();
        assert_eq!(
            clauses
                .iter()
                .map(|clause| clause.number.as_str())
                .collect::<Vec<_>>(),
            ["1", "2", "3", "4"],
            "the body must win selection and then stop at the transitorios boundary"
        );
        assert!(
            clauses[0].text.contains("Esta norma establece"),
            "expected the real body, not the índice: {:?}",
            clauses[0].text
        );
    }

    #[test]
    fn transitory_text_excludes_a_ciudad_de_mexico_signature_block() {
        // The signature-block marker previously recognized only the
        // pre-2016 "México, D.F., a ..." dateline; the modern "Ciudad de
        // México, a ..." form (used since the Distrito Federal/CDMX
        // renaming) fell through and bled into the last transitory's text
        // -- found compiling NOM-051's real retained text, whose transitory
        // SEXTO absorbed its decree's closing signature and the decree's
        // own sign-off date as a false-positive asserted date.
        let metadata = metadata();
        let transitories = parse_standard_transitories(CDMX_SIGNATURE_SAMPLE, &metadata).unwrap();
        assert_eq!(transitories.len(), 1);
        assert!(!transitories[0].text.contains("Director General"));
        assert!(!transitories[0].text.contains("Rúbrica"));
        assert_eq!(
            transitories[0].asserted_dates,
            vec![NaiveDate::from_ymd_opt(2025, 10, 1).unwrap()]
        );
    }

    #[test]
    fn transitory_parsing_skips_the_index_mention_and_finds_the_real_section() {
        let metadata = metadata();
        // The índice repeats "ARTÍCULOS TRANSITORIOS" as a bare heading
        // with no ordinal content directly beneath it -- the same
        // false-first-match hazard the bibliography heading has for
        // clauses. Reproduces the real defect found compiling NOM-051's
        // retained text, where its índice occurrence (line 220) preceded
        // the real section (line 1735) and was matched first.
        let transitories =
            parse_standard_transitories(INDEXED_TRANSITORIOS_SAMPLE, &metadata).unwrap();
        assert_eq!(
            transitories
                .iter()
                .map(|t| t.ordinal.as_str())
                .collect::<Vec<_>>(),
            vec!["PRIMERO", "SEGUNDO"]
        );
        assert_eq!(
            transitories[0].asserted_dates,
            vec![NaiveDate::from_ymd_opt(2025, 10, 1).unwrap()]
        );
        assert!(transitories[1].asserted_dates.is_empty());
    }

    #[test]
    fn parses_ordinal_transitories_and_scans_their_dates() {
        let metadata = metadata();
        let clauses = parse_standard_clauses(TRANSITORIOS_WITH_DATES_SAMPLE, &metadata).unwrap();
        assert_eq!(
            clauses
                .iter()
                .map(|clause| clause.number.as_str())
                .collect::<Vec<_>>(),
            vec!["1", "2"]
        );
        assert!(!clauses[1].text.contains("TRANSITORIOS"));

        let transitories =
            parse_standard_transitories(TRANSITORIOS_WITH_DATES_SAMPLE, &metadata).unwrap();
        assert_eq!(
            transitories
                .iter()
                .map(|t| t.ordinal.as_str())
                .collect::<Vec<_>>(),
            vec!["PRIMERO", "SEGUNDO", "TERCERO"]
        );
        assert_eq!(
            transitories[0].asserted_dates,
            vec![NaiveDate::from_ymd_opt(2025, 10, 1).unwrap()]
        );
        assert_eq!(
            transitories[1].asserted_dates,
            vec![
                NaiveDate::from_ymd_opt(2023, 9, 30).unwrap(),
                NaiveDate::from_ymd_opt(2023, 10, 1).unwrap(),
            ]
        );
        assert_eq!(
            transitories[2].asserted_dates,
            vec![NaiveDate::from_ymd_opt(2021, 3, 31).unwrap()]
        );
        assert_eq!(
            transitories[0].id,
            format!("{}:transitory:primero", metadata.id)
        );

        let report = validate_standard(
            &metadata,
            &clauses,
            &transitories,
            &[],
            TRANSITORIOS_WITH_DATES_SAMPLE,
        );
        assert!(report.valid, "{:#?}", report.issues);
    }

    #[test]
    fn stops_numbered_body_at_bibliography_and_articulos_transitorios() {
        let metadata = metadata();
        let clauses = parse_standard_clauses(BIBLIOGRAPHY_CONTINUATION_SAMPLE, &metadata).unwrap();
        // The bibliography's own restarted list (1., 2., ...) must not be
        // read as clauses continuing the outer top-level count, even
        // though its second entry coincidentally reaches the count that
        // would follow clause "3" (Bibliografía) if the run continued.
        assert_eq!(
            clauses
                .iter()
                .map(|clause| clause.number.as_str())
                .collect::<Vec<_>>(),
            vec!["0", "1", "1.1", "1.2", "2", "2.1", "2.2", "3"]
        );
        let bibliography = clauses.last().unwrap();
        assert_eq!(bibliography.number, "3");
        assert!(
            bibliography
                .text
                .contains("Segunda referencia bibliográfica")
        );
        assert!(!bibliography.text.contains("TRANSITORIOS"));
        assert!(!bibliography.text.contains("ÚNICO"));
        let transitories =
            parse_standard_transitories(BIBLIOGRAPHY_CONTINUATION_SAMPLE, &metadata).unwrap();
        assert_eq!(transitories.len(), 1);
        assert_eq!(transitories[0].ordinal, "ÚNICO");
        assert!(transitories[0].text.contains("Entrará en vigor"));
        assert!(transitories[0].asserted_dates.is_empty());
        let report = validate_standard(
            &metadata,
            &clauses,
            &transitories,
            &[],
            BIBLIOGRAPHY_CONTINUATION_SAMPLE,
        );
        assert!(report.valid, "{:#?}", report.issues);
    }

    #[test]
    fn parses_numbered_standard_with_exact_spans() {
        let metadata = metadata();
        let clauses = parse_standard_clauses(SAMPLE, &metadata).unwrap();
        assert_eq!(
            clauses
                .iter()
                .map(|clause| clause.number.as_str())
                .collect::<Vec<_>>(),
            vec!["0", "1", "2", "3", "4", "5", "5.1", "5.2", "6"]
        );
        assert_eq!(clauses[6].label, "5.1 El establecimiento debe medir.");
        let transitories = parse_standard_transitories(SAMPLE, &metadata).unwrap();
        assert!(transitories.is_empty());
        let report = validate_standard(&metadata, &clauses, &transitories, &[], SAMPLE);
        assert!(report.valid, "{:#?}", report.issues);
    }

    #[test]
    fn skips_index_and_stops_before_signature_and_appendix_numbering() {
        let metadata = metadata();
        let clauses = parse_standard_clauses(INDEX_AND_APPENDIX_SAMPLE, &metadata).unwrap();
        assert_eq!(
            clauses
                .iter()
                .map(|clause| clause.number.as_str())
                .collect::<Vec<_>>(),
            vec!["1", "1.1", "1.2", "2", "2.1", "3", "3.1", "4"]
        );
        assert!(!clauses[6].text.contains("Sufragio"));
        assert!(!clauses[7].text.contains("México, D.F."));
        assert!(!clauses[7].text.contains("APENDICE"));
        let transitories =
            parse_standard_transitories(INDEX_AND_APPENDIX_SAMPLE, &metadata).unwrap();
        // This fixture has no TRANSITORIOS heading at all -- just a bare
        // signature/date line -- so no transitorios should be recognized.
        assert!(transitories.is_empty());
        let report = validate_standard(
            &metadata,
            &clauses,
            &transitories,
            &[],
            INDEX_AND_APPENDIX_SAMPLE,
        );
        assert!(report.valid, "{:#?}", report.issues);
    }

    #[test]
    fn current_designation_does_not_hide_unconsolidated_modification() {
        let mut metadata = metadata();
        metadata.modifications.push(modification(None));
        let clauses = parse_standard_clauses(SAMPLE, &metadata).unwrap();
        let transitories = parse_standard_transitories(SAMPLE, &metadata).unwrap();
        let report = validate_standard(&metadata, &clauses, &transitories, &[], SAMPLE);
        assert!(report.valid, "{:#?}", report.issues);
        assert!(report.issues.iter().any(|issue| {
            issue.code == "standard_unconsolidated_modification"
                && issue.severity == lex_core::Severity::Warning
        }));
        assert!(
            report
                .issues
                .iter()
                .any(|issue| issue.code == "standard_modification_title_absent"),
            "a modification with no recorded title must say its scope is unlocatable: {:#?}",
            report.issues
        );
    }

    // The three titles below are quoted verbatim from the official DOF pages
    // for the only three unincorporated modifications in the committed corpus:
    // codigo 5188649 (10/05/2011), 5283480 (27/12/2012), and 5411988
    // (19/10/2015).
    const NOM_247_2011_TITLE: &str = "Modificación de los numerales 1.4, 2, 3.2, 3.10, 3.12, \
         3.17, 3.18, 3.19, 3.36, 3.44 y 8 de la Norma Oficial Mexicana NOM-247-SSA1-2008, \
         Productos y servicios. Cereales y sus productos. Cereales, harinas de cereales, sémolas \
         o semolinas. Alimentos a base de: cereales, semillas comestibles, de harinas, sémolas o \
         semolinas o sus mezclas. Productos de panificación. Disposiciones y especificaciones \
         sanitarias y nutrimentales. Métodos de prueba.";
    const NOM_247_2012_TITLE: &str = "Modificación de los numerales 3.2, 3.10, 3.33, 4, 5.1.1, \
         5.2.7.ii.1), adición del numeral 5.1.5 y eliminación de los numerales 5.2.2.8, 5.2.3.4, \
         5.2.4.5 y el Apéndice normativo A de la Norma Oficial Mexicana NOM-247-SSA1-2008, \
         Productos y servicios. Cereales y sus productos. Cereales, harinas de cereales, sémolas \
         o semolinas. Alimentos a base de: cereales, semillas comestibles, de harinas, sémolas o \
         semolinas o sus mezclas. Productos de panificación. Disposiciones y especificaciones \
         sanitarias y nutrimentales. Métodos de prueba.";
    const NOM_020_2015_TITLE: &str = "ACUERDO de Modificación a la Norma Oficial Mexicana \
         NOM-020-STPS-2011, Recipientes sujetos a presión, recipientes criogénicos y generadores \
         de vapor o calderas-Funcionamiento-Condiciones de seguridad.";

    #[test]
    fn a_decree_title_names_the_numerals_it_modifies() {
        assert_eq!(
            title_targets(NOM_247_2011_TITLE),
            [
                "1.4", "2", "3.2", "3.10", "3.12", "3.17", "3.18", "3.19", "3.36", "3.44", "8"
            ]
            .map(|clause| (clause.to_owned(), StandardModificationAction::Modified))
        );
    }

    #[test]
    fn a_compound_decree_title_separates_modified_added_and_eliminated() {
        // The real NOM-247 second decree does all three at once and ends by
        // eliminating an annex, which the corpus does not model at all.
        assert_eq!(
            title_targets(NOM_247_2012_TITLE),
            [
                ("3.2", StandardModificationAction::Modified),
                ("3.10", StandardModificationAction::Modified),
                ("3.33", StandardModificationAction::Modified),
                ("4", StandardModificationAction::Modified),
                ("5.1.1", StandardModificationAction::Modified),
                ("5.2.7.ii.1)", StandardModificationAction::Modified),
                ("5.1.5", StandardModificationAction::Added),
                ("5.2.2.8", StandardModificationAction::Eliminated),
                ("5.2.3.4", StandardModificationAction::Eliminated),
                ("5.2.4.5", StandardModificationAction::Eliminated),
                (
                    "Apéndice normativo A",
                    StandardModificationAction::Eliminated
                ),
            ]
            .map(|(clause, action)| (clause.to_owned(), action))
        );
    }

    #[test]
    fn a_date_inside_a_decree_title_is_never_read_as_a_numeral() {
        // "del diverso publicado el 30 de junio de 2011" is prose, but its
        // day and year are bare integers -- and "30" can collide with a real
        // top-level clause number, stamping a false amendment mark on an
        // unrelated clause. That is the one failure direction the feature
        // exists to prevent.
        assert_eq!(
            title_targets(
                "Modificación de los numerales 4.1 y 4.2 del diverso publicado el 30 de junio \
                 de 2011, de la Norma Oficial Mexicana NOM-999-TEST-2026, Prueba."
            ),
            [
                ("4.1", StandardModificationAction::Modified),
                ("4.2", StandardModificationAction::Modified)
            ]
            .map(|(clause, action)| (clause.to_owned(), action))
        );
    }

    #[test]
    fn a_lowercase_word_after_anexo_is_not_an_annex_identifier() {
        // A global (?i) case-folded the identifier class too, so [A-Z0-9]
        // matched the "d" of "de" and "eliminación del Anexo de la Norma..."
        // produced the bogus target "Anexo de". A real annex identifier
        // ("Apéndice normativo A", "Anexo 1") starts with a genuine capital
        // or digit; a bare "Anexo" followed by prose names nothing.
        assert!(
            title_targets(
                "Eliminación del Anexo de la Norma Oficial Mexicana NOM-999-TEST-2026, Prueba."
            )
            .is_empty()
        );
        // The legitimate forms keep working, including a digit identifier.
        assert_eq!(
            title_targets(
                "Eliminación del Anexo 1 de la Norma Oficial Mexicana NOM-999-TEST-2026, Prueba."
            ),
            [("Anexo 1".to_owned(), StandardModificationAction::Eliminated)]
        );
    }

    #[test]
    fn conjugated_action_verbs_are_covered_symmetrically() {
        // Regression: `reforman` matched while `derogan` did not, so this whole
        // title collapsed into one *modified* segment and both repeals came
        // back labelled as modifications. Segments are cut at verb matches, so
        // an action family the regex cannot see does not go missing -- its
        // targets are silently absorbed by the family before it, which is the
        // one failure direction amendment marks exist to prevent.
        assert_eq!(
            title_targets(
                "Acuerdo por el que se reforman los numerales 3.2 y 3.4 y se derogan los \
                 numerales 5.1 y 5.2 de la Norma Oficial Mexicana NOM-999-TEST-2026, Prueba."
            ),
            [
                ("3.2", StandardModificationAction::Modified),
                ("3.4", StandardModificationAction::Modified),
                ("5.1", StandardModificationAction::Eliminated),
                ("5.2", StandardModificationAction::Eliminated),
            ]
            .map(|(clause, action)| (clause.to_owned(), action))
        );
    }

    #[test]
    fn a_decree_title_naming_no_numeral_yields_no_targets() {
        // STPS publishes modifications as "ACUERDO de Modificación a la Norma
        // Oficial Mexicana NOM-020-STPS-2011, ..." -- the title carries the
        // standard's identity and nothing about what changed. Guessing a target
        // here would be worse than reporting the scope as unknown, and the
        // digits of the designation itself must never be read as numerals.
        assert!(title_targets(NOM_020_2015_TITLE).is_empty());

        let mut metadata = metadata();
        metadata
            .modifications
            .push(modification(Some(NOM_020_2015_TITLE)));
        let clauses = parse_standard_clauses(SAMPLE, &metadata).unwrap();
        let transitories = parse_standard_transitories(SAMPLE, &metadata).unwrap();
        let report = validate_standard(&metadata, &clauses, &transitories, &[], SAMPLE);
        assert!(report.valid, "{:#?}", report.issues);
        assert!(
            report
                .issues
                .iter()
                .any(|issue| issue.code == "standard_modification_scope_unknown"),
            "a title that names nothing is a different fact from no title: {:#?}",
            report.issues
        );
        assert!(
            !report
                .issues
                .iter()
                .any(|issue| issue.code == "standard_modification_title_absent"),
            "the title is recorded; only its scope is unknown: {:#?}",
            report.issues
        );
    }

    #[test]
    fn an_unmatched_target_is_reported_rather_than_resolved_to_an_ancestor() {
        // The fixture body is 1/1.1/1.2/2/3..., so "5.1.5" matches nothing.
        // Attaching the mark to a nearest committed ancestor would claim a
        // decree addressed text it never named.
        let mut metadata = metadata();
        metadata.modifications.push(modification(Some(
            "Adición del numeral 5.1.5 de la Norma Oficial Mexicana NOM-999-TEST-2026, Prueba.",
        )));
        let clauses = parse_standard_clauses(SAMPLE, &metadata).unwrap();
        let targets = parse_standard_modification_targets(&metadata, &clauses);
        assert_eq!(targets.len(), 1);
        assert!(!targets[0].resolved);
        assert!(
            clauses.iter().all(|clause| clause.amended_by.is_empty()),
            "an unresolved target must mark no clause"
        );

        let transitories = parse_standard_transitories(SAMPLE, &metadata).unwrap();
        let report = validate_standard(&metadata, &clauses, &transitories, &[], SAMPLE);
        assert!(report.valid, "an adición of a new numeral is not an error");
        assert!(
            report
                .issues
                .iter()
                .any(|issue| issue.code == "standard_modification_target_unresolved"),
            "{:#?}",
            report.issues
        );
    }

    #[test]
    fn a_resolved_target_marks_its_clause_with_the_decree_and_its_action() {
        // Two decrees naming the same numeral must both mark it, and the mark
        // must carry the action: a clause a decree *eliminated* cannot render
        // as "modificado".
        let mut metadata = metadata();
        metadata.modifications.push(modification(Some(
            "Modificación del numeral 5.1 de la Norma Oficial Mexicana NOM-999-TEST-2026, Prueba.",
        )));
        metadata.modifications.push(modification(Some(
            "Eliminación del numeral 5.1 de la Norma Oficial Mexicana NOM-999-TEST-2026, Prueba.",
        )));
        let clauses = parse_standard_clauses(SAMPLE, &metadata).unwrap();
        let marked = clauses
            .iter()
            .find(|clause| clause.number == "5.1")
            .expect("fixture has clause 5.1");
        assert_eq!(
            marked.amended_by,
            vec![
                StandardClauseAmendment {
                    modification_index: 0,
                    action: StandardModificationAction::Modified,
                },
                StandardClauseAmendment {
                    modification_index: 1,
                    action: StandardModificationAction::Eliminated,
                },
            ]
        );
        assert!(
            clauses
                .iter()
                .filter(|clause| !clause.amended_by.is_empty())
                .count()
                == 1,
            "no other clause may be marked"
        );
    }

    fn modification(title: Option<&str>) -> StandardModificationSource {
        StandardModificationSource {
            publication_date: NaiveDate::from_ymd_opt(2026, 7, 26).unwrap(),
            official_url: "https://example.test/dof/modification".parse().unwrap(),
            included_in_source: false,
            title: title.map(str::to_owned),
        }
    }

    fn metadata() -> StandardMetadata {
        StandardMetadata {
            schema_version: SCHEMA_VERSION.to_owned(),
            id: "urn:lex-mx:federal:nom:nom-999-test-2026".to_owned(),
            kind: StandardKind::Nom,
            designation: "NOM-999-TEST-2026".to_owned(),
            published_designation: None,
            official_title: "Norma de prueba".to_owned(),
            issuing_authorities: vec!["Secretaría de Prueba".to_owned()],
            regulatory_domains: vec!["fixture".to_owned()],
            publication_date: NaiveDate::from_ymd_opt(2026, 7, 25).unwrap(),
            effective_date: None,
            cancellation_date: None,
            status: StandardStatus::Unknown,
            replaces: Vec::new(),
            replaced_by: Vec::new(),
            joint_prefixes: Vec::new(),
            objective: None,
            scope: None,
            conformity_assessment: None,
            text_basis: StandardTextBasis::AsPublished,
            modifications: Vec::new(),
            supplement_starts: Vec::new(),
            systematic_review: None,
            source_url: "https://example.test/source.pdf".parse().unwrap(),
            official_dof_url: "https://example.test/dof".parse().unwrap(),
            official_registry_url: None,
            publisher: "Fixture".to_owned(),
            retrieved_at: Utc::now(),
            source_sha256: "a".repeat(64),
            extracted_text_sha256: "b".repeat(64),
            parser_version: env!("CARGO_PKG_VERSION").to_owned(),
            legal_review_status: ReviewStatus::NotAnalyzed,
            technical_review_status: TechnicalReviewStatus::NotAnalyzed,
        }
    }
}
