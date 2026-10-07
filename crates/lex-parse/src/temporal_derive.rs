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
    /// The instrument has no ordinary transitory provisions at all.
    NoOrdinaryTransitories,
    /// None of the ordinary transitories mention entering into force.
    NoCommencementClause,
    /// More than one ordinary transitory mentions entering into force —
    /// commencement is staged or split by subject matter.
    MultipleCommencementClauses { count: usize },
    /// The matched transitory carries an exception, condition, or
    /// staged-rollout qualifier (`con excepción`, `salvo`, `cuando`,
    /// `gradualmente`, `declaratoria`, `a más tardar`, …) — some or all of
    /// the instrument's own text does not share the plain reading.
    ExceptionOrConditionalLanguage { marker: String },
    /// The commencement period is counted in business days
    /// (`días hábiles`), which requires a holiday calendar this module does
    /// not have.
    BusinessDaysPeriod,
    /// The commencement period uses a unit this module does not compute
    /// (`meses`, `años`) — day arithmetic only is supported today.
    UnsupportedRelativePeriodUnit { unit: String },
    /// A day count is written in Spanish words this module's bounded number
    /// table does not cover, rather than digits.
    UnrecognizedNumberWord { word: String },
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
            Self::NoOrdinaryTransitories => "no-ordinary-transitories",
            Self::NoCommencementClause => "no-commencement-clause",
            Self::MultipleCommencementClauses { .. } => "multiple-commencement-clauses",
            Self::ExceptionOrConditionalLanguage { .. } => "exception-or-conditional-language",
            Self::BusinessDaysPeriod => "business-days-period",
            Self::UnsupportedRelativePeriodUnit { .. } => "unsupported-relative-period-unit",
            Self::UnrecognizedNumberWord { .. } => "unrecognized-number-word",
            Self::UnrecognizedCommencementPattern => "unrecognized-commencement-pattern",
        }
    }

    #[must_use]
    pub fn describe(&self) -> String {
        match self {
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
    let commencement = resolve_commencement(&ordinary_transitories, instrument.publication_date);
    if let Commencement::Resolved { effective_from, .. } = &commencement {
        let status = if *effective_from > today {
            TemporalStatus::FutureEffective
        } else {
            TemporalStatus::Effective
        };
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
            let article_date = (!has_amendment_evidence(provision)).then_some(*effective_from);
            determinations.push(determination(
                provision,
                instrument,
                status.clone(),
                article_date,
                0.9,
                vec![format!(
                    "instrument's own commencement clause resolves to {effective_from}"
                )],
            ));
        }
    }

    DerivationOutcome {
        determinations,
        commencement: Some(commencement),
    }
}

/// Whether an article's own text or footnote legend shows it was amended
/// after the instrument's enactment.
fn has_amendment_evidence(provision: &Provision) -> bool {
    !provision.amendment_marks.is_empty() || crate::mentions_dof_date(&provision.text)
}

/// Repair article start dates written by earlier versions of the rule: an
/// article with no amendment evidence takes the resolved original
/// commencement, and an amended one is left unset. Only values that are unset
/// or exactly the original commencement are touched, and only on
/// machine-accepted deterministic articles, so a model or human decision is
/// never altered. Returns how many dates changed.
#[must_use]
pub fn repair_article_dates(instrument: &Instrument, provisions: &mut [Provision]) -> usize {
    let transitories = provisions
        .iter()
        .filter(|provision| provision.provision_type == ProvisionType::Transitory)
        .collect::<Vec<_>>();
    let Commencement::Resolved { effective_from, .. } =
        resolve_commencement(&transitories, instrument.publication_date)
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
/// deterministic articles whose note [`crate::initial_repeals`] recognises are
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
            let repeals = crate::initial_repeals(&provision.text);
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
        publication_date: instrument.publication_date,
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

const EXCEPTION_MARKERS: [&str; 11] = [
    "con excepción",
    "a excepción",
    "excepción hecha",
    "salvo",
    "excepto",
    "sin perjuicio",
    "a más tardar",
    "gradual",
    "declaratoria",
    "en tanto",
    "hasta que",
];

fn resolve_commencement(transitories: &[&Provision], publication_date: NaiveDate) -> Commencement {
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
        1 => resolve_commencement_text(&matches[0].text, publication_date),
        count => Commencement::Skipped {
            reason: SkipReason::MultipleCommencementClauses { count },
            matched_text: None,
        },
    }
}

fn resolve_commencement_text(text: &str, publication_date: NaiveDate) -> Commencement {
    let lower = text.to_lowercase();
    if let Some(marker) = EXCEPTION_MARKERS
        .iter()
        .find(|marker| lower.contains(*marker))
    {
        return Commencement::Skipped {
            reason: SkipReason::ExceptionOrConditionalLanguage {
                marker: (*marker).to_owned(),
            },
            matched_text: Some(text.to_owned()),
        };
    }
    if lower.contains("hábil") {
        return Commencement::Skipped {
            reason: SkipReason::BusinessDaysPeriod,
            matched_text: Some(text.to_owned()),
        };
    }

    try_day_after_publication(text, publication_date)
        .or_else(|| try_same_day_as_publication(text, publication_date))
        .or_else(|| try_literal_date(text))
        .or_else(|| try_relative_days(text, publication_date))
        .unwrap_or(Commencement::Skipped {
            reason: SkipReason::UnrecognizedCommencementPattern,
            matched_text: Some(text.to_owned()),
        })
}

