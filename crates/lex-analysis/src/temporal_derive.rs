//! Deterministic derivation of an article's own in-force status, distinct
//! from the LLM-based transitory-effect analysis in `lex_core::temporal`.
//!
//! This module answers one narrow question with plain code, no model:
//! "given the instrument's own commencement clause, is this already-parsed
//! article currently in force?" It never touches a provision the corpus has
//! already formed an opinion about (`review_status != NotAnalyzed`), so it
//! can never overwrite a human's or a model's prior determination — only
//! ever advance a provision out of the untouched state.
//!
//! Two rules, both required to be unambiguous before they act:
//!
//! - **Repeal.** `initial_temporal_status` already marks a provision whose
//!   text opens with a repeal marker as [`TemporalStatus::Repealed`] at
//!   parse time; this module only promotes that pre-existing, already-safe
//!   classification's provenance and review state.
//! - **Commencement.** An instrument's *ordinary* (original-enactment)
//!   transitorios are scanned for exactly one unqualified "entrará en
//!   vigor" clause resolving to a single concrete date via a short list of
//!   known-safe shapes (day after publication, same day, a literal date, or
//!   a calendar-day count from publication). Anything else — more than one
//!   commencement clause, an exception/conditional qualifier anywhere in the
//!   matched transitory, a business-day period, a month/year period, or a
//!   shape this module does not recognize — is left untouched and reported
//!   as a distinct, named [`SkipReason`] so an operator can address one
//!   category at a time, the same way `docs/ingestion-difficulty-log.md`
//!   already tracks parser gaps.
//!
//! A "simple" instrument's commencement drives the existing machine status
//! classification of its non-repealed articles. An article's own start date
//! is one of exactly two things: the original commencement, if the article
//! was never amended, or the commencement of the reform that last touched
//! it. This rule derives the first only. An article showing amendment
//! evidence (a `DOF dd-mm-yyyy` note or a footnote mark) keeps no
//! `effective_from`, because its date is its reform's commencement, which
//! needs that reform's own transitories. This is not a historical-version
//! model: it says when the current wording could have taken effect, not what
//! the law said on a chosen past date.

use crate::business_days::{BusinessDayCalendar, CountOutcome, count_business_days_after};
use chrono::{Days, NaiveDate};
use lex_core::{
    Basis, Instrument, Provision, ProvisionType, ReviewStatus, TemporalDetermination,
    TemporalStatus,
};
use regex::Regex;

/// Identifies this rule set's output in `TemporalDetermination.model` /
/// `.prompt_version`, so a reviewer or downstream consumer can tell at a
/// glance that a determination came from code, not a model call, and which
/// version of the rules produced it.
pub const DETERMINATION_SOURCE: &str = "deterministic-rule";
pub const RULE_SET_VERSION: &str = "temporal-derive-v3";

/// Why an instrument's commencement could not be resolved automatically.
/// Each variant is a distinct, addressable outlier category: the intended
/// workflow is that an operator reviews one category, and — where the
/// category turns out to be a systematic gap rather than a genuinely
/// case-by-case legal question — extends this module with a fixture,
/// narrowing the category rather than the instrument.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SkipReason {
    PublicationNotExact,
    /// The instrument has no ordinary transitory provisions at all.
    NoOrdinaryTransitories,
    /// None of the ordinary transitories mention entering into force.
    NoCommencementClause,
    /// More than one ordinary transitory mentions entering into force —
    /// commencement is staged or split by subject matter.
    MultipleCommencementClauses {
        count: usize,
    },
    /// The matched transitory carries an exception, condition, or
    /// staged-rollout qualifier (`con excepción`, `salvo`, `cuando`,
    /// `gradualmente`, `declaratoria`, `a más tardar`, …) — some or all of
    /// the instrument's own text does not share the plain reading.
    ExceptionOrConditionalLanguage {
        marker: String,
    },
    /// The commencement period is counted in business days (`días hábiles`)
    /// in a clause shape this module does not recognize as complete.
    BusinessDaysPeriod,
    /// A complete, plain business-day clause whose count under article 28 of
    /// the Ley Federal de Procedimiento Administrativo cannot be finished from
    /// evidenced calendar inputs. The exact date stays unresolved.
    BusinessDaysCalendarInsufficient {
        detail: String,
    },
    /// The commencement period uses a unit this module does not compute
    /// (`meses`, `años`) — day arithmetic only is supported today.
    UnsupportedRelativePeriodUnit {
        unit: String,
    },
    /// A day count is written in Spanish words this module's bounded number
    /// table does not cover, rather than digits.
    UnrecognizedNumberWord {
        word: String,
    },
    /// Exactly one commencement clause was found, but its shape does not
    /// match any pattern this module resolves (a cross-reference to another
    /// transitory, a rollout tied to a different event, a spelled-out year,
    /// etc.), or a literal date it did match failed calendar validation.
    UnrecognizedCommencementPattern,
}

impl SkipReason {
    /// Stable, kebab-case identifier for reports and CLI output.
    #[must_use]
    pub fn code(&self) -> &'static str {
        match self {
            Self::PublicationNotExact => "publication-not-exact",
            Self::NoOrdinaryTransitories => "no-ordinary-transitories",
            Self::NoCommencementClause => "no-commencement-clause",
            Self::MultipleCommencementClauses { .. } => "multiple-commencement-clauses",
            Self::ExceptionOrConditionalLanguage { .. } => "exception-or-conditional-language",
            Self::BusinessDaysPeriod => "business-days-period",
            Self::BusinessDaysCalendarInsufficient { .. } => "business-days-calendar-insufficient",
            Self::UnsupportedRelativePeriodUnit { .. } => "unsupported-relative-period-unit",
            Self::UnrecognizedNumberWord { .. } => "unrecognized-number-word",
            Self::UnrecognizedCommencementPattern => "unrecognized-commencement-pattern",
        }
    }

    #[must_use]
    pub fn describe(&self) -> String {
        match self {
            Self::PublicationNotExact => "publication is a period, not an exact day".to_owned(),
            Self::NoOrdinaryTransitories => {
                "instrument has no ordinary transitory provisions".to_owned()
            }
            Self::NoCommencementClause => {
                "no ordinary transitory mentions entering into force".to_owned()
            }
            Self::MultipleCommencementClauses { count } => {
                format!("{count} ordinary transitories mention entering into force")
            }
            Self::ExceptionOrConditionalLanguage { marker } => {
                format!("commencement clause carries a qualifier: {marker:?}")
            }
            Self::BusinessDaysPeriod => "commencement period is in business days".to_owned(),
            Self::BusinessDaysCalendarInsufficient { detail } => format!(
                "plain business-day clause; the article 28 count is not completed by evidenced calendar inputs ({detail})"
            ),
            Self::UnsupportedRelativePeriodUnit { unit } => {
                format!("commencement period unit {unit:?} is not computed")
            }
            Self::UnrecognizedNumberWord { word } => {
                format!("commencement clause uses an unrecognized number word: {word:?}")
            }
            Self::UnrecognizedCommencementPattern => {
                "commencement clause found but its shape is not recognized".to_owned()
            }
        }
    }
}

/// The instrument-level commencement verdict, kept separately from the
/// per-article determinations for reporting.
#[derive(Debug, Clone)]
pub enum Commencement {
    Resolved {
        effective_from: NaiveDate,
        pattern: &'static str,
        matched_text: String,
        /// Where the date came from when it rests on evidenced inputs (a
        /// calendar count); `None` for dates written in the clause itself.
        provenance: Option<String>,
    },
    /// The exact date is not computed (a business-day period needs a holiday
    /// calendar; a month period has two readings) but the clause is a plain
    /// period after publication and its **latest possible** commencement is
    /// known. If that date has passed, articles are `Effective` with no
    /// asserted start date.
    Bounded {
        latest: NaiveDate,
        pattern: &'static str,
        matched_text: String,
    },
    Skipped {
        reason: SkipReason,
        matched_text: Option<String>,
    },
}

#[derive(Debug, Clone, Default)]
pub struct DerivationOutcome {
    /// Ready to apply via `lex_core::temporal::apply_temporal_determinations`.
    pub determinations: Vec<TemporalDetermination>,
    /// `None` only when the instrument has no ordinary transitories at all
    /// (repeal-marker articles, if any, are still classified via `Rule A`).
    pub commencement: Option<Commencement>,
}

/// Derive deterministic temporal determinations for an instrument's
/// articles. Never reads or writes anything outside `provisions`; the
/// caller applies the result and persists it.
#[must_use]
pub fn derive_article_temporal_determinations(
    instrument: &Instrument,
    provisions: &[Provision],
    today: NaiveDate,
) -> DerivationOutcome {
    derive_article_temporal_determinations_with_calendar(
        instrument,
        provisions,
        today,
        &BusinessDayCalendar::default(),
    )
}

/// As [`derive_article_temporal_determinations`], with the evidenced inputs a
/// business-day count under article 28 of the Ley Federal de Procedimiento
/// Administrativo depends on. An empty calendar resolves no business-day clause.
#[must_use]
pub fn derive_article_temporal_determinations_with_calendar(
    instrument: &Instrument,
    provisions: &[Provision],
    today: NaiveDate,
    calendar: &BusinessDayCalendar,
) -> DerivationOutcome {
    let Some(publication_date) = instrument.publication_date else {
        return DerivationOutcome {
            determinations: Vec::new(),
            commencement: Some(Commencement::Skipped {
                reason: SkipReason::PublicationNotExact,
                matched_text: None,
            }),
        };
    };
    let mut determinations = Vec::new();

    // Rule A: promote an already-repealed, untouched article's provenance.
    // The classification itself was already made at parse time by
    // `initial_temporal_status`; this only records that it is deterministic
    // and advances it out of `NotAnalyzed`.
    for provision in provisions {
        if provision.provision_type == ProvisionType::Article
            && provision.review_status == ReviewStatus::NotAnalyzed
            && provision.temporal_status == TemporalStatus::Repealed
        {
            determinations.push(determination(
                provision,
                instrument,
                TemporalStatus::Repealed,
                None,
                1.0,
                vec!["text opens with a repeal marker".to_owned()],
            ));
        }
    }

    // Rule A, partial: the parser classifies an article whose opening note
    // repeals only named paragraphs or fractions as `PartiallyRepealed` and
    // records that scope. Promote it like a whole repeal only while the note
    // still reads as exactly the recorded scope; the remaining text's
    // commencement is not derived here (ltfccg 16 lost its provenance on
    // reparse, Codex review 2026-10-20).
    for provision in provisions {
        if provision.provision_type == ProvisionType::Article
            && provision.review_status == ReviewStatus::NotAnalyzed
            && provision.temporal_status == TemporalStatus::PartiallyRepealed
            && !provision.repeals.is_empty()
            && crate::initial::initial_repeals(&provision.text) == provision.repeals
        {
            determinations.push(determination(
                provision,
                instrument,
                TemporalStatus::PartiallyRepealed,
                None,
                1.0,
                vec![
                    "text opens with a partial repeal note matching its recorded scope".to_owned(),
                ],
            ));
        }
    }

    // Rule B: resolve the instrument's own commencement, then apply it to
    // every remaining untouched, non-repealed article.
    let ordinary_transitories: Vec<&Provision> = provisions
        .iter()
        .filter(|provision| provision.provision_type == ProvisionType::Transitory)
        .collect();
    if ordinary_transitories.is_empty() {
        return DerivationOutcome {
            determinations,
            commencement: None,
        };
    }
    let commencement = resolve_commencement(&ordinary_transitories, publication_date, calendar);
    let rule_b = commencement_verdict(&commencement, today);
    if let Some((status, start, reason)) = rule_b {
        for provision in provisions {
            if provision.provision_type != ProvisionType::Article
                || provision.review_status != ReviewStatus::NotAnalyzed
                || matches!(
                    provision.temporal_status,
                    TemporalStatus::Repealed | TemporalStatus::PartiallyRepealed
                )
            {
                continue;
            }
            // An article with no amendment evidence still has its original
            // wording, so it took effect with the instrument. An amended one
            // took effect with the reform that touched it, which this rule
            // does not derive, so its date stays unset.
            let article_date = start.filter(|_| !has_amendment_evidence(provision));
            determinations.push(determination(
                provision,
                instrument,
                status.clone(),
                article_date,
                0.9,
                vec![reason.clone()],
            ));
        }
    }

    DerivationOutcome {
        determinations,
        commencement: Some(commencement),
    }
}

/// What an instrument's commencement means for its untouched articles: the
/// status, the start date to record (if any) and the evidence line, or `None`
/// when the commencement cannot be decided yet.
fn commencement_verdict(
    commencement: &Commencement,
    today: NaiveDate,
) -> Option<(TemporalStatus, Option<NaiveDate>, String)> {
    match commencement {
        Commencement::Resolved {
            effective_from,
            provenance,
            ..
        } => Some((
            if *effective_from > today {
                TemporalStatus::FutureEffective
            } else {
                TemporalStatus::Effective
            },
            Some(*effective_from),
            match provenance {
                Some(provenance) => format!(
                    "instrument's own commencement clause resolves to {effective_from} ({provenance})"
                ),
                None => {
                    format!("instrument's own commencement clause resolves to {effective_from}")
                }
            },
        )),
        // Certain only once the latest possible date has passed; the start
        // date itself is not asserted.
        Commencement::Bounded {
            latest, pattern, ..
        } if *latest <= today => Some((
            TemporalStatus::Effective,
            None,
            format!(
                "instrument's own commencement clause ({pattern}) takes effect no later than {latest}; the exact date is not computed"
            ),
        )),
        _ => None,
    }
}

/// Whether an article's own text or footnote legend shows it was amended
/// after the instrument's enactment.
fn has_amendment_evidence(provision: &Provision) -> bool {
    !provision.amendment_marks.is_empty() || lex_grammar::mentions_dof_date(&provision.text)
}

/// Repair article start dates written by earlier versions of the rule: an
/// article with no amendment evidence takes the resolved original
/// commencement, and an amended one is left unset. Only values that are unset
/// or exactly the original commencement are touched, and only on
/// machine-accepted deterministic articles, so a model or human decision is
/// never altered. Returns how many dates changed.
///
/// The commencement is resolved with the same `calendar` the derivation uses,
/// so a date written from evidenced business-day inputs is repaired on the
/// same evidence instead of being invisible to the repair.
#[must_use]
pub fn repair_article_dates(
    instrument: &Instrument,
    provisions: &mut [Provision],
    calendar: &BusinessDayCalendar,
) -> usize {
    let Some(publication_date) = instrument.publication_date else {
        return 0;
    };
    let transitories = provisions
        .iter()
        .filter(|provision| provision.provision_type == ProvisionType::Transitory)
        .collect::<Vec<_>>();
    let Commencement::Resolved { effective_from, .. } =
        resolve_commencement(&transitories, publication_date, calendar)
    else {
        return 0;
    };
    let mut changed = 0;
    for provision in provisions {
        if provision.provision_type == ProvisionType::Article
            && provision.temporal_basis == Some(Basis::DeterministicRule)
            && provision.review_status == ReviewStatus::MachineAccepted
            && matches!(
                provision.temporal_status,
                TemporalStatus::Effective | TemporalStatus::FutureEffective
            )
            && provision.effective_to.is_none()
            && provision.transitory_effects.is_empty()
        {
            let current = provision.effective_from;
            if current.is_some() && current != Some(effective_from) {
                continue;
            }
            let target = (!has_amendment_evidence(provision)).then_some(effective_from);
            if current != target {
                provision.effective_from = target;
                changed += 1;
            }
        }
    }
    changed
}

/// Reclassify committed articles that an earlier parser stored as wholly
/// `Repealed` although their opening note repeals only named paragraphs or
/// fractions (`(Se deroga el primer párrafo)`). Only machine-accepted
/// deterministic articles whose note [`crate::initial::initial_repeals`] recognises are
/// touched; the rest of the article text stays in force. Returns how many
/// provisions changed.
#[must_use]
pub fn repair_partial_repeals(provisions: &mut [Provision]) -> usize {
    let mut changed = 0;
    for provision in provisions {
        if provision.temporal_status == TemporalStatus::Repealed
            && provision.temporal_basis == Some(Basis::DeterministicRule)
            && provision.review_status == ReviewStatus::MachineAccepted
            && provision.repeals.is_empty()
        {
            let repeals = crate::initial::initial_repeals(&provision.text);
            if !repeals.is_empty() {
                provision.temporal_status = TemporalStatus::PartiallyRepealed;
                provision.repeals = repeals;
                changed += 1;
            }
        }
    }
    changed
}

fn determination(
    provision: &Provision,
    instrument: &Instrument,
    status: TemporalStatus,
    effective_from: Option<NaiveDate>,
    confidence: f32,
    supporting_text: Vec<String>,
) -> TemporalDetermination {
    TemporalDetermination {
        provision_id: provision.id.clone(),
        temporal_status: status,
        publication_date: instrument
            .publication_date
            .expect("exact day checked before derivation"),
        effective_from,
        effective_to: None,
        confidence,
        basis: Basis::DeterministicRule,
        supporting_text,
        review_required: false,
        review_reason: None,
        model: DETERMINATION_SOURCE.to_owned(),
        prompt_version: RULE_SET_VERSION.to_owned(),
        effects: Vec::new(),
        evidence_sha256: String::new(),
    }
}

/// Qualifiers that keep a commencement clause out of the plain rule. Each is
/// matched as whole words on the lowercased, accent-folded, whitespace-collapsed
/// text, so `Interno entrará` is not a negation and `no  entrará` (two spaces
/// or a line break) still is. The second item is the name reported.
const EXCEPTION_MARKERS: [(&str, &str); 14] = [
    (r"\bcon\s+excepcion\b", "con excepción"),
    (r"\ba\s+excepcion\b", "a excepción"),
    (r"\bexcepcion\s+hecha\b", "excepción hecha"),
    (r"\bsalvo\b", "salvo"),
    (r"\bexcepto\b", "excepto"),
    (r"\bsin\s+perjuicio\b", "sin perjuicio"),
    (r"\ba\s+mas\s+tardar\b", "a más tardar"),
    (r"\bgradual", "gradual"),
    (r"\bdeclaratoria\b", "declaratoria"),
    (r"\ben\s+tanto\b", "en tanto"),
    (r"\bhasta\s+que\b", "hasta que"),
    // a negated commencement verb is never a plain commencement
    (r"\bno\s+entrara\s+en\s+vigor\b", "no entrará en vigor"),
    (r"\bno\s+entraran\s+en\s+vigor\b", "no entrarán en vigor"),
    (r"\bno\s+entran?\s+en\s+vigor\b", "no entra en vigor"),
];