fn try_day_after_publication(text: &str, publication_date: NaiveDate) -> Option<Commencement> {
    let re = Regex::new(
        r"(?i)entrar[áa]n?\s+en\s+vigor\s+(?:a\s+partir\s+del?\s+)?(?:(?:el\s+|al\s+)?d[ií]a\s+siguiente\s+(?:al?\s+)?(?:de\s+)?(?:su|la)\s+publicaci[óo]n|(?:el\s+|al\s+)?siguiente\s+d[ií]a\s+(?:de\s+)?(?:su|la)\s+publicaci[óo]n)",
    )
    .expect("static regex");
    let found = re.find(text)?;
    let effective_from = publication_date.checked_add_days(Days::new(1))?;
    Some(Commencement::Resolved {
        effective_from,
        pattern: "day_after_publication",
        matched_text: found.as_str().to_owned(),
    })
}

fn try_same_day_as_publication(text: &str, publication_date: NaiveDate) -> Option<Commencement> {
    let re = Regex::new(
        r"(?i)entrar[áa]n?\s+en\s+vigor\s+(?:desde\s+|en\s+)?(?:el\s+mismo\s+d[ií]a|el\s+d[ií]a|la\s+fecha)\s+de\s+(?:su|la)\s+(?:publicaci[óo]n|promulgaci[óo]n)",
    )
    .expect("static regex");
    let found = re.find(text)?;
    Some(Commencement::Resolved {
        effective_from: publication_date,
        pattern: "same_day_as_publication",
        matched_text: found.as_str().to_owned(),
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

fn try_literal_date(text: &str) -> Option<Commencement> {
    let re = Regex::new(
        r"(?i)entrar[áa]n?\s+en\s+vigor\s+(?:en\s+toda\s+la\s+Rep[uú]blica[,]?\s+)?(?:el\s+(?:d[ií]a\s+)?|a\s+partir\s+del?\s+)(\d{1,2}|primero|1[oº]\.?)\s+de\s+([a-záéíóúñ]+)\s+de(?:l)?\s+(\d{4})",
    )
    .expect("static regex");
    let captures = re.captures(text)?;
    let day_word = captures[1].to_lowercase();
    let day = if matches!(day_word.as_str(), "primero" | "1o" | "1º" | "1o." | "1°") {
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
    })
}

/// Common day counts written as Spanish words in commencement clauses.
/// Deliberately bounded: an unrecognized word is reported, not guessed at.
const NUMBER_WORDS: [(&str, u64); 20] = [
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
];

fn try_relative_days(text: &str, publication_date: NaiveDate) -> Option<Commencement> {
    let re = Regex::new(
        r"(?i)entrar[áa]n?\s+en\s+vigor\s+(?:a\s+los\s+)?(\d{1,3}|[a-záéíóúñ]+(?:\s+y\s+[a-záéíóúñ]+)?)\s+(d[ií]as?|mes(?:es)?|a[ñn]os?)\b(?:\s+[a-záéíóúñ]+){0,3}?\s+(?:al?\s+de\s+|de\s+|a\s+)(?:su|la)\s+publicaci[óo]n",
    )
    .expect("static regex");
    let captures = re.captures(text)?;
    let unit = captures[2].to_lowercase();
    if !unit.starts_with("dia") && !unit.starts_with("día") {
        return Some(Commencement::Skipped {
            reason: SkipReason::UnsupportedRelativePeriodUnit {
                unit: captures[2].to_owned(),
            },
            matched_text: Some(captures[0].to_owned()),
        });
    }
    let count_word = captures[1].to_lowercase();
    let count = if let Ok(digits) = count_word.parse::<u64>() {
        digits
    } else if let Some((_, value)) = NUMBER_WORDS
        .iter()
        .find(|(word, _)| *word == count_word.as_str())
    {
        *value
    } else {
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
            publication_date,
            latest_reform_date: None,
            retrieved_at: chrono::Utc::now(),
            source_url: "https://example.test/lm.pdf".parse().expect("valid url"),
            source_sha256: "0".repeat(64),
            extracted_text_sha256: "0".repeat(64),
            parser_version: "0.1.0".to_owned(),
            status: lex_core::InstrumentStatus::InForce,
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
            publication_date: date(1990, 1, 1),
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

        assert_eq!(super::repair_article_dates(&inst, &mut provisions), 2);
        assert_eq!(provisions[0].effective_from, Some(date(1995, 12, 23)));
        assert_eq!(provisions[1].effective_from, None);
        // Human review and an unrelated date are untouched.
        assert_eq!(provisions[3].effective_from, human.effective_from);
        assert_eq!(provisions[3].review_status, ReviewStatus::LawyerVerified);
        assert_eq!(provisions[4].effective_from, Some(date(2001, 1, 1)));
        assert_eq!(super::repair_article_dates(&inst, &mut provisions), 0);
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

    #[test]
    fn business_days_period_is_skipped() {
        let inst = instrument(date(2005, 1, 1));
        let provisions = vec![
            article("1", "Texto."),
            transitory(
                "unico",
                "El presente Decreto entrará en vigor a los sesenta días hábiles siguientes a su publicación en el Diario Oficial de la Federación.",
            ),
        ];
        let outcome = derive_article_temporal_determinations(&inst, &provisions, date(2026, 1, 1));
        assert!(outcome.determinations.is_empty());
        assert!(matches!(
            outcome.commencement,
            Some(Commencement::Skipped {
                reason: SkipReason::BusinessDaysPeriod,
                ..
            })
        ));
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
    fn singular_month_unit_is_still_reported_as_unsupported_unit() {
        // lfdc's real Primero: "un mes después", singular "mes" not plural
        // "meses" -- must still land in the specific unsupported-unit
        // category, not the generic unrecognized-pattern catch-all.
        let inst = instrument(date(2005, 1, 1));
        let provisions = vec![
            article("1", "Texto."),
            transitory(
                "primero",
                "La presente Ley entrará en vigor un mes después de su publicación en el Diario Oficial de la Federación.",
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
}