/// Lowercase, accent-folded and whitespace-collapsed text for the qualifier
/// and unit guards. `ñ` is kept; only the vowel accents and `ü` are folded.
fn folded(text: &str) -> String {
    let lower = text.to_lowercase();
    let mapped = lower
        .chars()
        .map(|c| match c {
            'á' | 'à' | 'ä' => 'a',
            'é' | 'è' | 'ë' => 'e',
            'í' | 'ì' | 'ï' => 'i',
            'ó' | 'ò' | 'ö' => 'o',
            'ú' | 'ù' | 'ü' => 'u',
            other => other,
        })
        .collect::<String>();
    mapped.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// Whitespace-collapsed clause text with ordinal periods removed (`1o.` and
/// `1º.` read as `1o`), so the anchored patterns see one sentence shape and a
/// date such as `1o. de mayo` does not look like a sentence end.
fn clause_text(text: &str) -> String {
    let whole = text.split_whitespace().collect::<Vec<_>>().join(" ");
    Regex::new(r"(\d{1,2})\s*[oº°]\.")
        .expect("static regex")
        .replace_all(&whole, "${1}o")
        .into_owned()
}

fn resolve_commencement(
    transitories: &[&Provision],
    publication_date: NaiveDate,
    calendar: &BusinessDayCalendar,
) -> Commencement {
    let trigger = Regex::new(r"(?i)entrar[áa]n?\s+en\s+vigor").expect("static regex");
    let matches: Vec<&&Provision> = transitories
        .iter()
        .filter(|provision| trigger.is_match(&provision.text))
        .collect();
    match matches.len() {
        0 => Commencement::Skipped {
            reason: SkipReason::NoCommencementClause,
            matched_text: None,
        },
        1 => resolve_commencement_text(&matches[0].text, publication_date, calendar),
        count => Commencement::Skipped {
            reason: SkipReason::MultipleCommencementClauses { count },
            matched_text: None,
        },
    }
}

fn resolve_commencement_text(
    text: &str,
    publication_date: NaiveDate,
    calendar: &BusinessDayCalendar,
) -> Commencement {
    let guard = folded(text);
    if let Some((_, marker)) = EXCEPTION_MARKERS
        .iter()
        .find(|(pattern, _)| Regex::new(pattern).expect("static regex").is_match(&guard))
    {
        return Commencement::Skipped {
            reason: SkipReason::ExceptionOrConditionalLanguage {
                marker: (*marker).to_owned(),
            },
            matched_text: Some(text.to_owned()),
        };
    }
    // Complete, supported clause forms only: the whole provision text must be
    // the clause, so a qualifier, a negation or a second event anywhere in it
    // sends the clause to review instead of through a fragment match.
    if let Some(commencement) = try_complete_clause(text, publication_date, calendar) {
        return commencement;
    }
    // Every other date path is anchored the same way: subject, verb, date
    // phrase and an optional Diario Oficial tail must be the whole text.
    let clause = clause_text(text);
    if let Some(commencement) = try_day_after_publication(&clause, publication_date)
        .or_else(|| try_same_day_as_publication(&clause, publication_date))
        .or_else(|| try_literal_date(&clause))
        .or_else(|| try_relative_days(&clause, publication_date))
    {
        return commencement;
    }
    // Nothing anchored matched: report the closest category for review.
    if Regex::new(r"\bhabil")
        .expect("static regex")
        .is_match(&guard)
    {
        return Commencement::Skipped {
            reason: SkipReason::BusinessDaysPeriod,
            matched_text: Some(text.to_owned()),
        };
    }
    if let Some(unit) = Regex::new(
        r"entraran?\s+en\s+vigor\s+(?:a\s+los\s+)?\S+(?:\s+y\s+\S+)?\s+(mes(?:es)?|anos?)\b",
    )
    .expect("static regex")
    .captures(&guard)
    {
        return Commencement::Skipped {
            reason: SkipReason::UnsupportedRelativePeriodUnit {
                unit: unit[1].to_owned(),
            },
            matched_text: Some(text.to_owned()),
        };
    }
    Commencement::Skipped {
        reason: SkipReason::UnrecognizedCommencementPattern,
        matched_text: Some(text.to_owned()),
    }
}

/// The subject of a plain commencement sentence on the legacy date paths:
/// `la presente Ley`, `el presente Reglamento`, `esta Ley`, `las presentes
/// disposiciones`, `el presente "Reglamento"`, optionally followed by a short
/// instrument name (`la presente Ley Federal de Sanidad Animal`). A sentence
/// that opens with anything else (a declaration, a condition, a second verb)
/// is not a plain commencement and goes to review.
const LEGACY_SUBJECT: &str = r#"(?:(?:la|el|las|los)\s+present[ea]s?|est[ae]s?)\s+"?(?:ley|reglamento|decreto|c[oó]digo|ordenamiento|disposiciones|acuerdo|estatuto)"?(?:\s+(?:federal|general|org[aá]nica|nacional)(?:\s+(?:de|del|para|sobre)\s+[a-záéíóúñ]+(?:\s+[a-záéíóúñ]+){0,3})?)?"#;

/// What may follow the date phrase: the Diario Oficial (quoted or not, with or
/// without `de la Federación`) and a plain abrogation of an earlier law, then
/// the end of the text. Two kinds of text that the source extraction leaves
/// behind are also allowed, because neither can carry a legal qualifier: a
/// legislative signature block (place and date, an optional quoted year
/// motto, then only `Name, D. P.-` style signatures ending in `Rúbricas`, where a
/// name is capitalised name words only, so no sentence can pass as one) and
/// a bare trailing page number.
const LEGACY_TAIL: &str = r#"(?:\s+en\s+el\s+"?diario\s+oficial"?(?:\s+de\s+la\s+federaci[oó]n)?)?(?:\s+y\s+abroga\s+[^.;:]+)?\s*\.?(?:\s+(?:m[eé]xico,\s+d\.\s?f\.,\s+a\s+\d{1,2}\s+de\s+[a-záéíóúñ]+\s+de\s+\d{4}\.?-\s*)?(?:"[^"]+"\.?-\s*)?(?:(?-i:[A-ZÁÉÍÓÚÑ][\w.]*(?:\s+(?:y|de|del|la|[A-ZÁÉÍÓÚÑ][\w.]*)){0,6}),\s+(?-i:[DS]\.\s?(?:V\.\s?)?[PS]\.)\s?-\s*)+r[úu]bricas?[".]*)?(?:\s+\d{1,4})?\s*$"#;

/// `^<subject> entrará(n) en vigor <phrase><tail>$`, case-insensitive.
fn anchored(phrase: &str) -> Regex {
    Regex::new(&format!(
        r"(?i)^{LEGACY_SUBJECT}\s+entrar[áa]n?\s+en\s+vigor\s+{phrase}{LEGACY_TAIL}"
    ))
    .expect("static regex")
}

fn try_day_after_publication(clause: &str, publication_date: NaiveDate) -> Option<Commencement> {
    let re = anchored(
        r"(?:a\s+partir\s+)?(?:del?|al|el)\s+(?:d[ií]a\s+siguiente|siguiente\s+d[ií]a)\s+(?:al\s+de|al|de|a)\s+(?:su|la)\s+publicaci[óo]n",
    );
    let found = re.find(clause)?;
    let effective_from = publication_date.checked_add_days(Days::new(1))?;
    Some(Commencement::Resolved {
        effective_from,
        pattern: "day_after_publication",
        matched_text: found.as_str().to_owned(),
        provenance: None,
    })
}

fn try_same_day_as_publication(clause: &str, publication_date: NaiveDate) -> Option<Commencement> {
    let re = anchored(
        r"(?:desde\s+|en\s+)?(?:el\s+mismo\s+d[ií]a|el\s+d[ií]a|la\s+fecha)\s+de\s+(?:su|la)\s+(?:publicaci[óo]n|promulgaci[óo]n)(?:\s+del\s+presente\s+decreto)?",
    );
    let found = re.find(clause)?;
    Some(Commencement::Resolved {
        effective_from: publication_date,
        pattern: "same_day_as_publication",
        matched_text: found.as_str().to_owned(),
        provenance: None,
    })
}

const MONTHS: [&str; 12] = [
    "enero",
    "febrero",
    "marzo",
    "abril",
    "mayo",
    "junio",
    "julio",
    "agosto",
    "septiembre",
    "octubre",
    "noviembre",
    "diciembre",
];

fn month_number(word: &str) -> Option<u32> {
    MONTHS
        .iter()
        .position(|month| *month == word)
        .map(|index| u32::try_from(index + 1).unwrap_or(1))
}

fn try_literal_date(clause: &str) -> Option<Commencement> {
    let re = anchored(
        r"(?:en\s+toda\s+la\s+rep[uú]blica,?\s+)?(?:el\s+(?:d[ií]a\s+)?|a\s+partir\s+del?\s+)(\d{1,2}|primero|1[oº°])\s+de\s+([a-záéíóúñ]+)\s+del?\s+(\d{4})",
    );
    let captures = re.captures(clause)?;
    let day_word = captures[1].to_lowercase();
    let day = if matches!(day_word.as_str(), "primero" | "1o" | "1º" | "1°") {
        1
    } else {
        day_word.parse().ok()?
    };
    let month = month_number(&captures[2].to_lowercase())?;
    let year: i32 = captures[3].parse().ok()?;
    let date = NaiveDate::from_ymd_opt(year, month, day)?;
    Some(Commencement::Resolved {
        effective_from: date,
        pattern: "literal_date",
        matched_text: captures[0].to_owned(),
        provenance: None,
    })
}

/// Day counts written as Spanish words in commencement clauses, for both the
/// business-day and the calendar-day forms. Deliberately bounded: an
/// unrecognized word is reported, not guessed at.
const NUMBER_WORDS: [(&str, u64); 23] = [
    ("un", 1),
    ("uno", 1),
    ("dos", 2),
    ("tres", 3),
    ("cuatro", 4),
    ("cinco", 5),
    ("seis", 6),
    ("siete", 7),
    ("ocho", 8),
    ("nueve", 9),
    ("diez", 10),
    ("quince", 15),
    ("veinte", 20),
    ("veinticinco", 25),
    ("treinta", 30),
    ("cuarenta", 40),
    ("cuarenta y cinco", 45),
    ("cincuenta", 50),
    ("sesenta", 60),
    ("noventa", 90),
    ("cien", 100),
    ("ciento veinte", 120),
    ("ciento ochenta", 180),
];

fn number_word(word: &str) -> Option<u64> {
    NUMBER_WORDS
        .iter()
        .find(|(candidate, _)| *candidate == word)
        .map(|(_, value)| *value)
}

/// The subject of a plain commencement sentence.
const CLAUSE_SUBJECT: &str =
    r"(?:la\s+presente\s+ley|el\s+presente\s+(?:decreto|reglamento|c[oó]digo))";
const CLAUSE_TAIL: &str = r"(?:\s+en\s+el\s+diario\s+oficial\s+de\s+la\s+federaci[oó]n)?\s*\.?$";

/// Recognize the two complete clause forms this rule supports. The pattern is
/// anchored to the whole (whitespace-collapsed) provision text:
///
/// * `<subject> entrará en vigor a los N días hábiles siguientes (de su | al de
///   su | a su) publicación [en el Diario Oficial de la Federación].`
/// * `<subject> entrará en vigor (un mes después de su | al mes siguiente
///   (de su | al de su | a su)) publicación [en el Diario Oficial de la
///   Federación].`
///
/// Anything else, including a negation, a condition, a further sentence or a
/// second commencement event, is not a complete supported form and stays with
/// the generic path, which skips it for review.
fn try_complete_clause(
    text: &str,
    publication_date: NaiveDate,
    calendar: &BusinessDayCalendar,
) -> Option<Commencement> {
    let whole = text.split_whitespace().collect::<Vec<_>>().join(" ");
    let business = Regex::new(&format!(
        r"(?i)^{CLAUSE_SUBJECT}\s+entrar[áa]\s+en\s+vigor\s+a\s+los\s+(\d{{1,3}}|[a-záéíóúñ]+(?:\s+(?:y\s+)?[a-záéíóúñ]+)?)\s+d[ií]as\s+h[áa]biles\s+siguientes\s+(?:de\s+su|al\s+de\s+su|a\s+su)\s+publicaci[oó]n{CLAUSE_TAIL}"
    ))
    .expect("static regex");
    if let Some(captures) = business.captures(&whole) {
        let word = captures[1].to_lowercase();
        let count = word.parse::<u64>().ok().or_else(|| number_word(&word))?;
        return Some(
            match count_business_days_after(publication_date, count, calendar) {
                CountOutcome::Date { date, provenance } => Commencement::Resolved {
                    effective_from: date,
                    pattern: "business_days_lfpa_art28",
                    matched_text: whole.clone(),
                    provenance: Some(provenance),
                },
                CountOutcome::Insufficient {
                    earliest_possible,
                    reason,
                } => Commencement::Skipped {
                    reason: SkipReason::BusinessDaysCalendarInsufficient {
                        detail: match earliest_possible {
                            Some(date) => format!(
                                "{reason}; counting only weekends and statutory holidays the date could not be earlier than {date}, which is a lower bound and not a result"
                            ),
                            None => reason,
                        },
                    },
                    matched_text: Some(whole.clone()),
                },
            },
        );
    }
    let month = Regex::new(&format!(
        r"(?i)^{CLAUSE_SUBJECT}\s+entrar[áa]\s+en\s+vigor\s+(?:un\s+mes\s+despu[eé]s\s+de\s+su|al\s+mes\s+siguiente\s+(?:de\s+su|al\s+de\s+su|a\s+su))\s+publicaci[oó]n{CLAUSE_TAIL}"
    ))
    .expect("static regex");
    if month.is_match(&whole) {
        // The two readings of a month period differ by days; both fall within
        // two calendar months of publication, so only that bound is asserted.
        let latest = publication_date.checked_add_months(chrono::Months::new(2))?;
        return Some(Commencement::Bounded {
            latest,
            pattern: "month_bounded",
            matched_text: whole,
        });
    }
    None
}

/// `a los N días [naturales] [siguientes | después] (al de | al | de | a) [la
/// fecha de] su publicación`, as the whole clause. Only calendar days: a
/// `hábiles` count never matches here, accented or not.
fn try_relative_days(clause: &str, publication_date: NaiveDate) -> Option<Commencement> {
    let re = anchored(
        r"(?:a\s+los\s+)?(\d{1,3}|[a-záéíóúñ]+(?:\s+(?:y\s+)?[a-záéíóúñ]+)?)\s+d[ií]as(?:\s+naturales)?(?:\s+(?:siguientes|despu[eé]s))?\s+(?:al\s+de|al|de|a)\s+(?:la\s+fecha\s+de\s+)?(?:su|la)\s+publicaci[óo]n",
    );
    let captures = re.captures(clause)?;
    let count_word = captures[1].to_lowercase();
    let Some(count) = count_word
        .parse::<u64>()
        .ok()
        .or_else(|| number_word(&count_word))
    else {
        return Some(Commencement::Skipped {
            reason: SkipReason::UnrecognizedNumberWord {
                word: captures[1].to_owned(),
            },
            matched_text: Some(captures[0].to_owned()),
        });
    };
    let effective_from = publication_date.checked_add_days(Days::new(count))?;
    Some(Commencement::Resolved {
        effective_from,
        pattern: "relative_days",
        matched_text: captures[0].to_owned(),
        provenance: None,
    })
}

#[cfg(test)]
mod tests {
    use lex_core::{HeadingContext, InstrumentType, ReviewStatus};

    use super::*;

    fn date(y: i32, m: u32, d: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(y, m, d).expect("valid date")
    }

    fn instrument(publication_date: NaiveDate) -> Instrument {
        Instrument {
            schema_version: "0.1.0".to_owned(),
            id: "urn:lex-mx:federal:statute:muestra".to_owned(),
            jurisdiction: "federal".to_owned(),
            level: "federal".to_owned(),
            instrument_type: InstrumentType::Statute,
            official_title: "Ley de Muestra".to_owned(),
            short_name: "LM".to_owned(),
            operational_source: "diputados".to_owned(),
            formal_publication_source: "dof".to_owned(),
            publication_date: Some(publication_date),
            publication_period: None,
            latest_reform_date: None,
            retrieved_at: chrono::Utc::now(),
            source_url: "https://example.test/lm.pdf".parse().expect("valid url"),
            source_sha256: "0".repeat(64),
            extracted_text_sha256: "0".repeat(64),
            parser_version: "0.1.0".to_owned(),
            status: lex_core::InstrumentStatus::InForce,
            source_repeal_notice: None,
            provision_notes: Vec::new(),
            issuing_authorities: Vec::new(),
            formal_publication_url: None,
            formal_publication_code: None,
            formal_source_sha256: None,
            formal_extracted_text_sha256: None,
        }
    }

    fn provision(
        id: &str,
        provision_type: ProvisionType,
        number: &str,
        text: &str,
        temporal_status: TemporalStatus,
        review_status: ReviewStatus,
    ) -> Provision {
        Provision {
            numbering_basis: None,
            schema_version: "0.1.0".to_owned(),
            id: format!("urn:lex-mx:federal:statute:muestra:{provision_type:?}:{id}")
                .to_lowercase(),
            instrument_id: "urn:lex-mx:federal:statute:muestra".to_owned(),
            provision_type,
            label: format!("Artículo {number}"),
            number: number.to_owned(),
            heading_context: HeadingContext {
                libro: None,
                title: None,
                chapter: None,
                section: None,
                apartado: None,
            },
            text: text.to_owned(),
            publication_date: Some(date(1990, 1, 1)),
            publication_period: None,
            effective_from: None,
            effective_to: None,
            temporal_status,
            temporal_basis: None,
            temporal_confidence: None,
            review_status,
            transitory_effects: Vec::new(),
            amendment_marks: Vec::new(),
            repeals: Vec::new(),
            commencement_condition: None,
        }
    }

    fn article(id: &str, text: &str) -> Provision {
        provision(
            id,
            ProvisionType::Article,
            id,
            text,
            TemporalStatus::Unknown,
            ReviewStatus::NotAnalyzed,
        )
    }

    fn repealed_article(id: &str) -> Provision {
        provision(
            id,
            ProvisionType::Article,
            id,
            "(Se deroga).",
            TemporalStatus::Repealed,
            ReviewStatus::NotAnalyzed,
        )
    }

    fn transitory(id: &str, text: &str) -> Provision {
        provision(
            id,
            ProvisionType::Transitory,
            id,
            text,
            TemporalStatus::Unknown,
            ReviewStatus::NotAnalyzed,
        )
    }

    fn resolved_date(outcome: &super::DerivationOutcome) -> Option<NaiveDate> {
        match outcome.commencement {
            Some(Commencement::Resolved { effective_from, .. }) => Some(effective_from),
            _ => None,
        }
    }

    #[test]
    fn repair_partial_repeals_reclassifies_only_deterministic_partial_notes() {
        let mut partial = article(
            "16",
            "(Se deroga el primer párrafo). Párrafo derogado DOF 15-06-2007\n\nTexto vigente.",
        );
        partial.temporal_status = TemporalStatus::Repealed;
        partial.temporal_basis = Some(Basis::DeterministicRule);
        partial.review_status = ReviewStatus::MachineAccepted;
        let mut whole = article("2", "(Se deroga)");
        whole.temporal_status = TemporalStatus::Repealed;
        whole.temporal_basis = Some(Basis::DeterministicRule);
        whole.review_status = ReviewStatus::MachineAccepted;
        let mut reviewed = partial.clone();
        reviewed.review_status = ReviewStatus::LawyerVerified;
        let mut provisions = vec![partial, whole, reviewed];

        assert_eq!(repair_partial_repeals(&mut provisions), 1);
        assert_eq!(
            provisions[0].temporal_status,
            TemporalStatus::PartiallyRepealed
        );
        assert_eq!(provisions[0].repeals[0].ordinals, ["1"]);
        assert_eq!(provisions[1].temporal_status, TemporalStatus::Repealed);
        assert_eq!(provisions[2].temporal_status, TemporalStatus::Repealed);
        assert_eq!(repair_partial_repeals(&mut provisions), 0);
    }

    #[test]
    fn a_parsed_partial_repeal_is_promoted_only_while_its_note_matches_its_scope() {
        let inst = instrument(date(2002, 12, 30));
        let text =
            "(Se deroga el primer párrafo). Párrafo derogado DOF 15-06-2007\n\nTexto vigente.";
        let mut partial = article("16", text);
        partial.temporal_status = TemporalStatus::PartiallyRepealed;
        partial.repeals = crate::initial::initial_repeals(text);
        let mut widened = partial.clone();
        widened.id.push_str("-widened");
        widened.repeals[0].ordinals.push("2".to_owned());
        let mut reviewed = partial.clone();
        reviewed.id.push_str("-reviewed");
        reviewed.review_status = ReviewStatus::LawyerVerified;
        let mut provisions = vec![partial, widened, reviewed];

        let outcome = derive_article_temporal_determinations(&inst, &provisions, date(2026, 1, 1));
        let promoted: Vec<&str> = outcome
            .determinations
            .iter()
            .map(|determination| determination.provision_id.as_str())
            .collect();
        assert_eq!(promoted, [provisions[0].id.as_str()]);
        let determination = &outcome.determinations[0];
        assert_eq!(
            determination.temporal_status,
            TemporalStatus::PartiallyRepealed
        );
        assert_eq!(determination.effective_from, None);
        assert!((determination.confidence - 1.0).abs() < f32::EPSILON);

        lex_core::apply_temporal_determinations(&mut provisions, &outcome.determinations);
        assert_eq!(provisions[0].review_status, ReviewStatus::MachineAccepted);
        assert_eq!(provisions[0].temporal_basis, Some(Basis::DeterministicRule));
        assert_eq!(provisions[0].repeals, crate::initial::initial_repeals(text));
        // Deriving again changes nothing: the article is no longer untouched.
        let again = derive_article_temporal_determinations(&inst, &provisions, date(2026, 1, 1));
        assert!(again.determinations.is_empty());
    }

    #[test]
    fn unamended_article_takes_the_original_date_and_an_amended_one_stays_unset() {
        let inst = instrument(date(1995, 12, 22));
        let transitory_one = transitory(
            "primero",
            "La presente Ley entrará en vigor al día siguiente de su publicación.",
        );
        let mut footnoted = article("12", "Texto reformado con nota al pie.");
        footnoted.amendment_marks = vec![3];
        let provisions = vec![
            article("1", "Texto original sin reformas."),
            article("10-Bis", "Texto.\n\nArtículo adicionado DOF 08-06-2016"),
            footnoted,
            transitory_one,
        ];
        let outcome = derive_article_temporal_determinations(&inst, &provisions, date(2026, 1, 1));
        assert_eq!(resolved_date(&outcome), Some(date(1995, 12, 23)));
        let date_of = |suffix: &str| {
            outcome
                .determinations
                .iter()
                .find(|item| item.provision_id.ends_with(suffix))
                .unwrap_or_else(|| panic!("determination for {suffix}"))
                .effective_from
        };
        assert_eq!(date_of(":1"), Some(date(1995, 12, 23)));
        assert_eq!(date_of(":10-bis"), None);
        assert_eq!(date_of(":12"), None);
    }

    #[test]
    fn repair_restores_unamended_dates_clears_amended_ones_and_preserves_review() {
        let inst = instrument(date(1995, 12, 22));
        let mut provisions = vec![
            article("1", "Texto original sin reformas."),
            article("10-Bis", "Texto.\n\nArtículo adicionado DOF 08-06-2016"),
            transitory(
                "primero",
                "La presente Ley entrará en vigor al día siguiente de su publicación.",
            ),
        ];
        let outcome = derive_article_temporal_determinations(&inst, &provisions, date(2026, 1, 1));
        lex_core::apply_temporal_determinations(&mut provisions, &outcome.determinations);
        // State written by the v1/v2 rules: every article dated, or none.
        provisions[0].effective_from = None;
        provisions[1].effective_from = Some(date(1995, 12, 23));
        let mut human = provisions[1].clone();
        human.id = "human-reviewed".to_owned();
        human.temporal_basis = Some(Basis::LawyerVerified);
        human.review_status = ReviewStatus::LawyerVerified;
        provisions.push(human.clone());
        let mut other_date = provisions[0].clone();
        other_date.id = "other-date".to_owned();
        other_date.effective_from = Some(date(2001, 1, 1));
        provisions.push(other_date);

        assert_eq!(
            super::repair_article_dates(&inst, &mut provisions, &BusinessDayCalendar::default()),
            2
        );
        assert_eq!(provisions[0].effective_from, Some(date(1995, 12, 23)));
        assert_eq!(provisions[1].effective_from, None);
        // Human review and an unrelated date are untouched.
        assert_eq!(provisions[3].effective_from, human.effective_from);
        assert_eq!(provisions[3].review_status, ReviewStatus::LawyerVerified);
        assert_eq!(provisions[4].effective_from, Some(date(2001, 1, 1)));
        assert_eq!(
            super::repair_article_dates(&inst, &mut provisions, &BusinessDayCalendar::default()),
            0
        );
    }

    #[test]
    #[allow(clippy::float_cmp)] // 1.0 is the deterministic rule's exact contract.
    fn day_after_publication_classifies_all_non_repealed_articles() {
        let inst = instrument(date(1990, 1, 1));
        let provisions = vec![
            article("1", "Texto del artículo primero."),
            repealed_article("2"),
            transitory(
                "primero",
                "La presente Ley entrará en vigor al día siguiente de su publicación en el Diario Oficial de la Federación.",
            ),
        ];
        let outcome = derive_article_temporal_determinations(&inst, &provisions, date(2026, 1, 1));
        assert_eq!(outcome.determinations.len(), 2);
        let art1 = outcome
            .determinations
            .iter()
            .find(|d| d.provision_id.ends_with(":1"))
            .expect("article 1 determination");
        assert_eq!(art1.temporal_status, TemporalStatus::Effective);
        assert_eq!(art1.effective_from, Some(date(1990, 1, 2)));
        assert_eq!(resolved_date(&outcome), Some(date(1990, 1, 2)));
        assert_eq!(art1.basis, Basis::DeterministicRule);
        assert!(!art1.review_required);
        let art2 = outcome
            .determinations
            .iter()
            .find(|d| d.provision_id.ends_with(":2"))
            .expect("article 2 determination");
        assert_eq!(art2.temporal_status, TemporalStatus::Repealed);
        assert_eq!(art2.confidence, 1.0);
        assert!(matches!(
            outcome.commencement,
            Some(Commencement::Resolved {
                pattern: "day_after_publication",
                ..
            })
        ));
    }

    #[test]
    fn reversed_word_order_still_resolves() {
        let inst = instrument(date(2001, 3, 15));
        let provisions = vec![
            article("1", "Texto."),
            transitory(
                "unico",
                "El presente Reglamento entrará en vigor el día siguiente al de su publicación en el Diario Oficial de la Federación.",
            ),
        ];
        let outcome = derive_article_temporal_determinations(&inst, &provisions, date(2026, 1, 1));
        assert_eq!(resolved_date(&outcome), Some(date(2001, 3, 16)));
    }

    #[test]
    fn literal_date_with_word_day_resolves() {
        let inst = instrument(date(1980, 12, 30));
        let provisions = vec![
            article("1", "Texto."),
            transitory(
                "primero",
                "Esta Ley entrará en vigor en toda la República, el día primero de enero de 1981.",
            ),
        ];
        let outcome = derive_article_temporal_determinations(&inst, &provisions, date(2026, 1, 1));
        assert_eq!(resolved_date(&outcome), Some(date(1981, 1, 1)));
        assert!(matches!(
            outcome.commencement,
            Some(Commencement::Resolved {
                pattern: "literal_date",
                ..
            })
        ));
    }

    #[test]
    fn relative_days_with_digit_count_resolves() {
        let inst = instrument(date(2010, 6, 1));
        let provisions = vec![
            article("1", "Texto."),
            transitory(
                "unico",
                "El presente ordenamiento entrará en vigor a los 30 días naturales siguientes de su publicación en el Diario Oficial de la Federación.",
            ),
        ];
        let outcome = derive_article_temporal_determinations(&inst, &provisions, date(2026, 1, 1));
        assert_eq!(resolved_date(&outcome), Some(date(2010, 7, 1)));
    }

    #[test]
    fn relative_days_with_word_count_resolves() {
        let inst = instrument(date(1980, 12, 30));
        let provisions = vec![
            article("1", "Texto."),
            transitory(
                "primero",
                "La presente Ley entrará en vigor a los noventa días de su publicación en el Diario Oficial de la Federación.",
            ),
        ];
        let outcome = derive_article_temporal_determinations(&inst, &provisions, date(2026, 1, 1));
        assert_eq!(resolved_date(&outcome), Some(date(1981, 3, 30)));
    }

    fn bounded_outcome(outcome: &DerivationOutcome) -> Option<(NaiveDate, &'static str)> {
        match &outcome.commencement {
            Some(Commencement::Bounded {
                latest, pattern, ..
            }) => Some((*latest, *pattern)),
            _ => None,
        }
    }

    fn calendar_for(year: i32, suspended: &[(NaiveDate, NaiveDate)]) -> BusinessDayCalendar {
        use crate::business_days::{DateRange, FixedHoliday, SexennialHoliday, StatuteVersion};
        use chrono::Weekday;
        let range = |&(from, to): &(NaiveDate, NaiveDate)| DateRange {
            from,
            to,
            source: "test notice review".to_owned(),
        };
        BusinessDayCalendar {
            rule_id: "lfpa-art-28".to_owned(),
            versions: vec![StatuteVersion {
                id: "lfpa-art28-original".to_owned(),
                source: "test record".to_owned(),
                applies_from: date(1995, 6, 1),
                applies_to: None,
                weekend_days: vec![Weekday::Sat, Weekday::Sun],
                fixed_holidays: [
                    (1, 1),
                    (2, 5),
                    (3, 21),
                    (5, 1),
                    (5, 5),
                    (9, 1),
                    (9, 16),
                    (11, 20),
                    (12, 25),
                ]
                .iter()
                .map(|&(month, day)| FixedHoliday { month, day })
                .collect(),
                sexennial_holidays: vec![SexennialHoliday {
                    month: 12,
                    day: 1,
                    first_year: 1994,
                    last_year: 2018,
                }],
            }],
            suspensions: suspended.iter().map(range).collect(),
            notice_coverage: vec![range(&(date(year, 1, 1), date(year, 12, 31)))],
        }
    }

    const FIVE_BUSINESS_DAYS: &str = "La presente Ley entrará en vigor a los 5 días hábiles siguientes a su publicación en el Diario Oficial de la Federación.";

    #[test]
    fn a_plain_business_day_clause_without_evidenced_calendar_inputs_stays_unresolved() {
        // lbogm, lfea and reg-lgeepa-maaa: no suspension notices are evidenced for their windows
        for text in [
            "La presente Ley entrará en vigor a los treinta días hábiles siguientes de su publicación en el Diario Oficial de la Federación.",
            "La presente Ley entrará en vigor a los ciento veinte días hábiles siguientes al de su publicación en el Diario Oficial de la Federación.",
            "El presente Decreto entrará en vigor a los sesenta días hábiles siguientes a su publicación en el Diario Oficial de la Federación.",
        ] {
            let inst = instrument(date(2005, 1, 1));
            let provisions = vec![article("1", "Texto."), transitory("primero", text)];
            let outcome =
                derive_article_temporal_determinations(&inst, &provisions, date(2026, 10, 27));
            assert!(outcome.determinations.is_empty(), "{text}");
            match &outcome.commencement {
                Some(Commencement::Skipped {
                    reason: SkipReason::BusinessDaysCalendarInsufficient { detail },
                    ..
                }) => {
                    assert!(
                        detail.contains("no statutory text version is supplied"),
                        "{detail}"
                    );
                }
                other => panic!("{text}: {other:?}"),
            }
        }
        // a text version without notice coverage reports only a lower bound, not a result
        let mut thin = calendar_for(2005, &[]);
        thin.notice_coverage.clear();
        let inst = instrument(date(2005, 1, 3));
        let provisions = vec![
            article("1", "Texto."),
            transitory("primero", FIVE_BUSINESS_DAYS),
        ];
        let outcome = derive_article_temporal_determinations_with_calendar(
            &inst,
            &provisions,
            date(2026, 10, 27),
            &thin,
        );
        assert!(outcome.determinations.is_empty());
        match &outcome.commencement {
            Some(Commencement::Skipped {
                reason: SkipReason::BusinessDaysCalendarInsufficient { detail },
                ..
            }) => {
                assert!(detail.contains("lower bound and not a result"), "{detail}");
                assert!(detail.contains("not evidenced"), "{detail}");
            }
            other => panic!("{other:?}"),
        }
    }

    #[test]
    fn a_plain_business_day_clause_with_evidenced_inputs_resolves_to_the_counted_date() {
        let inst = instrument(date(2023, 6, 1));
        let provisions = vec![
            article("1", "Texto."),
            transitory("primero", FIVE_BUSINESS_DAYS),
        ];
        let outcome = derive_article_temporal_determinations_with_calendar(
            &inst,
            &provisions,
            date(2026, 10, 27),
            &calendar_for(2023, &[]),
        );
        assert_eq!(resolved_date(&outcome), Some(date(2023, 6, 8)));
        assert_eq!(outcome.determinations.len(), 1);
        let d = &outcome.determinations[0];
        assert_eq!(
            (d.temporal_status.clone(), d.effective_from),
            (TemporalStatus::Effective, Some(date(2023, 6, 8)))
        );
        assert!(
            d.supporting_text
                .iter()
                .any(|line| line.contains("lfpa-art-28") && line.contains("lfpa-art28-original")),
            "{:?}",
            d.supporting_text
        );
        // a supplied suspension moves the date; the same clause with the notice absent from coverage does not resolve
        let suspended = calendar_for(2023, &[(date(2023, 6, 5), date(2023, 6, 6))]);
        let moved = derive_article_temporal_determinations_with_calendar(
            &inst,
            &provisions,
            date(2026, 10, 27),
            &suspended,
        );
        assert_eq!(resolved_date(&moved), Some(date(2023, 6, 12)));
        let mut thin = calendar_for(2023, &[]);
        thin.notice_coverage.clear();
        let unresolved = derive_article_temporal_determinations_with_calendar(
            &inst,
            &provisions,
            date(2026, 10, 27),
            &thin,
        );
        assert!(unresolved.determinations.is_empty());
    }

    #[test]
    fn a_month_bound_that_may_not_have_passed_decides_nothing() {
        let inst = instrument(date(2026, 9, 1));
        let provisions = vec![
            article("1", "Texto."),
            transitory(
                "primero",
                "La presente Ley entrará en vigor un mes después de su publicación en el Diario Oficial de la Federación.",
            ),
        ];
        // 2026-09-01 + 2 months is 2026-11-01: still in the future on 2026-10-01
        let outcome = derive_article_temporal_determinations(&inst, &provisions, date(2026, 10, 1));
        assert!(bounded_outcome(&outcome).is_some());
        assert!(outcome.determinations.is_empty());
    }

    #[test]
    fn conditional_negated_and_multi_event_clauses_are_retained_for_review() {
        // Codex review B-002 probes and the two known outliers: none may become a deterministic result
        let business = "La presente Ley entrará en vigor a los treinta días hábiles siguientes de su publicación en el Diario Oficial de la Federación";
        let month = "La presente Ley entrará en vigor un mes después de su publicación en el Diario Oficial de la Federación";
        let cases = [
            ("conditional business day", format!("{business}, siempre que el Consejo haya emitido la autorización correspondiente.")),
            ("conditional month", format!("{month}, siempre y cuando se haya expedido el reglamento correspondiente.")),
            ("negated month", "La presente Ley no entrará en vigor un mes después de su publicación en el Diario Oficial de la Federación.".to_owned()),
            ("negated business day", "La presente Ley no entrará en vigor a los treinta días hábiles siguientes de su publicación en el Diario Oficial de la Federación.".to_owned()),
            ("negated calendar-day count", "La presente Ley no entrará en vigor a los treinta días siguientes de su publicación en el Diario Oficial de la Federación.".to_owned()),
            ("ligie: runs from an authority's determination", "El presente Decreto entrará en vigor a los diez días hábiles siguientes a aquél en el que el Servicio de Administración Tributaria determine que los sistemas se encuentran listos para operar.".to_owned()),
            ("reg-lfppi: a second commencement event", "El presente Decreto entrará en vigor a los sesenta días hábiles siguientes al de su publicación en el Diario Oficial de la Federación. El procedimiento en línea del Capítulo VII entrará en vigor al día siguiente al que se publique el acuerdo.".to_owned()),
            ("lfdc: a further sentence after the clause", format!("{month}. Las autoridades fiscales realizarán una campaña masiva para difundir las nuevas disposiciones.")),
            ("plural subject is not a supported form", "Las presentes disposiciones entrarán en vigor a los treinta días hábiles siguientes a su publicación en el Diario Oficial de la Federación.".to_owned()),
        ];
        let full = calendar_for(2005, &[]);
        for (name, text) in cases {
            let inst = instrument(date(2005, 1, 1));
            let provisions = vec![article("1", "Texto."), transitory("primero", &text)];
            let outcome = derive_article_temporal_determinations_with_calendar(
                &inst,
                &provisions,
                date(2026, 10, 27),
                &full,
            );
            assert!(
                outcome.determinations.is_empty(),
                "{name}: {:?}",
                outcome.determinations
            );
            assert!(
                matches!(outcome.commencement, Some(Commencement::Skipped { .. })),
                "{name}: {:?}",
                outcome.commencement
            );
        }
    }

    #[test]
    fn an_amended_article_keeps_no_start_date_under_a_bounded_commencement() {
        let inst = instrument(date(2005, 1, 1));
        let mut amended = article("2", "Texto reformado DOF 10-05-2012");
        amended.amendment_marks = vec![1];
        let provisions = vec![
            article("1", "Texto original."),
            amended,
            transitory(
                "primero",
                "La presente Ley entrará en vigor un mes después de su publicación en el Diario Oficial de la Federación.",
            ),
        ];
        let outcome =
            derive_article_temporal_determinations(&inst, &provisions, date(2026, 10, 27));
        assert_eq!(bounded_outcome(&outcome).unwrap().1, "month_bounded");
        assert_eq!(outcome.determinations.len(), 2);
        assert!(
            outcome
                .determinations
                .iter()
                .all(|d| d.temporal_status == TemporalStatus::Effective
                    && d.effective_from.is_none())
        );
    }

    #[test]
    fn month_unit_relative_period_is_skipped_as_unsupported() {
        let inst = instrument(date(2005, 1, 1));
        let provisions = vec![
            article("1", "Texto."),
            transitory(
                "unico",
                "La presente Ley entrará en vigor a los dos meses siguientes a su publicación en el Diario Oficial de la Federación.",
            ),
        ];
        let outcome = derive_article_temporal_determinations(&inst, &provisions, date(2026, 1, 1));
        assert!(outcome.determinations.is_empty());
        assert!(matches!(
            outcome.commencement,
            Some(Commencement::Skipped {
                reason: SkipReason::UnsupportedRelativePeriodUnit { .. },
                ..
            })
        ));
    }

    #[test]
    fn exception_clause_is_skipped_even_with_a_recognizable_date() {
        // LIEPS's real Primero: a clean day-after-publication clause
        // immediately followed by a carve-out for specific provisions.
        let inst = instrument(date(1980, 12, 30));
        let provisions = vec![
            article("1", "Texto."),
            transitory(
                "primero",
                "Esta Ley entrará en vigor en toda la República, el día primero de enero de 1981, con excepción de las disposiciones contenidas en los incisos A, B y C de la fracción I, del artículo 2o. de este ordenamiento.",
            ),
        ];
        let outcome = derive_article_temporal_determinations(&inst, &provisions, date(2026, 1, 1));
        assert!(outcome.determinations.is_empty());
        assert!(matches!(
            outcome.commencement,
            Some(Commencement::Skipped {
                reason: SkipReason::ExceptionOrConditionalLanguage { .. },
                ..
            })
        ));
    }

    #[test]
    fn multiple_commencement_clauses_are_skipped() {
        let inst = instrument(date(2000, 1, 1));
        let provisions = vec![
            article("1", "Texto."),
            transitory(
                "primero",
                "La presente Ley entrará en vigor al día siguiente de su publicación en el Diario Oficial de la Federación.",
            ),
            transitory(
                "segundo",
                "El Título Cuarto entrará en vigor el 1 de enero de 2005.",
            ),
        ];
        let outcome = derive_article_temporal_determinations(&inst, &provisions, date(2026, 1, 1));
        assert!(outcome.determinations.is_empty());
        assert!(matches!(
            outcome.commencement,
            Some(Commencement::Skipped {
                reason: SkipReason::MultipleCommencementClauses { count: 2 },
                ..
            })
        ));
    }

    #[test]
    fn no_commencement_clause_is_skipped() {
        let inst = instrument(date(2000, 1, 1));
        let provisions = vec![
            article("1", "Texto."),
            transitory("primero", "Se derogan las disposiciones que se opongan."),
        ];
        let outcome = derive_article_temporal_determinations(&inst, &provisions, date(2026, 1, 1));
        assert!(outcome.determinations.is_empty());
        assert!(matches!(
            outcome.commencement,
            Some(Commencement::Skipped {
                reason: SkipReason::NoCommencementClause,
                ..
            })
        ));
    }

    #[test]
    fn no_ordinary_transitories_reports_no_commencement_and_still_promotes_repeals() {
        let inst = instrument(date(2000, 1, 1));
        let provisions = vec![repealed_article("1")];
        let outcome = derive_article_temporal_determinations(&inst, &provisions, date(2026, 1, 1));
        assert_eq!(outcome.determinations.len(), 1);
        assert_eq!(
            outcome.determinations[0].temporal_status,
            TemporalStatus::Repealed
        );
        assert!(outcome.commencement.is_none());
    }

    #[test]
    fn future_publication_date_yields_future_effective() {
        let inst = instrument(date(2099, 1, 1));
        let provisions = vec![
            article("1", "Texto."),
            transitory(
                "unico",
                "La presente Ley entrará en vigor al día siguiente de su publicación en el Diario Oficial de la Federación.",
            ),
        ];
        let outcome = derive_article_temporal_determinations(&inst, &provisions, date(2026, 1, 1));
        assert_eq!(
            outcome.determinations[0].temporal_status,
            TemporalStatus::FutureEffective
        );
    }

    #[test]
    fn already_analyzed_articles_are_never_touched() {
        let inst = instrument(date(1990, 1, 1));
        let mut already_reviewed = article("1", "Texto.");
        already_reviewed.review_status = ReviewStatus::LawyerVerified;
        already_reviewed.temporal_status = TemporalStatus::PublishedNotEffective;
        let provisions = vec![
            already_reviewed,
            transitory(
                "unico",
                "La presente Ley entrará en vigor al día siguiente de su publicación en el Diario Oficial de la Federación.",
            ),
        ];
        let outcome = derive_article_temporal_determinations(&inst, &provisions, date(2026, 1, 1));
        assert!(
            outcome.determinations.is_empty(),
            "must never emit a determination for an already-reviewed provision"
        );
    }

    #[test]
    fn del_year_contraction_resolves() {
        // lfpca's real Primero: "del 2006" (de+el), not "de 2006".
        let inst = instrument(date(2005, 12, 1));
        let provisions = vec![
            article("1", "Texto."),
            transitory(
                "primero",
                "La presente Ley entrará en vigor en toda la República el día 1o. de enero del 2006.",
            ),
        ];
        let outcome = derive_article_temporal_determinations(&inst, &provisions, date(2026, 1, 1));
        assert_eq!(resolved_date(&outcome), Some(date(2006, 1, 1)));
    }

    #[test]
    fn one_month_after_publication_is_bounded_and_other_month_periods_stay_unsupported() {
        // lfdc: "un mes después"; reg-lfpc-art122: "al mes siguiente" (two readings, one bound).
        let inst = instrument(date(2005, 1, 1));
        for text in [
            "La presente Ley entrará en vigor un mes después de su publicación en el Diario Oficial de la Federación.",
            "El presente Reglamento entrará en vigor al mes siguiente de su publicación en el Diario Oficial de la Federación.",
        ] {
            let provisions = vec![article("1", "Texto."), transitory("primero", text)];
            let outcome =
                derive_article_temporal_determinations(&inst, &provisions, date(2026, 10, 27));
            let (latest, pattern) = bounded_outcome(&outcome).expect(text);
            assert_eq!((latest, pattern), (date(2005, 3, 1), "month_bounded"));
            assert_eq!(outcome.determinations.len(), 1);
        }
        // "dos meses" and any other unit are still reported, not guessed
        let provisions = vec![
            article("1", "Texto."),
            transitory(
                "primero",
                "La presente Ley entrará en vigor a los dos meses siguientes a su publicación en el Diario Oficial de la Federación.",
            ),
        ];
        let outcome =
            derive_article_temporal_determinations(&inst, &provisions, date(2026, 10, 27));
        assert!(matches!(
            outcome.commencement,
            Some(Commencement::Skipped {
                reason: SkipReason::UnsupportedRelativePeriodUnit { .. },
                ..
            })
        ));
    }

    #[test]
    fn contados_a_partir_del_dia_siguiente_wrapper_is_left_unrecognized() {
        // lfaebsp's real Primero: the day count is offset by an extra "day
        // after" wrapper this module deliberately does not attempt, rather
        // than risk computing a date one day too early.
        let inst = instrument(date(2010, 1, 1));
        let provisions = vec![
            article("1", "Texto."),
            transitory(
                "primero",
                "El presente Decreto entrará en vigor a los 180 días naturales contados a partir del día siguiente al de su publicación en el Diario Oficial de la Federación.",
            ),
        ];
        let outcome = derive_article_temporal_determinations(&inst, &provisions, date(2026, 1, 1));
        assert!(outcome.determinations.is_empty());
        assert!(matches!(
            outcome.commencement,
            Some(Commencement::Skipped {
                reason: SkipReason::UnrecognizedCommencementPattern,
                ..
            })
        ));
    }

    #[test]
    fn spelled_out_year_is_left_unrecognized() {
        let inst = instrument(date(1984, 6, 1));
        let provisions = vec![
            article("1", "Texto."),
            transitory(
                "unico",
                "La presente Ley entrará en vigor a partir del primero de julio de mil novecientos ochenta y cuatro.",
            ),
        ];
        let outcome = derive_article_temporal_determinations(&inst, &provisions, date(2026, 1, 1));
        assert!(outcome.determinations.is_empty());
        assert!(matches!(
            outcome.commencement,
            Some(Commencement::Skipped {
                reason: SkipReason::UnrecognizedCommencementPattern,
                ..
            })
        ));
    }
    #[test]
    fn nonexact_publication_never_guesses_an_endpoint_or_changes_dates() {
        let mut inst = instrument(date(1912, 1, 1));
        inst.publication_date = None;
        let mut provisions = vec![provision(
            "1",
            ProvisionType::Transitory,
            "UNICO",
            "La Ley entrará en vigor al día siguiente de su publicación.",
            TemporalStatus::Unknown,
            ReviewStatus::NotAnalyzed,
        )];
        let before = format!("{provisions:?}");
        let result = derive_article_temporal_determinations(&inst, &provisions, date(2026, 1, 1));
        assert!(result.determinations.is_empty());
        assert!(matches!(
            result.commencement,
            Some(Commencement::Skipped {
                reason: SkipReason::PublicationNotExact,
                ..
            })
        ));
        assert_eq!(
            repair_article_dates(&inst, &mut provisions, &BusinessDayCalendar::default()),
            0
        );
        assert_eq!(format!("{provisions:?}"), before);
    }

    fn commencement_of(text: &str, calendar: &BusinessDayCalendar) -> DerivationOutcome {
        let inst = instrument(date(2020, 1, 1));
        let provisions = vec![article("1", "Texto."), transitory("primero", text)];
        derive_article_temporal_determinations_with_calendar(
            &inst,
            &provisions,
            date(2026, 11, 17),
            calendar,
        )
    }

    fn skip_reason(outcome: &DerivationOutcome) -> SkipReason {
        match &outcome.commencement {
            Some(Commencement::Skipped { reason, .. }) => reason.clone(),
            other => panic!("expected a skip, got {other:?}"),
        }
    }

    #[test]
    fn an_unaccented_business_day_clause_never_becomes_a_calendar_day_date() {
        // Codex audit 2026-11-17, C1: this plural, unaccented clause used to
        // resolve through the relative-days fragment to 2020-01-31.
        let full = calendar_for(2020, &[]);
        for text in [
            "Las presentes disposiciones ENTRARAN EN VIGOR A LOS TREINTA DIAS HABILES siguientes a su publicacion.",
            "Las presentes disposiciones entrarán en vigor a los treinta días habiles siguientes a su publicación en el Diario Oficial de la Federación.",
            "El presente Acuerdo entrará en vigor a los diez dias habiles siguientes de su publicación.",
        ] {
            for calendar in [&BusinessDayCalendar::default(), &full] {
                let outcome = commencement_of(text, calendar);
                assert!(outcome.determinations.is_empty(), "{text}");
                assert_eq!(
                    skip_reason(&outcome),
                    SkipReason::BusinessDaysPeriod,
                    "{text}"
                );
            }
        }
    }

    #[test]
    fn legacy_date_paths_accept_only_the_whole_clause() {
        // Codex review 2026-10-28, deferred limit 1: a condition or a further
        // sentence after a recognizable date fragment goes to review.
        let none = BusinessDayCalendar::default();
        for text in [
            "La presente Ley entrará en vigor al día siguiente de su publicación en el Diario Oficial de la Federación, siempre que se haya expedido el reglamento.",
            "La presente Ley entrará en vigor a los treinta días siguientes de su publicación, una vez que la Secretaría lo determine.",
            "El presente Reglamento entrará en vigor el 1o. de enero de 2021. Las solicitudes en trámite se resolverán conforme a las disposiciones anteriores.",
            // a declaration before the verb (the shape of lnsijpa's Vigencia clause)
            "Vigencia Para los efectos del artículo segundo transitorio se declara que la presente legislación incorpora el sistema y entrará en vigor el 18 de junio de 2016.",
            // a second verb before the commencement (reg-senado)
            "El presente Reglamento se publicará en el Diario Oficial de la Federación y entrará en vigor el primero de septiembre de 2010.",
        ] {
            let outcome = commencement_of(text, &none);
            assert!(outcome.determinations.is_empty(), "{text}");
            assert_eq!(
                skip_reason(&outcome),
                SkipReason::UnrecognizedCommencementPattern,
                "{text}"
            );
        }
    }

    #[test]
    fn negation_is_matched_as_words_across_any_whitespace() {
        // Codex audit 2026-11-17, C3, and deferred limit 1: doubled spaces and
        // line breaks still negate; a word ending in "no" does not.
        let none = BusinessDayCalendar::default();
        for text in [
            "La presente Ley no  entrará en vigor al día siguiente de su publicación en el Diario Oficial de la Federación.",
            "La presente Ley no\nentrará en vigor al día siguiente de su publicación en el Diario Oficial de la Federación.",
            "La presente Ley NO ENTRARÁ EN VIGOR al día siguiente de su publicación.",
            "Las presentes disposiciones no entraran en vigor el 1o. de enero de 2021.",
        ] {
            let outcome = commencement_of(text, &none);
            assert!(outcome.determinations.is_empty(), "{text}");
            assert!(
                matches!(
                    skip_reason(&outcome),
                    SkipReason::ExceptionOrConditionalLanguage { ref marker } if marker.starts_with("no entra")
                ),
                "{text}: {:?}",
                outcome.commencement
            );
        }
        let interno = commencement_of(
            "El Reglamento Interno entrará en vigor al día siguiente de su publicación.",
            &none,
        );
        assert!(
            !matches!(
                skip_reason(&interno),
                SkipReason::ExceptionOrConditionalLanguage { .. }
            ),
            "{:?}",
            interno.commencement
        );
    }

    #[test]
    fn qualifiers_are_matched_without_accents_or_case() {
        let none = BusinessDayCalendar::default();
        for (text, marker) in [
            (
                "La presente Ley entrará en vigor al día siguiente de su publicación, CON EXCEPCION del Capítulo III.",
                "con excepción",
            ),
            (
                "La presente Ley entrará en vigor a mas tardar el 1o. de enero de 2021.",
                "a más tardar",
            ),
            (
                "El presente Decreto entrará en vigor gradualmente conforme al calendario.",
                "gradual",
            ),
        ] {
            let outcome = commencement_of(text, &none);
            assert_eq!(
                skip_reason(&outcome),
                SkipReason::ExceptionOrConditionalLanguage {
                    marker: marker.to_owned()
                },
                "{text}"
            );
        }
        // "salvo" is a word, not a prefix of another word
        let outcome = commencement_of(
            "La presente Ley entrará en vigor al día siguiente de su publicación en el Diario Oficial de la Federación y abroga la Ley de Salvoconductos.",
            &none,
        );
        assert_eq!(resolved_date(&outcome), Some(date(2020, 1, 2)));
    }

    #[test]
    fn the_corpus_clause_shapes_still_resolve() {
        // One case per accepted legacy shape found in the 2026-11-17 corpus
        // census, including the source-extraction leftovers the tail allows.
        let none = BusinessDayCalendar::default();
        for (text, expected) in [
            (
                "Esta Ley entrará en vigor el día siguiente de su publicación en el \"Diario Oficial\".",
                date(2020, 1, 2),
            ),
            (
                "El presente Reglamento entrará en vigor al día siguiente de su publicación en el Diario Oficial.",
                date(2020, 1, 2),
            ),
            (
                "La presente Ley Federal de Sanidad Animal entrará en vigor al día siguiente de su publicación en el Diario Oficial de la Federación.",
                date(2020, 1, 2),
            ),
            (
                "El presente \"Reglamento\" entrará en vigor al día siguiente de su publicación en el Diario Oficial de la Federación.",
                date(2020, 1, 2),
            ),
            (
                "Esta Ley entrará en vigor al día siguiente de su publicación en el \"Diario Oficial\" de la Federación y abroga la Ley de Extradición de 1o. de mayo de 1897.",
                date(2020, 1, 2),
            ),
            (
                "La presente Ley entrará en vigor tres días después de su publicación en el Diario Oficial de la Federación. Miguel Moreno Padilla, D. S.- Eugenio Prado, S. P.- Melquiades Ramírez, D. S.- Nabor Ojeda, S. S.- Rúbricas.",
                date(2020, 1, 4),
            ),
            (
                "La presente ley entrará en vigor al día siguiente de su publicación en el \"Diario Oficial\" de la Federación. México, D.F., a 29 de diciembre de 1974.- \"Año de la República Federal y del Senado\".- Píndaro Urióstegui Miranda, D.P.- Francisco Luna Kan, S.P.- Agustín Ruiz Soto, S.S.- Rúbricas.",
                date(2020, 1, 2),
            ),
            (
                "La presente Ley entrará en vigor desde la fecha de su promulgación en el \"Diario Oficial\" de la Federación. José Escudero Andrade, D. P.- José María Dávila, S. V. P.- Vicente L. Benéitez, S. S.-Rúbricas\".",
                date(2020, 1, 1),
            ),
            (
                "El presente Reglamento entrará en vigor a los treinta días siguientes de su publicación en el Diario Oficial de la Federación. 60",
                date(2020, 1, 31),
            ),
            (
                "Esta ley entrará en vigor a los quince días de la fecha de su publicación en el \"Diario Oficial\" de la Federación.",
                date(2020, 1, 16),
            ),
            (
                "El presente Decreto entrará en vigor 90 días naturales después de su publicación en el Diario Oficial de la Federación.",
                date(2020, 3, 31),
            ),
            (
                "Esta Ley entrará en vigor en toda la República, el día primero de enero de 1982.",
                date(1982, 1, 1),
            ),
            (
                "Esta ley entrará en vigor el día de la publicación del presente decreto en el Diario Oficial de la Federación.",
                date(2020, 1, 1),
            ),
            // the shared number table now reaches the larger counts
            (
                "La presente Ley entrará en vigor a los ciento ochenta días de su publicación en el Diario Oficial de la Federación.",
                date(2020, 6, 29),
            ),
        ] {
            let outcome = commencement_of(text, &none);
            assert_eq!(resolved_date(&outcome), Some(expected), "{text}");
            assert_eq!(outcome.determinations.len(), 1, "{text}");
        }
    }

    #[test]
    fn a_signature_tail_may_not_carry_other_text() {
        let none = BusinessDayCalendar::default();
        let outcome = commencement_of(
            "La presente Ley entrará en vigor al día siguiente de su publicación en el Diario Oficial de la Federación. Las solicitudes en trámite se resolverán conforme a la ley anterior, D. P.- Rúbricas.",
            &none,
        );
        assert!(outcome.determinations.is_empty());
    }

    #[test]
    fn date_repair_uses_the_same_calendar_as_the_derivation() {
        // Codex audit 2026-11-17, C6: with an evidenced calendar the derive
        // writes a counted date; the repair must see that same commencement,
        // so an article later shown to be amended has its date cleared.
        let inst = instrument(date(2023, 6, 1));
        let calendar = calendar_for(2023, &[]);
        let mut provisions = vec![
            article("1", "Texto original."),
            transitory("primero", FIVE_BUSINESS_DAYS),
        ];
        let outcome = derive_article_temporal_determinations_with_calendar(
            &inst,
            &provisions,
            date(2026, 11, 17),
            &calendar,
        );
        lex_core::apply_temporal_determinations(&mut provisions, &outcome.determinations);
        assert_eq!(provisions[0].effective_from, Some(date(2023, 6, 8)));
        provisions[0].text = "Texto reformado DOF 10-05-2024".to_owned();
        // without the calendar the commencement is invisible and nothing is repaired
        assert_eq!(
            repair_article_dates(&inst, &mut provisions, &BusinessDayCalendar::default()),
            0
        );
        assert_eq!(provisions[0].effective_from, Some(date(2023, 6, 8)));
        assert_eq!(repair_article_dates(&inst, &mut provisions, &calendar), 1);
        assert_eq!(provisions[0].effective_from, None);
    }
}
