use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use url::Url;

mod temporal;

pub use temporal::{
    ReappliedTemporalState, RoutedTemporalAnalysis, TemporalReviewOpenError,
    TemporalReviewResolutionError, TemporalRoutingError, apply_temporal_determinations,
    evidence_sha256, open_temporal_review, parse_temporal_model_response,
    preserve_temporal_review_history, reapply_temporal_determinations, resolve_temporal_review,
    route_temporal_analysis,
};

pub const SCHEMA_VERSION: &str = "0.1.0";
pub const LRITF_INSTRUMENT_ID: &str = "urn:lex-mx:federal:statute:lritf";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum InstrumentType {
    Constitution,
    Code,
    Statute,
    Regulation,
    Guideline,
    Circular,
    Other,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum StandardKind {
    Nom,
    Nmx,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum StandardStatus {
    Current,
    Cancelled,
    Replaced,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TechnicalReviewStatus {
    NotAnalyzed,
    ReviewRequired,
    TechnicalVerified,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum StandardTextBasis {
    AsPublished,
    OfficialCompilation,
    /// Built by the sequential canonical-state fold (Scope 2 Stage C,
    /// `docs/decisions.md` 2026-07-31): a strict left fold over the
    /// instrument's own decree history, not the publisher's own compiled
    /// text. No committed standard uses this yet -- Stage C is not wired
    /// into any parser or CLI command. Landed now, ahead of that wiring, so
    /// the schema and this enum do not have to change again once it is.
    DerivedConsolidation,
}

/// One per-unit decision within a decree's `apply()` step in the sequential
/// canonical-state fold (Scope 2 Stage C's candidate engine, `docs/decisions.md`
/// 2026-07-31): `canonical := base; for decree in chronological order:
/// canonical := apply(canonical, decree)`.
///
/// Deliberately three variants and no fourth "delete": the doctrine's own
/// rule is that nothing is ever removed from the structure or renumbered
/// around -- a repealed unit becomes [`Self::Replace`] with a `derogado`
/// marker recording the repealing decree, in place, matching how DOF's own
/// compiled texts render repeals.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case", tag = "operation")]
pub enum CanonicalFoldOperation {
    /// The decree gives explicit new text for this unit.
    Replace { text: String },
    /// Ellipsis: this unit's content and position are both unchanged by
    /// this decree.
    Keep,
    /// The decree's own resolving-clause prose renumbers this unit
    /// ("recorriéndose los demás en su orden") without changing its
    /// content. Collapsing this into [`Self::Keep`] is the specific way a
    /// naive fold implementation silently mislabels a provision.
    Shift { new_position: String },
}

/// Fold one unit's chronological decree history into its current text and
/// position. A pure function over already-classified per-decree operations
/// -- it does not read a decree's prose or classify ellipsis-diff itself
/// (that is the unwired, not-yet-scoped part of Stage C); this is the
/// annex-independent core the doctrine describes, isolated so it can be
/// exercised and trusted before anything reads a real decree body.
#[must_use]
pub fn fold_unit(
    initial_text: &str,
    initial_position: &str,
    steps: &[CanonicalFoldOperation],
) -> (String, String) {
    let mut text = initial_text.to_owned();
    let mut position = initial_position.to_owned();
    for step in steps {
        match step {
            CanonicalFoldOperation::Replace { text: new_text } => text.clone_from(new_text),
            CanonicalFoldOperation::Keep => {}
            CanonicalFoldOperation::Shift { new_position } => position.clone_from(new_position),
        }
    }
    (text, position)
}

/// What a modifying decree does to one addressed unit of a standard.
///
/// Deliberately three values and no more: this records the decree's own verb,
/// not a consolidated outcome. Applying any of them to the retained text is
/// Scope 2 Stage C, not this.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum StandardModificationAction {
    Modified,
    Added,
    Eliminated,
}

impl StandardModificationAction {
    /// The same `snake_case` token serde serializes, for prose messages, so
    /// the two can never drift apart.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Modified => "modified",
            Self::Added => "added",
            Self::Eliminated => "eliminated",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StandardModificationSource {
    pub publication_date: NaiveDate,
    pub official_url: Url,
    pub included_in_source: bool,
    /// The decree's own DOF publication title, verbatim.
    ///
    /// Pure source-grounded input, never derived. A Mexican modifying decree
    /// names the numerals it touches in its own title ("Modificación de los
    /// numerales 3.2, 3.10 ... y eliminación ... del Apéndice normativo A de
    /// la Norma Oficial Mexicana NOM-247-SSA1-2008, ..."), which is what turns
    /// an instrument-level "this standard has an unincorporated modification"
    /// warning into a clause-level one without reading the decree's body.
    ///
    /// Optional because the form is not universal: STPS publishes "ACUERDO de
    /// Modificación a la Norma Oficial Mexicana NOM-020-STPS-2011, ..." with
    /// no numerals in the title at all. A title recorded with no parseable
    /// targets is a different fact from no title recorded, and the validator
    /// distinguishes them.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    /// SHA-256 of this decree's own source document bytes (Stage B,
    /// `docs/decisions.md` 2026-07-31) -- the same hash semantics as
    /// [`StandardMetadata::source_sha256`] on the base publication, not the
    /// decree's extracted text. `modifications` is already the per-decree
    /// list Scope 2's "multi-source provenance" note called for; this pins
    /// each entry's own bytes rather than introducing a second list.
    ///
    /// Absent, not merely unset, whenever `included_in_source` is `true`:
    /// a decree already folded into the retained base text has no separate
    /// document to pin independently of `source_sha256` above.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_sha256: Option<String>,
}

/// One decree-to-clause amendment mark on a [`StandardClause`].
///
/// Derived, not authored: parsed from the modification's `title` and matched
/// against committed clause numbers. `modification_index` indexes
/// [`StandardMetadata::modifications`].
///
/// This records that a clause's text is **known outdated**, precisely located.
/// It is the opposite of a currency claim -- the clause text remains exactly
/// the base publication, and nothing here applies the modification.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct StandardClauseAmendment {
    pub modification_index: usize,
    pub action: StandardModificationAction,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StandardSystematicReview {
    pub review_date: NaiveDate,
    pub result: String,
    pub report_url: Option<Url>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct StandardSupplementStart {
    /// Exact source text at which this top-level supplement begins. The
    /// anchor may span lines when the visible heading alone is duplicated.
    pub anchor: String,
    pub kind: StandardSupplementKind,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum StandardSupplementKind {
    Appendix,
    Annex,
    ReferenceGuide,
    Table,
    List,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum StandardSupplementLegalCharacter {
    ExplicitlyNormative,
    ExplicitlyNonNormative,
    Unspecified,
}

/// Source-grounded identity and lifecycle facts for one NOM or NMX.
///
/// This remains separate from [`Instrument`]: standards have designation,
/// technical-review, replacement-chain, and conformity-assessment semantics
/// that statutes and regulations do not.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StandardMetadata {
    pub schema_version: String,
    pub id: String,
    pub kind: StandardKind,
    pub designation: String,
    /// The designation under which the retained text was actually published,
    /// when the registry has since redesignated the instrument.
    ///
    /// Mexican norm prefixes track the issuing authority, and the authority is
    /// sometimes reorganized. Where the registry redesignates, `designation`
    /// carries the current form and this field preserves the published one:
    /// NOM-002-SEMARNAT-1996's own text is titled NOM-002-**ECOL**-1996 and
    /// names SEMARNAP as issuer. Where the registry keeps the historical
    /// prefix, nothing is recorded here even though the authority was renamed
    /// -- SCFI persists in NOM-051-SCFI/SSA1-2010 despite the Secretaría de
    /// Comercio y Fomento Industrial becoming the Secretaría de Economía.
    ///
    /// Without this, a record would assert a designation appearing nowhere in
    /// its own source text, with nothing marking the discrepancy.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub published_designation: Option<String>,
    pub official_title: String,
    pub issuing_authorities: Vec<String>,
    pub regulatory_domains: Vec<String>,
    pub publication_date: NaiveDate,
    pub effective_date: Option<NaiveDate>,
    pub cancellation_date: Option<NaiveDate>,
    pub status: StandardStatus,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub replaces: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub replaced_by: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub joint_prefixes: Vec<String>,
    pub objective: Option<String>,
    pub scope: Option<String>,
    pub conformity_assessment: Option<String>,
    pub text_basis: StandardTextBasis,
    #[serde(default)]
    pub modifications: Vec<StandardModificationSource>,
    /// Exact, source-ordered starts of represented top-level material after a
    /// genuine TRANSITORIOS section. Internal rows and subsections stay opaque.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub supplement_starts: Vec<StandardSupplementStart>,
    pub systematic_review: Option<StandardSystematicReview>,
    pub source_url: Url,
    pub official_dof_url: Url,
    pub official_registry_url: Option<Url>,
    pub publisher: String,
    pub retrieved_at: DateTime<Utc>,
    pub source_sha256: String,
    pub extracted_text_sha256: String,
    pub parser_version: String,
    pub legal_review_status: ReviewStatus,
    pub technical_review_status: TechnicalReviewStatus,
}

/// One numbered clause in a compiled standard, anchored to the unchanged
/// extracted source text rather than represented as a legal article.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct StandardClause {
    pub schema_version: String,
    pub id: String,
    pub standard_id: String,
    pub number: String,
    pub label: String,
    pub text: String,
    pub start_char: usize,
    pub end_char: usize,
    /// Modifying decrees that name this clause in their own DOF title.
    ///
    /// Empty for a clause no recorded decree addresses -- which is not a claim
    /// that the clause is current, only that no *recorded* modification names
    /// it. Targets that resolve to no committed clause are not silently
    /// dropped; they are reported by `validate_standard`.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub amended_by: Vec<StandardClauseAmendment>,
}

/// One ordinal-labeled transitory provision from a standard's TRANSITORIOS
/// section, addressable but deliberately not deeply parsed: a standard's
/// transitorios can be long and structurally complex (phased criteria,
/// tables, cross-references), and this stays a lightweight inspection
/// rather than a full clause-style structural parse of that complexity.
/// `asserted_dates` is a raw scan of the retained text for "N de MES de
/// AAAA" phrases, in order of appearance -- not a claim about what each
/// date means (entry into force, phase boundary, deadline, ...); that
/// requires reading the surrounding text.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct StandardTransitory {
    pub schema_version: String,
    pub id: String,
    pub standard_id: String,
    pub ordinal: String,
    pub text: String,
    pub start_char: usize,
    pub end_char: usize,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub asserted_dates: Vec<NaiveDate>,
}

/// One top-level appendix, annex, reference guide, standalone table, or list
/// following a genuine TRANSITORIOS section. Its interior remains opaque.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct StandardSupplement {
    pub schema_version: String,
    pub id: String,
    pub standard_id: String,
    pub sequence: usize,
    pub kind: StandardSupplementKind,
    pub heading: String,
    pub legal_character: StandardSupplementLegalCharacter,
    pub text: String,
    pub start_char: usize,
    pub end_char: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StandardValidationReport {
    pub schema_version: String,
    pub standard_id: String,
    pub valid: bool,
    pub clause_count: usize,
    pub supplement_count: usize,
    pub issues: Vec<ValidationIssue>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum InstrumentStatus {
    InForce,
    PartiallyEffective,
    Repealed,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum ProvisionType {
    Article,
    Transitory,
    Annex,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TemporalStatus {
    Unknown,
    PublishedNotEffective,
    Effective,
    FutureEffective,
    // Legacy v1 values retained only so existing artifacts remain readable.
    // Temporal v2 rejects these as provision statuses and models them as effects.
    PartiallyEffective,
    ConditionallyEffective,
    Repealed,
    /// Part of the provision is repealed or judicially invalidated and the
    /// rest remains; `Provision::repeals` records what was removed.
    PartiallyRepealed,
    /// Commencement depends on a future act that has not been shown to occur;
    /// `Provision::commencement_condition` records the condition.
    ConditionalPending,
    RepealedWithSurvival,
    Superseded,
    TemporarilyApplicable,
    PendingConsolidation,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Basis {
    SourceText,
    ExpressCrossReference,
    ExpressDefinition,
    DeterministicRule,
    LlmInference,
    LawyerVerified,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ReviewStatus {
    NotAnalyzed,
    MachineAccepted,
    ReviewRequired,
    LawyerVerified,
}

/// A named batch of already-committed instruments assigned to one reviewer,
/// so ingestion is not gated on a legal review happening one instrument at a
/// time.
///
/// Deliberately distinct from [`ReviewStatus`] and [`StandardMetadata`]'s
/// `legal_review_status`/`technical_review_status`: those describe whether
/// an individual provision's or standard's *content* has been reviewed. A
/// `ReviewPacket` tracks workflow only -- who is responsible for looking at
/// this group of instruments, and how far they've gotten -- and carries no
/// verdict of its own. Reviewing a packet does not set any instrument's
/// review status; that link is deliberately not built yet (see
/// `docs/decisions.md`, packet-based review assignment).
///
/// `instruments` is frozen at generation time -- if a batch later gains a
/// newly-committed instrument, its existing packet does not grow to include
/// it, and nothing detects the drift. Reconciling membership after the fact
/// is not built; see `docs/decisions.md` 2026-07-31 for the known gap.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewPacket {
    pub schema_version: String,
    /// Stable identifier for this packet. Currently always the `batch_id`
    /// of the `batches/*.json` manifest it was generated from -- see
    /// `grouping_key`.
    pub packet_id: String,
    /// What `instruments` were grouped by. Always `"batch_id"` today: the
    /// per-topic groupings `batches/*.json` already uses for ingestion
    /// (`labor_L1_labor`, `financial_F2_banking`, ...) are legally coherent
    /// clusters a reviewer can look at together, and the field exists so a
    /// future non-batch grouping (standards, CNBV DCG instruments -- neither
    /// has a batch manifest today) does not have to overload this shape.
    pub grouping_key: String,
    /// Committed instrument slugs in this packet, restricted to what
    /// `corpus/mx` actually holds at generation time -- a batch manifest can
    /// list instruments not yet ingested, and those are not this reviewer's
    /// concern until they exist.
    pub instruments: Vec<String>,
    pub status: PacketReviewStatus,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reviewer: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub assigned_at: Option<DateTime<Utc>>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub notes: Vec<String>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PacketReviewStatus {
    Unassigned,
    Assigned,
    InReview,
    Reviewed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceManifest {
    pub schema_version: String,
    pub instrument_id: String,
    pub operational_source: String,
    pub formal_publication_source: String,
    pub publisher: String,
    pub official_url: Url,
    pub reference_url: Option<Url>,
    pub retrieved_at: DateTime<Utc>,
    pub http_status: u16,
    pub content_type: Option<String>,
    pub content_length: Option<u64>,
    pub etag: Option<String>,
    pub last_modified: Option<String>,
    pub source_sha256: String,
    pub extracted_text_sha256: Option<String>,
    pub extraction_tool: Option<String>,
    pub parser_version: String,
    pub schema_version_used: String,
    pub resulting_git_commit: Option<String>,
}

/// A source-stated inclusive publication period. It is not an exact day.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct PublicationPeriod {
    pub metadata_version: String,
    pub kind: PublicationPeriodKind,
    pub start: NaiveDate,
    pub end: NaiveDate,
    pub source_note: String,
    pub source_sha256: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PublicationPeriodKind {
    SerializedRange,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum NumberingBasis {
    ImplicitSingleton,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum EnactmentOwner {
    ContainingDecree,
    ContainingAct,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum EnactmentEvidenceKind {
    Transitory,
    ElidedSiblingArticle,
    /// Substantive containing-act article, never a provision of the nested law.
    ContainingArticle,
}

/// Source apparatus belonging to the enacting decree, not law provisions or
/// later reform evidence. No temporal or legal determination is stored here.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EnactmentEvidence {
    pub schema_version: String,
    pub id: String,
    pub instrument_id: String,
    pub owner: EnactmentOwner,
    pub kind: EnactmentEvidenceKind,
    pub containing_act_title: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub section_heading: Option<String>,
    pub source_heading: String,
    pub number: String,
    pub text: String,
    pub publication_date: Option<NaiveDate>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub publication_period: Option<PublicationPeriod>,
    pub source_sha256: String,
    pub extracted_text_sha256: String,
    pub text_sha256: String,
}

/// Source-backed instrument-level historical notice. Publication wording is
/// retained as evidence; no date or provision-level legal effect is inferred.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct SourceRepealNotice {
    pub source_note: String,
    pub source_sha256: String,
    pub formal_publication_url: Url,
    pub formal_source_sha256: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub successor_instrument_id: Option<String>,
    /// Alternative evidence basis: the retained source carries no abrogation
    /// header, so the notice binds to the abrogating provision of an in-corpus
    /// successor instrument instead.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub successor_clause: Option<SuccessorAbrogationClause>,
    pub reviewer: String,
    pub captured_at: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reviewer_supplied_at: Option<String>,
    pub rationale: String,
    pub decision_source: String,
    pub decision_source_sha256: String,
}

/// Exact abrogating provision of the successor instrument, bound by canonical text hash.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct SuccessorAbrogationClause {
    pub provision_id: String,
    pub provision_text_sha256: String,
}

fn normalized_whitespace(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

impl SourceRepealNotice {
    pub fn validate(&self) -> Result<(), String> {
        let schema: serde_json::Value = serde_json::from_str(include_str!(
            "../../../schemas/source-repeal-notice.schema.json"
        ))
        .map_err(|e| e.to_string())?;
        jsonschema::validator_for(&schema)
            .map_err(|e| e.to_string())?
            .validate(&serde_json::to_value(self).map_err(|e| e.to_string())?)
            .map_err(|e| e.to_string())?;
        let host = self.formal_publication_url.host_str().unwrap_or("");
        if self.formal_publication_url.scheme() != "https"
            || !(host == "dof.gob.mx"
                || host.ends_with(".dof.gob.mx")
                || host == "sidof.segob.gob.mx")
            || match &self.successor_clause {
                None => self
                    .source_note
                    .strip_prefix("Ley Abrogada DOF ")
                    .is_none_or(|date| {
                        chrono::NaiveDate::parse_from_str(date, "%d-%m-%Y").is_err()
                    }),
                Some(clause) => self
                    .successor_instrument_id
                    .as_ref()
                    .is_none_or(|id| !clause.provision_id.starts_with(&format!("{id}:"))),
            }
            || chrono::DateTime::parse_from_rfc3339(&self.captured_at).is_err()
            || self
                .reviewer_supplied_at
                .as_ref()
                .is_some_and(|t| chrono::DateTime::parse_from_rfc3339(t).is_err())
        {
            return Err("invalid source-backed repeal notice".to_owned());
        }
        Ok(())
    }

    #[must_use]
    pub fn matches_source(&self, source_hash: &str, raw: &str) -> bool {
        self.validate().is_ok()
            && self.successor_clause.is_none()
            && self.source_sha256 == source_hash
            && raw
                .split_whitespace()
                .collect::<Vec<_>>()
                .join(" ")
                .contains(
                    &self
                        .source_note
                        .split_whitespace()
                        .collect::<Vec<_>>()
                        .join(" "),
                )
    }

    /// Successor-clause basis: the successor corpus must hold the exact
    /// abrogating provision, unchanged, and the note must be quoted from it.
    #[must_use]
    pub fn matches_successor(&self, source_hash: &str, successor: &Corpus) -> bool {
        let Some(clause) = &self.successor_clause else {
            return false;
        };
        self.validate().is_ok()
            && self.source_sha256 == source_hash
            && self.successor_instrument_id.as_deref() == Some(&successor.instrument.id)
            && successor.provisions.iter().any(|p| {
                p.id == clause.provision_id
                    && p.instrument_id == successor.instrument.id
                    && evidence_sha256(&p.text) == clause.provision_text_sha256
                    && normalized_whitespace(&p.text)
                        .contains(&normalized_whitespace(&self.source_note))
            })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Instrument {
    pub schema_version: String,
    pub id: String,
    pub jurisdiction: String,
    pub level: String,
    pub instrument_type: InstrumentType,
    pub official_title: String,
    pub short_name: String,
    pub operational_source: String,
    pub formal_publication_source: String,
    pub publication_date: Option<NaiveDate>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub publication_period: Option<PublicationPeriod>,
    pub latest_reform_date: Option<NaiveDate>,
    pub retrieved_at: DateTime<Utc>,
    pub source_url: Url,
    pub source_sha256: String,
    pub extracted_text_sha256: String,
    pub parser_version: String,
    pub status: InstrumentStatus,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_repeal_notice: Option<SourceRepealNotice>,
    /// Authorities that jointly issued the instrument. Empty for instruments
    /// recorded before joint-issuer support (for example, statutes enacted by
    /// Congress and recorded only through their operational publisher).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub issuing_authorities: Vec<String>,
    /// Formal publication (Diario Oficial de la Federación) locator for the
    /// instrument itself, when the formal source was acquired directly.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub formal_publication_url: Option<Url>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub formal_publication_code: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub formal_source_sha256: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub formal_extracted_text_sha256: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct HeadingContext {
    /// The enclosing Libro, verbatim (`LIBRO SEGUNDO`), for códigos that
    /// use one; absent from every instrument committed before 2026-07-11,
    /// and omitted from serialization so those corpora stay byte-identical.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub libro: Option<String>,
    pub title: Option<String>,
    pub chapter: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub section: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub apartado: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Provision {
    pub schema_version: String,
    pub id: String,
    pub instrument_id: String,
    pub provision_type: ProvisionType,
    pub label: String,
    pub number: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub numbering_basis: Option<NumberingBasis>,
    pub heading_context: HeadingContext,
    pub text: String,
    pub publication_date: Option<NaiveDate>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub publication_period: Option<PublicationPeriod>,
    pub effective_from: Option<NaiveDate>,
    pub effective_to: Option<NaiveDate>,
    pub temporal_status: TemporalStatus,
    pub temporal_basis: Option<Basis>,
    pub temporal_confidence: Option<f32>,
    pub review_status: ReviewStatus,
    #[serde(default)]
    pub transitory_effects: Vec<TransitoryEffect>,
    /// Amendment markers a compiled CNBV document prints in the margin of
    /// this provision — the numeral system linking amended text to its
    /// resolución modificatoria. Each number resolves through the
    /// instrument's `amendment_references` legend. Sorted and deduplicated;
    /// empty for instruments without compiled-document markers.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub amendment_marks: Vec<u32>,
    /// What a partial repeal or judicial invalidation removed, as the source
    /// states it. Required for `PartiallyRepealed`; may also qualify `Repealed`.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub repeals: Vec<ProvisionRepeal>,
    /// The unmet or unverified condition a `ConditionalPending` provision
    /// waits on. Kept once met, as the evidence for the commencement date.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub commencement_condition: Option<CommencementCondition>,
}

/// The kind of unit a repeal removes.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RepealScope {
    Paragraph,
    Fraction,
    Inciso,
    Other,
}

/// Who removed the text: the legislature, or the Supreme Court by an
/// invalidation or general declaration of unconstitutionality.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RepealCause {
    Legislative,
    Judicial,
}

/// One partial repeal. `ordinals` are source statements about the text as it
/// stood before the repeal ("1", "2", "4" or "III"), never offsets into the
/// current text, because renumbering makes offsets unreliable.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ProvisionRepeal {
    pub scope: RepealScope,
    pub ordinals: Vec<String>,
    pub cause: RepealCause,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub dof_date: Option<NaiveDate>,
    /// The source says the remaining units are renumbered ("recorriéndose").
    #[serde(default)]
    pub renumbering: bool,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ConditionStatus {
    Unverified,
    Met,
    Unmet,
}

/// A commencement that waits on a future act. Only an audited resolution may
/// set `Met`; the parser and a model never do.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CommencementCondition {
    pub source_provision_id: String,
    /// The condition in the source's words.
    pub condition_text: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub authority: Option<String>,
    pub condition_status: ConditionStatus,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub met_on: Option<NaiveDate>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub evidence: Option<String>,
}

/// One entry of a compiled CNBV document's REFERENCIAS legend: the marker
/// number the margin notes use, and the verbatim legend text identifying
/// the amending resolución and action (Reformado / Adicionado / Derogado).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AmendmentReference {
    pub marker: u32,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ReferenceQualifierType {
    Paragraph,
    Fraction,
    Subsection,
    Other,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ReferenceQualifier {
    pub qualifier_type: ReferenceQualifierType,
    pub text: String,
    /// Unicode character span of the qualifier within the source
    /// provision's text, when the extractor anchored it. Enables
    /// presentation features such as linking `fracción XI` to the target's
    /// fraction block.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub start_char: Option<usize>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub end_char: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ReferenceResolutionStatus {
    Resolved,
    Unresolved,
    /// The source expressly cites a prior legal instrument that is no longer
    /// represented by the current consolidated target corpus. The edge is
    /// retained as a historical citation, not treated as a live graph link.
    HistoricalTargetUnavailable,
    /// An exact human-reviewed stale citation within a still-identified target.
    HistoricalSupersededReferenceOutdated,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ReferenceForm {
    Direct,
    RangeExpansion,
    /// A position-relative citation (`artículo anterior` / `artículo
    /// siguiente`) resolved against the source provision's neighbors in
    /// document order, kept distinguishable from explicit numeric
    /// citations because the target is inferred from position, not named.
    Relative,
}

/// How the linker chose a reference edge's target instrument and provision.
/// Recorded so a consumer can weigh an inferred target (a bare `la Ley`
/// resolved to a regulation's parent, a position-relative citation) against
/// one the text names. Absent on edges written before the field existed.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ResolutionRule {
    /// No instrument named; the citing instrument is assumed.
    OwnInstrumentDefault,
    /// An explicit self marker (`de esta Ley`, `del presente Reglamento`).
    OwnInstrumentMarker,
    /// A configured instrument name, alias or official title in the text.
    NamedInstrument,
    /// A bare `la Ley` / `el Código` resolved to the adapter's parent law.
    ParentShorthand,
    /// A fraction cited without an article number, inside its own article.
    SameArticle,
    /// `artículo anterior` / `artículo siguiente`, resolved by document order.
    RelativePosition,
}

/// Opt-in source-backed boundaries after deterministic block normalization.
/// Markers match block prefixes exactly and must occur once in the source.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NestedInstrumentProfile {
    pub containing_act_title: String,
    pub wrapper_marker: String,
    pub body_start_marker: String,
    pub body_end_marker: String,
    pub outer_transitory_marker: String,
}

/// Human authorization for one exact source citation, not a target-wide exemption.
/// `captured_at` records software capture; it is not the reviewer's decision time.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct OutdatedReferenceApproval {
    pub source_provision_id: String,
    pub target_instrument_id: String,
    pub target_provision_id: String,
    pub source_span: String,
    pub start_char: usize,
    pub end_char: usize,
    pub source_text_sha256: String,
    pub source_sha256: String,
    pub reviewer: String,
    pub captured_at: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reviewer_supplied_at: Option<String>,
    pub rationale: String,
    pub decision_source: String,
    pub decision_source_sha256: String,
}

impl OutdatedReferenceApproval {
    pub fn validate(&self) -> Result<(), String> {
        let hash = |s: &str| {
            s.len() == 64
                && s.bytes()
                    .all(|c| c.is_ascii_digit() || (b'a'..=b'f').contains(&c))
        };
        if self.source_provision_id.trim().is_empty()
            || self.target_instrument_id.trim().is_empty()
            || !self
                .target_provision_id
                .starts_with(&format!("{}:", self.target_instrument_id))
            || self.source_span.is_empty()
            || self.start_char >= self.end_char
            || self.source_span.chars().count() != self.end_char - self.start_char
            || !hash(&self.source_text_sha256)
            || !hash(&self.source_sha256)
            || self.reviewer.trim().is_empty()
            || self.rationale.trim().is_empty()
            || self.decision_source.trim().is_empty()
            || !hash(&self.decision_source_sha256)
            || chrono::DateTime::parse_from_rfc3339(&self.captured_at).is_err()
            || self
                .reviewer_supplied_at
                .as_ref()
                .is_some_and(|t| chrono::DateTime::parse_from_rfc3339(t).is_err())
        {
            return Err("invalid exact outdated-reference approval".to_owned());
        }
        Ok(())
    }

    #[must_use]
    pub fn matches(&self, edge: &ReferenceEdge, source_text: &str) -> bool {
        self.validate().is_ok()
            && self.source_provision_id == edge.source_provision_id
            && self.target_instrument_id == edge.target_instrument_id
            && self.target_provision_id == edge.target_provision_id
            && self.source_span == edge.source_span
            && self.start_char == edge.start_char
            && self.end_char == edge.end_char
            && self.source_text_sha256 == evidence_sha256(source_text)
    }
}

/// A human legal note on one exact source citation. It never changes the edge's
/// target or resolution status; it records what the reviewer found, for example
/// the article a stale citation should now point to. `captured_at` records
/// software capture, not the reviewer's decision time.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct LegalOperatorReview {
    pub source_provision_id: String,
    pub target_instrument_id: String,
    pub target_provision_id: String,
    pub source_span: String,
    pub start_char: usize,
    pub end_char: usize,
    pub source_text_sha256: String,
    pub reviewer: String,
    pub captured_at: String,
    pub note: String,
    pub decision_source: String,
    pub decision_source_sha256: String,
}

impl LegalOperatorReview {
    pub fn validate(&self) -> Result<(), String> {
        let hash = |s: &str| {
            s.len() == 64
                && s.bytes()
                    .all(|c| c.is_ascii_digit() || (b'a'..=b'f').contains(&c))
        };
        if self.source_provision_id.trim().is_empty()
            || self.target_instrument_id.trim().is_empty()
            || !self
                .target_provision_id
                .starts_with(&format!("{}:", self.target_instrument_id))
            || self.source_span.is_empty()
            || self.start_char >= self.end_char
            || self.source_span.chars().count() != self.end_char - self.start_char
            || !hash(&self.source_text_sha256)
            || self.reviewer.trim().is_empty()
            || self.note.trim().is_empty()
            || self.decision_source.trim().is_empty()
            || !hash(&self.decision_source_sha256)
            || chrono::DateTime::parse_from_rfc3339(&self.captured_at).is_err()
        {
            return Err("invalid exact legal operator review".to_owned());
        }
        Ok(())
    }

    #[must_use]
    pub fn matches(&self, edge: &ReferenceEdge, source_text: &str) -> bool {
        self.validate().is_ok()
            && self.source_provision_id == edge.source_provision_id
            && self.target_instrument_id == edge.target_instrument_id
            && self.target_provision_id == edge.target_provision_id
            && self.source_span == edge.source_span
            && self.start_char == edge.start_char
            && self.end_char == edge.end_char
            && self.source_text_sha256 == evidence_sha256(source_text)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReferenceEdge {
    pub schema_version: String,
    pub id: String,
    pub source_provision_id: String,
    pub source_span: String,
    pub start_char: usize,
    pub end_char: usize,
    pub target_instrument_id: String,
    pub target_provision_id: String,
    pub qualifiers: Vec<ReferenceQualifier>,
    pub basis: Basis,
    pub confidence: f32,
    pub resolution_status: ReferenceResolutionStatus,
    pub reference_form: ReferenceForm,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub outdated_reference_approval: Option<OutdatedReferenceApproval>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub legal_operator_review: Option<LegalOperatorReview>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub resolution_rule: Option<ResolutionRule>,
}

/// A term expressly defined by a glossary provision, anchored to the exact
/// span of its definition entry within that provision's canonical text.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DefinedTerm {
    pub schema_version: String,
    /// `urn:…:<instrument>:term:<slug>`.
    pub id: String,
    /// The term exactly as defined, for example `Comité Interinstitucional`.
    pub term: String,
    pub instrument_id: String,
    pub defining_provision_id: String,
    /// Printed label of the defining fraction, for fraction-style
    /// glossaries such as LRITF Article 4: a Roman numeral, optionally
    /// qualified (`I Bis`, `VII Ter`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fraction: Option<String>,
    /// Unicode character span of the complete definition entry (including
    /// its continuation paragraphs) within the defining provision's text.
    pub start_char: usize,
    pub end_char: usize,
    pub basis: Basis,
}

/// One exact occurrence of a defined term inside a provision's canonical
/// text, resolved to its defining term (possibly in another instrument).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TermUsage {
    pub schema_version: String,
    pub id: String,
    /// Provision whose text contains the occurrence.
    pub provision_id: String,
    /// The resolved [`DefinedTerm`] identifier.
    pub term_id: String,
    /// The exact matched text, which may be a singular/plural variant of
    /// the defined term.
    pub span: String,
    pub start_char: usize,
    pub end_char: usize,
}

/// Version of the glossary extraction strategies a [`GlossaryCoverage`]
/// describes. Bump it when detection or completeness rules change meaning.
pub const GLOSSARY_STRATEGY_VERSION: &str = "glossary-extraction-1";

/// How `terms.json` was produced for one instrument
/// (`glossary-coverage.json`). It separates a glossary held as incomplete
/// from an instrument with no detected glossary, which an empty terms list
/// cannot. It is a deterministic technical record, never a legal approval.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GlossaryCoverage {
    pub schema_version: String,
    pub strategy_version: String,
    pub instrument_id: String,
    pub source_sha256: String,
    pub extracted_text_sha256: String,
    pub status: GlossaryCoverageStatus,
    /// The glossary provisions whose terms were extracted, in order.
    pub glossaries: Vec<GlossaryProvisionCoverage>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hold: Option<GlossaryHoldRecord>,
    /// The `additive_to` entries whose terms usages also resolve against.
    pub imports: Vec<GlossaryImportCoverage>,
    pub term_count: usize,
    pub usage_count: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GlossaryCoverageStatus {
    /// The adapter names the glossary provisions and their styles.
    Configured,
    /// Auto-detection found a complete glossary.
    Detected,
    /// Auto-detection found a glossary it could not extract completely, so
    /// the instrument has no terms.
    Held,
    /// Auto-detection found no glossary.
    NoneDetected,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GlossaryProvisionCoverage {
    pub provision_id: String,
    pub provision_text_sha256: String,
    /// `established_pass` or `wider_pass` for a detected glossary, otherwise
    /// the configured style.
    pub method: String,
    /// The printed fraction label of each extracted term, in order.
    pub fractions: Vec<String>,
    /// Repealed fractions skipped as whole placeholders. Only the wider-pass
    /// grammar (`wider_pass`, `auto_whole`) reports them; absent otherwise.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub repeal_placeholders: Option<Vec<String>>,
    pub term_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GlossaryHoldRecord {
    pub provision_id: String,
    pub provision_text_sha256: String,
    pub code: GlossaryHoldCode,
    pub message: String,
}

/// Why a detected glossary was held as incomplete.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GlossaryHoldCode {
    SequenceBreak,
    UnknownQualifier,
    NoncanonicalNumeral,
    NoTermDelimiter,
    SwallowedFraction,
    RepealWording,
    MissingTerm,
    UnplainTerm,
    RepeatedTerm,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GlossaryImportCoverage {
    pub additive_to: String,
    pub term_count: usize,
}

impl GlossaryCoverage {
    /// Check the record against its schema and against the instrument's
    /// persisted record, provisions, terms and usages: each named provision's
    /// text must still hash as recorded, and the counts and fractions must be
    /// those of `terms.json`. A mismatch means the record is stale.
    pub fn validate(
        &self,
        instrument: &Instrument,
        provisions: &[Provision],
        terms: &[DefinedTerm],
        usages: &[TermUsage],
    ) -> Result<(), String> {
        let schema: serde_json::Value = serde_json::from_str(include_str!(
            "../../../schemas/glossary-coverage.schema.json"
        ))
        .map_err(|e| e.to_string())?;
        jsonschema::validator_for(&schema)
            .map_err(|e| e.to_string())?
            .validate(&serde_json::to_value(self).map_err(|e| e.to_string())?)
            .map_err(|e| e.to_string())?;
        if self.instrument_id != instrument.id
            || self.source_sha256 != instrument.source_sha256
            || self.extracted_text_sha256 != instrument.extracted_text_sha256
        {
            return Err("coverage does not belong to this instrument record".to_owned());
        }
        if self.term_count != terms.len() || self.usage_count != usages.len() {
            return Err(format!(
                "coverage counts {} terms and {} usages; terms.json has {} and term-usages.json {}",
                self.term_count,
                self.usage_count,
                terms.len(),
                usages.len()
            ));
        }
        let text_hash = |id: &str| {
            provisions
                .iter()
                .find(|provision| provision.id == id)
                .map(|provision| evidence_sha256(&provision.text))
        };
        for glossary in &self.glossaries {
            if text_hash(&glossary.provision_id).as_deref()
                != Some(glossary.provision_text_sha256.as_str())
            {
                return Err(format!(
                    "glossary provision {} is missing or its text changed",
                    glossary.provision_id
                ));
            }
            let own: Vec<&DefinedTerm> = terms
                .iter()
                .filter(|term| term.defining_provision_id == glossary.provision_id)
                .collect();
            let fractions: Vec<&str> = own
                .iter()
                .filter_map(|term| term.fraction.as_deref())
                .collect();
            if own.len() != glossary.term_count || fractions != glossary.fractions {
                return Err(format!(
                    "glossary provision {} does not match its terms in terms.json",
                    glossary.provision_id
                ));
            }
        }
        if self
            .glossaries
            .iter()
            .map(|glossary| glossary.term_count)
            .sum::<usize>()
            != terms.len()
        {
            return Err("terms.json has terms outside the recorded glossaries".to_owned());
        }
        if let Some(hold) = &self.hold
            && text_hash(&hold.provision_id).as_deref() != Some(hold.provision_text_sha256.as_str())
        {
            return Err(format!(
                "held glossary provision {} is missing or its text changed",
                hold.provision_id
            ));
        }
        if !self.status_is_consistent(!terms.is_empty()) {
            return Err(format!(
                "coverage status {:?} contradicts its record",
                self.status
            ));
        }
        Ok(())
    }

    /// Whether the status agrees with the rest of the record: a held or
    /// absent glossary has no provisions or imports, and only a detected one
    /// names an auto-detection pass.
    fn status_is_consistent(&self, has_terms: bool) -> bool {
        let detected_method = |glossary: &GlossaryProvisionCoverage| {
            matches!(glossary.method.as_str(), "established_pass" | "wider_pass")
        };
        match self.status {
            GlossaryCoverageStatus::Configured => {
                !self.glossaries.is_empty()
                    && self.hold.is_none()
                    && !self.glossaries.iter().any(detected_method)
            }
            GlossaryCoverageStatus::Detected => {
                self.glossaries.len() == 1
                    && detected_method(&self.glossaries[0])
                    && self.hold.is_none()
                    && self.imports.is_empty()
                    && has_terms
            }
            GlossaryCoverageStatus::Held => {
                self.glossaries.is_empty() && self.hold.is_some() && self.imports.is_empty()
            }
            GlossaryCoverageStatus::NoneDetected => {
                self.glossaries.is_empty() && self.hold.is_none() && self.imports.is_empty()
            }
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Corpus {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub enactment_evidence: Vec<EnactmentEvidence>,
    pub instrument: Instrument,
    pub provisions: Vec<Provision>,
    #[serde(default)]
    pub references: Vec<ReferenceEdge>,
    #[serde(default)]
    pub terms: Vec<DefinedTerm>,
    #[serde(default)]
    pub term_usages: Vec<TermUsage>,
    /// The compiled document's REFERENCIAS legend, mapping each margin
    /// marker number to its amending resolución. Empty for instruments
    /// without compiled-document markers.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub amendment_references: Vec<AmendmentReference>,
}

#[must_use]
pub fn enactment_evidence_id(
    instrument_id: &str,
    date: Option<NaiveDate>,
    kind: &EnactmentEvidenceKind,
    number: &str,
) -> String {
    let date = date.map_or_else(|| "serialized-range".to_owned(), |d| d.to_string());
    let key = number
        .to_lowercase()
        .replace('á', "a")
        .replace('é', "e")
        .replace('í', "i")
        .replace('ó', "o")
        .replace('ú', "u")
        .replace(' ', "-");
    let unit = match kind {
        EnactmentEvidenceKind::Transitory => "transitory",
        EnactmentEvidenceKind::ElidedSiblingArticle => "elided",
        EnactmentEvidenceKind::ContainingArticle => "containing-article",
    };
    format!("{instrument_id}:enactment:{date}:decree:{unit}:{key}")
}

/// Validate newly introduced publication/evidence boundaries without changing
/// legacy exact-day records. New ranges never provide an arithmetic anchor.
/// Stable neutral identity for a containing act whose formal legal kind is retained in its title.
#[must_use]
pub fn containing_act_evidence_id(
    instrument_id: &str,
    date: Option<NaiveDate>,
    kind: &EnactmentEvidenceKind,
    number: &str,
) -> String {
    enactment_evidence_id(instrument_id, date, kind, number).replace(":decree:", ":containing-act:")
}

fn validate_review_metadata(corpus: &Corpus) -> Result<(), String> {
    if let Some(notice) = &corpus.instrument.source_repeal_notice {
        notice.validate()?;
        if corpus.instrument.status != InstrumentStatus::Repealed
            || notice.source_sha256 != corpus.instrument.source_sha256
            || notice.successor_instrument_id.as_deref() == Some(&corpus.instrument.id)
        {
            return Err("source-backed repeal notice/status identity mismatch".to_owned());
        }
    }
    for edge in &corpus.references {
        if let Some(approval) = &edge.outdated_reference_approval {
            approval.validate()?;
            if approval.source_sha256 != corpus.instrument.source_sha256 {
                return Err(format!(
                    "outdated-reference approval source hash mismatch: {}",
                    edge.id
                ));
            }
        }
    }
    Ok(())
}

pub fn validate_publication_metadata(corpus: &Corpus) -> Result<(), String> {
    validate_review_metadata(corpus)?;
    let instrument = &corpus.instrument;
    match (instrument.publication_date, &instrument.publication_period) {
        (Some(_), None) => {}
        (None, Some(period)) => {
            let schema: serde_json::Value = serde_json::from_str(include_str!(
                "../../../schemas/publication-period.schema.json"
            ))
            .map_err(|e| e.to_string())?;
            let validator = jsonschema::validator_for(&schema).map_err(|e| e.to_string())?;
            validator
                .validate(&serde_json::to_value(period).map_err(|e| e.to_string())?)
                .map_err(|e| e.to_string())?;
            if period.start > period.end || period.source_sha256 != instrument.source_sha256 {
                return Err("invalid or source-mismatched publication period".to_owned());
            }
        }
        _ => return Err("publication must specify exactly one of exact day or period".to_owned()),
    }
    for provision in &corpus.provisions {
        if provision.publication_date != instrument.publication_date
            || provision.publication_period != instrument.publication_period
        {
            return Err(format!(
                "publication metadata differs from owning instrument: {}",
                provision.id
            ));
        }
        if provision.numbering_basis.is_some()
            && (provision.provision_type != ProvisionType::Transitory
                || provision.number != "UNICO"
                || !provision.label.to_uppercase().contains("TRANSITORIO"))
        {
            return Err(format!(
                "invalid implicit singleton numbering: {}",
                provision.id
            ));
        }
    }
    if corpus
        .provisions
        .iter()
        .any(|p| p.numbering_basis.is_some())
        && corpus
            .provisions
            .iter()
            .filter(|p| p.provision_type == ProvisionType::Transitory)
            .count()
            != 1
    {
        return Err("implicit singleton cannot coexist with another own transitory".to_owned());
    }
    let mut ids: std::collections::HashSet<_> =
        corpus.provisions.iter().map(|p| p.id.as_str()).collect();
    if !corpus.enactment_evidence.is_empty() {
        let schema: serde_json::Value = serde_json::from_str(include_str!(
            "../../../schemas/enactment-evidence.schema.json"
        ))
        .map_err(|e| e.to_string())?;
        let validator = jsonschema::validator_for(&schema).map_err(|e| e.to_string())?;
        for item in &corpus.enactment_evidence {
            validator
                .validate(&serde_json::to_value(item).map_err(|e| e.to_string())?)
                .map_err(|e| e.to_string())?;
            if !ids.insert(&item.id)
                || item.instrument_id != instrument.id
                || item.id
                    != if item.owner == EnactmentOwner::ContainingAct {
                        containing_act_evidence_id(
                            &instrument.id,
                            instrument.publication_date,
                            &item.kind,
                            &item.number,
                        )
                    } else {
                        enactment_evidence_id(
                            &instrument.id,
                            instrument.publication_date,
                            &item.kind,
                            &item.number,
                        )
                    }
                || item.source_sha256 != instrument.source_sha256
                || item.extracted_text_sha256 != instrument.extracted_text_sha256
                || item.text_sha256 != evidence_sha256(&item.text)
                || item.publication_date != instrument.publication_date
                || item.publication_period != instrument.publication_period
            {
                return Err(format!(
                    "invalid or source-mismatched enactment evidence: {}",
                    item.id
                ));
            }
        }
    }
    Ok(())
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemporalAnalysisRequest {
    pub schema_version: String,
    pub prompt_version: String,
    pub instrument_id: String,
    pub publication_date: NaiveDate,
    pub latest_reform_date: Option<NaiveDate>,
    pub relevant_provisions: Vec<TemporalEvidence>,
    pub required_output_schema: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemporalEvidence {
    pub provision_id: String,
    pub label: String,
    pub text: String,
    /// Amendment-marker numbers carried by a reform transitory that was
    /// itself later amended (CNBV consolidated disposiciones re-amend
    /// their own reform transitorios). Each number references the source
    /// document's REFERENCIAS legend; the mention is kept without linking
    /// to the modifying resolution, which is not a corpus instrument.
    /// Empty for the committed IFPE/ITF corpora, so their serialized
    /// evidence is unchanged.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub amendment_marks: Vec<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemporalDetermination {
    pub provision_id: String,
    pub temporal_status: TemporalStatus,
    pub publication_date: NaiveDate,
    pub effective_from: Option<NaiveDate>,
    pub effective_to: Option<NaiveDate>,
    pub confidence: f32,
    pub basis: Basis,
    pub supporting_text: Vec<String>,
    pub review_required: bool,
    pub review_reason: Option<String>,
    pub model: String,
    pub prompt_version: String,
    #[serde(default)]
    pub effects: Vec<TransitoryEffect>,
    /// SHA-256 of the exact evidence text this determination was made
    /// against. A reparse re-applies a determination only when the current
    /// evidence hashes identically; a quotation merely remaining a
    /// substring of materially different text is not sufficient. Empty for
    /// determinations recorded before this field existed; these remain stale
    /// until evidence is verified through a new analysis and review.
    #[serde(default)]
    pub evidence_sha256: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TemporalModelBatch {
    pub determinations: Vec<TemporalModelDetermination>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TemporalModelDetermination {
    pub provision_id: String,
    pub temporal_status: TemporalStatus,
    pub effective_from: Option<NaiveDate>,
    pub effective_to: Option<NaiveDate>,
    pub confidence: f32,
    pub supporting_text: Vec<String>,
    #[serde(default)]
    pub effects: Vec<TransitoryEffect>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TransitoryEffectType {
    Commencement,
    ImplementationDeadline,
    RegulatoryDeadline,
    AdaptationPeriod,
    TransitionalPermission,
    ProceduralSurvival,
    Migration,
    AuthorityAssignment,
    CoordinationMandate,
    StagedCommencement,
    Sunset,
    Repeal,
    Other,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TransitoryApplicationRule {
    NotApplicable,
    NewRuleProspectively,
    PriorRuleForExistingMatters,
    TransitionalPermission,
    MigrationToNewRule,
    Mixed,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TemporalBoundaryType {
    None,
    Publication,
    EffectiveDate,
    FixedDate,
    RelativePeriod,
    ExternalEvent,
    CohortExhaustion,
    AuthorityAction,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TemporalVerificationStatus {
    NotRequired,
    ConfirmedBySourceText,
    ExternalVerificationRequired,
    ExternallyVerified,
    OpenEndedByDesign,
    UnknownMaterial,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct TemporalBoundary {
    pub boundary_type: TemporalBoundaryType,
    pub date: Option<NaiveDate>,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct TransitoryEffect {
    pub effect_type: TransitoryEffectType,
    pub affected_scope: String,
    pub application_rule: TransitoryApplicationRule,
    pub trigger: TemporalBoundary,
    pub end_condition: TemporalBoundary,
    pub responsible_authorities: Vec<String>,
    pub verification_status: TemporalVerificationStatus,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub verification_source_url: Option<Url>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub verified_event_date: Option<NaiveDate>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub verification_note: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemporalAnalysisResult {
    pub schema_version: String,
    pub instrument_id: String,
    pub request_sha256: String,
    pub response_sha256: String,
    pub response_id: Option<String>,
    pub model: String,
    pub prompt_version: String,
    pub analyzed_at: DateTime<Utc>,
    pub determinations: Vec<TemporalDetermination>,
}

#[derive(Debug, Clone)]
pub struct TemporalAnalysisMetadata {
    pub request_sha256: String,
    pub response_sha256: String,
    pub response_id: Option<String>,
    pub model: String,
    pub analyzed_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewItem {
    pub id: String,
    pub instrument_id: String,
    pub provision_id: String,
    pub exact_issue: String,
    pub proposed_machine_conclusion: TemporalDetermination,
    pub evidence: TemporalEvidence,
    pub camara_source_url: Url,
    pub formal_source_url: Option<Url>,
    pub provision_diff: Option<String>,
    pub resolution_options: Vec<ReviewResolution>,
    pub status: ReviewItemStatus,
    pub reviewer_note: Option<String>,
    pub resolution: Option<ReviewResolution>,
    pub resolved_by: Option<String>,
    pub resolved_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone)]
pub struct TemporalReviewResolution {
    pub resolution: ReviewResolution,
    pub reviewer: String,
    pub note: Option<String>,
    pub temporal_status: Option<TemporalStatus>,
    pub effective_from: Option<NaiveDate>,
    pub effective_to: Option<NaiveDate>,
    pub effects: Option<Vec<TransitoryEffect>>,
    pub resolved_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ReviewItemStatus {
    Pending,
    Resolved,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ReviewResolution {
    AcceptMachineConclusion,
    SetUnknown,
    LawyerOverride,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationIssue {
    pub severity: Severity,
    pub code: String,
    pub message: String,
    pub provision_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Severity {
    Error,
    Warning,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationReport {
    pub schema_version: String,
    pub instrument_id: String,
    pub valid: bool,
    pub article_count: usize,
    pub transitory_count: usize,
    #[serde(default)]
    pub reference_count: usize,
    pub issues: Vec<ValidationIssue>,
}

#[cfg(test)]
mod fold_tests {
    use super::{CanonicalFoldOperation, fold_unit};

    #[test]
    fn keep_carries_the_base_text_and_position_forward_unchanged() {
        let (text, position) = fold_unit(
            "Texto original.",
            "tercer párrafo",
            &[CanonicalFoldOperation::Keep],
        );
        assert_eq!(text, "Texto original.");
        assert_eq!(position, "tercer párrafo");
    }

    #[test]
    fn replace_changes_text_but_not_position() {
        let (text, position) = fold_unit(
            "Texto original.",
            "tercer párrafo",
            &[CanonicalFoldOperation::Replace {
                text: "Texto reformado.".to_owned(),
            }],
        );
        assert_eq!(text, "Texto reformado.");
        assert_eq!(position, "tercer párrafo");
    }

    #[test]
    fn shift_moves_position_without_touching_content_and_is_not_keep() {
        // The doctrine's own warning: collapsing `Shift` into `Keep` is the
        // specific way a naive fold silently mislabels a provision whose
        // content the ellipsis left alone but whose ordinal position the
        // decree's own "recorriéndose los demás en su orden" prose moved.
        let (text, position) = fold_unit(
            "Texto original.",
            "tercer párrafo",
            &[CanonicalFoldOperation::Shift {
                new_position: "cuarto párrafo".to_owned(),
            }],
        );
        assert_eq!(text, "Texto original.", "shift must not touch content");
        assert_ne!(
            CanonicalFoldOperation::Shift {
                new_position: "cuarto párrafo".to_owned()
            },
            CanonicalFoldOperation::Keep,
            "shift and keep must remain distinct operations"
        );
        assert_eq!(position, "cuarto párrafo");
    }

    #[test]
    fn a_chronological_sequence_folds_left_to_right_over_the_previous_result() {
        // canonical := base; for decree in chronological order:
        // canonical := apply(canonical, decree) -- each step sees the prior
        // step's output, not the original base every time. This is exactly
        // the bug the doctrine was proposed to correct: a draft that took
        // each decree's restatement in isolation against the base, rather
        // than against the running canonical state.
        let steps = [
            CanonicalFoldOperation::Replace {
                text: "Primera reforma.".to_owned(),
            },
            CanonicalFoldOperation::Keep,
            CanonicalFoldOperation::Shift {
                new_position: "cuarto párrafo".to_owned(),
            },
            CanonicalFoldOperation::Replace {
                text: "(Derogado, D.O.F. 2026-07-31)".to_owned(),
            },
        ];
        let (text, position) = fold_unit("Texto original.", "tercer párrafo", &steps);
        assert_eq!(text, "(Derogado, D.O.F. 2026-07-31)");
        assert_eq!(position, "cuarto párrafo");
    }
}

#[cfg(test)]
mod publication_metadata_tests {
    use super::*;

    fn exact_corpus() -> Corpus {
        Corpus {
            instrument: serde_json::from_str(include_str!(
                "../../../corpus/mx/lritf/instrument.json"
            ))
            .unwrap(),
            provisions: serde_json::from_str(include_str!(
                "../../../corpus/mx/lritf/provisions.json"
            ))
            .unwrap(),
            enactment_evidence: Vec::new(),
            references: Vec::new(),
            terms: Vec::new(),
            term_usages: Vec::new(),
            amendment_references: Vec::new(),
        }
    }

    fn range_corpus() -> Corpus {
        let mut corpus = exact_corpus();
        let period = PublicationPeriod {
            metadata_version: "1.0.0".to_owned(),
            kind: PublicationPeriodKind::SerializedRange,
            start: NaiveDate::from_ymd_opt(1912, 1, 1).unwrap(),
            end: NaiveDate::from_ymd_opt(1912, 1, 8).unwrap(),
            source_note: "Source states serialized publication".to_owned(),
            source_sha256: corpus.instrument.source_sha256.clone(),
        };
        corpus.instrument.publication_date = None;
        corpus.instrument.publication_period = Some(period.clone());
        for p in &mut corpus.provisions {
            p.publication_date = None;
            p.publication_period = Some(period.clone());
        }
        corpus
    }

    #[test]
    fn existing_exact_day_fields_roundtrip_without_shape_or_value_change() {
        let corpus = exact_corpus();
        let old: serde_json::Value =
            serde_json::from_str(include_str!("../../../corpus/mx/lritf/instrument.json")).unwrap();
        assert_eq!(serde_json::to_value(&corpus.instrument).unwrap(), old);
        let old: serde_json::Value =
            serde_json::from_str(include_str!("../../../corpus/mx/lritf/provisions.json")).unwrap();
        let actual: serde_json::Value =
            serde_json::from_str(&serde_json::to_string(&corpus.provisions).unwrap()).unwrap();
        assert_eq!(
            old.as_array().unwrap().len(),
            actual.as_array().unwrap().len()
        );
        for (before, after) in old
            .as_array()
            .unwrap()
            .iter()
            .zip(actual.as_array().unwrap())
        {
            assert_eq!(
                before.as_object().unwrap().keys().collect::<Vec<_>>(),
                after.as_object().unwrap().keys().collect::<Vec<_>>(),
                "field shape for {}",
                before["id"]
            );
            for (key, value) in before.as_object().unwrap() {
                assert!(
                    value == &after[key],
                    "changed field {key} for {}",
                    before["id"]
                );
            }
            assert!(after.get("publication_period").is_none());
            assert!(after.get("numbering_basis").is_none());
        }
        validate_publication_metadata(&corpus).unwrap();
    }

    #[test]
    fn period_records_have_no_exact_day_and_reject_conflicting_or_reversed_metadata() {
        let mut corpus = range_corpus();
        validate_publication_metadata(&corpus).unwrap();
        let value = serde_json::to_value(&corpus.instrument).unwrap();
        assert!(value["publication_date"].is_null());
        assert_eq!(value["publication_period"]["metadata_version"], "1.0.0");
        corpus.instrument.publication_date = Some(NaiveDate::from_ymd_opt(1912, 1, 1).unwrap());
        assert!(validate_publication_metadata(&corpus).is_err());
        corpus = range_corpus();
        corpus.instrument.publication_period.as_mut().unwrap().start =
            NaiveDate::from_ymd_opt(1912, 1, 9).unwrap();
        assert!(validate_publication_metadata(&corpus).is_err());
        corpus = range_corpus();
        corpus.provisions[0].publication_date = Some(NaiveDate::from_ymd_opt(1912, 1, 8).unwrap());
        assert!(validate_publication_metadata(&corpus).is_err());
    }

    #[test]
    fn persisted_implicit_singleton_rejects_multiple_or_mixed_own_clauses() {
        let mut corpus = exact_corpus();
        let mut clause = corpus
            .provisions
            .iter()
            .find(|p| p.provision_type == ProvisionType::Transitory)
            .unwrap()
            .clone();
        clause.number = "UNICO".to_owned();
        clause.label = "ARTICULO TRANSITORIO:".to_owned();
        clause.numbering_basis = Some(NumberingBasis::ImplicitSingleton);
        corpus.provisions = vec![clause.clone()];
        validate_publication_metadata(&corpus).unwrap();
        clause.id.push_str(":second");
        clause.number = "PRIMERO".to_owned();
        clause.numbering_basis = None;
        corpus.provisions.push(clause);
        assert!(validate_publication_metadata(&corpus).is_err());
        corpus.provisions[1] = corpus.provisions[0].clone();
        corpus.provisions[1].id.push_str(":second");
        assert!(validate_publication_metadata(&corpus).is_err());
    }

    #[test]
    fn enactment_sidecar_identity_owner_source_and_text_are_checked() {
        let mut corpus = exact_corpus();
        let date = corpus.instrument.publication_date;
        let kind = EnactmentEvidenceKind::Transitory;
        let item = EnactmentEvidence {
            schema_version: SCHEMA_VERSION.to_owned(),
            id: enactment_evidence_id(&corpus.instrument.id, date, &kind, "PRIMERO"),
            instrument_id: corpus.instrument.id.clone(),
            owner: EnactmentOwner::ContainingDecree,
            kind,
            containing_act_title: "DECRETO de expedición".to_owned(),
            section_heading: Some("TRANSITORIOS".to_owned()),
            source_heading: "PRIMERO.-".to_owned(),
            number: "PRIMERO".to_owned(),
            text: "Texto fuente.".to_owned(),
            publication_date: date,
            publication_period: None,
            source_sha256: corpus.instrument.source_sha256.clone(),
            extracted_text_sha256: corpus.instrument.extracted_text_sha256.clone(),
            text_sha256: evidence_sha256("Texto fuente."),
        };
        corpus.enactment_evidence.push(item.clone());
        validate_publication_metadata(&corpus).unwrap();
        corpus.enactment_evidence[0].text.push_str(" Altered");
        assert!(validate_publication_metadata(&corpus).is_err());
        corpus.enactment_evidence[0] = item.clone();
        corpus.enactment_evidence[0].id.push('x');
        assert!(validate_publication_metadata(&corpus).is_err());
        corpus.enactment_evidence[0] = item.clone();
        corpus.enactment_evidence[0].source_sha256 = "0".repeat(64);
        assert!(validate_publication_metadata(&corpus).is_err());
        let mut value = serde_json::to_value(item).unwrap();
        value["owner"] = "law".into();
        assert!(serde_json::from_value::<EnactmentEvidence>(value).is_err());
    }
    #[test]
    fn source_repeal_notice_binds_header_and_acquisition_without_inventing_dates() {
        let raw = include_str!("../../../fixtures/diputados/held-lftr-repeal-header.txt");
        let notice = SourceRepealNotice {
            source_note: "Ley Abrogada DOF 16-07-2025".to_owned(),
            source_sha256: "a".repeat(64),
            formal_publication_url: "https://www.dof.gob.mx/nota_detalle.php?codigo=fixture"
                .parse()
                .unwrap(),
            formal_source_sha256: "b".repeat(64),
            successor_instrument_id: Some("urn:lex-mx:federal:statute:lmtr".to_owned()),
            successor_clause: None,
            reviewer: "Fixture reviewer".to_owned(),
            captured_at: "2026-10-16T00:00:00Z".to_owned(),
            reviewer_supplied_at: None,
            rationale: "Fixture only; no effect date asserted".to_owned(),
            decision_source: "fixture decision".to_owned(),
            decision_source_sha256: "c".repeat(64),
        };
        assert!(notice.matches_source(&"a".repeat(64), raw));
        assert!(!notice.matches_source(&"b".repeat(64), raw));
        assert!(!notice.matches_source(&"a".repeat(64), "Ley vigente"));
        let mut invalid = notice.clone();
        invalid.formal_publication_url = "https://example.com/unofficial".parse().unwrap();
        assert!(invalid.validate().is_err());
        let mut invalid = notice.clone();
        invalid.reviewer_supplied_at = Some("unknown".to_owned());
        assert!(invalid.validate().is_err());
        let serialized = serde_json::to_value(notice).unwrap();
        assert!(
            serialized.get("effective_from").is_none() && serialized.get("effective_to").is_none()
        );
        assert!(serialized.get("reviewer_supplied_at").is_none());
        // Header-basis notices must not serialize the successor-clause field.
        assert!(serialized.get("successor_clause").is_none());
    }

    fn successor_corpus_and_notice() -> (Corpus, SourceRepealNotice) {
        const SUCCESSOR: &str = "urn:lex-mx:federal:regulation:succ";
        let clause_text = "Con la entrada en vigor del presente Reglamento, se abroga al \
            Reglamento anterior, publicado el 14 de enero de 1999, y sus reformas.";
        let instrument: Instrument = serde_json::from_value(serde_json::json!({
            "schema_version": SCHEMA_VERSION,
            "id": SUCCESSOR,
            "jurisdiction": "mx",
            "level": "federal",
            "instrument_type": "regulation",
            "official_title": "Reglamento sucesor de prueba",
            "short_name": "SUCC",
            "operational_source": "camara_de_diputados",
            "formal_publication_source": "dof",
            "publication_date": "2026-09-04",
            "latest_reform_date": null,
            "retrieved_at": "2026-10-16T00:00:00Z",
            "source_url": "https://www.diputados.gob.mx/LeyesBiblio/regley/Fixture.pdf",
            "source_sha256": "e".repeat(64),
            "extracted_text_sha256": "f".repeat(64),
            "parser_version": "0.1.0",
            "status": "in_force"
        }))
        .unwrap();
        let provision: Provision = serde_json::from_value(serde_json::json!({
            "schema_version": SCHEMA_VERSION,
            "id": format!("{SUCCESSOR}:transitory:segundo"),
            "instrument_id": SUCCESSOR,
            "provision_type": "transitory",
            "label": "Segundo",
            "number": "SEGUNDO",
            "heading_context": {},
            "text": clause_text,
            "publication_date": "2026-09-04",
            "effective_from": null,
            "effective_to": null,
            "temporal_status": "unknown",
            "temporal_basis": null,
            "temporal_confidence": null,
            "review_status": "not_analyzed"
        }))
        .unwrap();
        let corpus = Corpus {
            instrument,
            provisions: vec![provision.clone()],
            references: vec![],
            terms: vec![],
            term_usages: vec![],
            amendment_references: vec![],
            enactment_evidence: vec![],
        };
        let notice = SourceRepealNotice {
            source_note: "se abroga al Reglamento anterior, publicado el 14 de enero de 1999"
                .to_owned(),
            source_sha256: "a".repeat(64),
            formal_publication_url: "https://dof.gob.mx/nota_detalle.php?codigo=fixture"
                .parse()
                .unwrap(),
            formal_source_sha256: "b".repeat(64),
            successor_instrument_id: Some(SUCCESSOR.to_owned()),
            successor_clause: Some(SuccessorAbrogationClause {
                provision_id: provision.id,
                provision_text_sha256: evidence_sha256(clause_text),
            }),
            reviewer: "Fixture reviewer".to_owned(),
            captured_at: "2026-10-16T00:00:00Z".to_owned(),
            reviewer_supplied_at: None,
            rationale: "Fixture only; no effect date asserted".to_owned(),
            decision_source: "fixture decision".to_owned(),
            decision_source_sha256: "c".repeat(64),
        };
        (corpus, notice)
    }

    #[test]
    fn successor_clause_notice_binds_exact_abrogating_provision() {
        let (corpus, notice) = successor_corpus_and_notice();
        assert!(notice.validate().is_ok());
        assert!(notice.matches_successor(&"a".repeat(64), &corpus));
        // The header basis never accepts a successor-clause notice, and vice versa.
        assert!(!notice.matches_source(&"a".repeat(64), &notice.source_note));
        let (_, header_only) = successor_corpus_and_notice();
        let mut header_only = header_only;
        header_only.successor_clause = None;
        assert!(!header_only.matches_successor(&"a".repeat(64), &corpus));
        // Fails closed on acquisition drift.
        assert!(!notice.matches_successor(&"d".repeat(64), &corpus));
        // Fails closed on changed provision text (hash drift).
        let mut drifted = corpus.clone();
        drifted.provisions[0].text.push_str(" Modificado.");
        assert!(!notice.matches_successor(&"a".repeat(64), &drifted));
        // Fails closed when the note is not quoted from the provision.
        let mut unquoted = notice.clone();
        unquoted.source_note = "se abroga otro reglamento".to_owned();
        assert!(!unquoted.matches_successor(&"a".repeat(64), &corpus));
        // Fails closed when the provision is missing or the successor is another instrument.
        let mut missing = corpus.clone();
        missing.provisions.clear();
        assert!(!notice.matches_successor(&"a".repeat(64), &missing));
        let mut other = corpus.clone();
        other.instrument.id = "urn:lex-mx:federal:regulation:other".to_owned();
        assert!(!notice.matches_successor(&"a".repeat(64), &other));
        // Validation rejects a clause not owned by the declared successor or with no successor.
        let mut foreign = notice.clone();
        foreign.successor_clause.as_mut().unwrap().provision_id =
            "urn:lex-mx:federal:regulation:other:transitory:segundo".to_owned();
        assert!(foreign.validate().is_err());
        let mut no_successor = notice.clone();
        no_successor.successor_instrument_id = None;
        assert!(no_successor.validate().is_err());
        let mut bad_hash = notice;
        bad_hash
            .successor_clause
            .as_mut()
            .unwrap()
            .provision_text_sha256 = "zz".to_owned();
        assert!(bad_hash.validate().is_err());
    }
}

#[cfg(test)]
mod glossary_coverage_tests {
    use super::*;

    struct Lritf {
        instrument: Instrument,
        provisions: Vec<Provision>,
        terms: Vec<DefinedTerm>,
        usages: Vec<TermUsage>,
    }

    fn lritf() -> Lritf {
        Lritf {
            instrument: serde_json::from_str(include_str!(
                "../../../corpus/mx/lritf/instrument.json"
            ))
            .unwrap(),
            provisions: serde_json::from_str(include_str!(
                "../../../corpus/mx/lritf/provisions.json"
            ))
            .unwrap(),
            terms: serde_json::from_str(include_str!("../../../corpus/mx/lritf/terms.json"))
                .unwrap(),
            usages: serde_json::from_str(include_str!("../../../corpus/mx/lritf/term-usages.json"))
                .unwrap(),
        }
    }

    fn configured(record: &Lritf) -> GlossaryCoverage {
        let provision_id = record.terms[0].defining_provision_id.clone();
        let text = &record
            .provisions
            .iter()
            .find(|provision| provision.id == provision_id)
            .unwrap()
            .text;
        GlossaryCoverage {
            schema_version: SCHEMA_VERSION.to_owned(),
            strategy_version: GLOSSARY_STRATEGY_VERSION.to_owned(),
            instrument_id: record.instrument.id.clone(),
            source_sha256: record.instrument.source_sha256.clone(),
            extracted_text_sha256: record.instrument.extracted_text_sha256.clone(),
            status: GlossaryCoverageStatus::Configured,
            glossaries: vec![GlossaryProvisionCoverage {
                provision_id,
                provision_text_sha256: evidence_sha256(text),
                method: "fractions".to_owned(),
                fractions: record
                    .terms
                    .iter()
                    .filter_map(|term| term.fraction.clone())
                    .collect(),
                repeal_placeholders: None,
                term_count: record.terms.len(),
            }],
            hold: None,
            imports: Vec::new(),
            term_count: record.terms.len(),
            usage_count: record.usages.len(),
        }
    }

    fn check(record: &Lritf, coverage: &GlossaryCoverage) -> Result<(), String> {
        coverage.validate(
            &record.instrument,
            &record.provisions,
            &record.terms,
            &record.usages,
        )
    }

    #[test]
    fn a_coverage_record_must_describe_the_persisted_terms_and_text() {
        let record = lritf();
        let coverage = configured(&record);
        check(&record, &coverage).unwrap();

        let mut edited = lritf();
        let glossary_id = &coverage.glossaries[0].provision_id;
        edited
            .provisions
            .iter_mut()
            .find(|provision| &provision.id == glossary_id)
            .unwrap()
            .text
            .push(' ');
        assert!(
            check(&edited, &coverage)
                .unwrap_err()
                .contains("text changed")
        );

        let mut recounted = coverage.clone();
        recounted.usage_count += 1;
        assert!(
            check(&record, &recounted)
                .unwrap_err()
                .contains("coverage counts")
        );

        let mut renumbered = coverage.clone();
        renumbered.glossaries[0].fractions.reverse();
        assert!(
            check(&record, &renumbered)
                .unwrap_err()
                .contains("does not match")
        );

        let mut other_source = coverage.clone();
        other_source.source_sha256 = "0".repeat(64);
        assert!(check(&record, &other_source).is_err());
    }

    #[test]
    fn a_held_or_empty_status_cannot_carry_terms_or_glossaries() {
        let record = lritf();
        let mut held = configured(&record);
        held.status = GlossaryCoverageStatus::Held;
        assert!(check(&record, &held).unwrap_err().contains("contradicts"));

        let mut detected = configured(&record);
        detected.status = GlossaryCoverageStatus::Detected;
        assert!(
            check(&record, &detected)
                .unwrap_err()
                .contains("contradicts")
        );

        let empty = Lritf {
            terms: Vec::new(),
            usages: Vec::new(),
            ..lritf()
        };
        let mut none = configured(&record);
        none.status = GlossaryCoverageStatus::NoneDetected;
        none.glossaries.clear();
        none.term_count = 0;
        none.usage_count = 0;
        check(&empty, &none).unwrap();

        let mut held_empty = none.clone();
        held_empty.status = GlossaryCoverageStatus::Held;
        assert!(
            check(&empty, &held_empty)
                .unwrap_err()
                .contains("contradicts")
        );
        let provision = &record.provisions[3];
        held_empty.hold = Some(GlossaryHoldRecord {
            provision_id: provision.id.clone(),
            provision_text_sha256: evidence_sha256(&provision.text),
            code: GlossaryHoldCode::SequenceBreak,
            message: "fraction V breaks the I, II, III… sequence after 3".to_owned(),
        });
        check(&empty, &held_empty).unwrap();
    }

    #[test]
    fn coverage_deserialization_rejects_fields_the_schema_forbids() {
        let record = lritf();
        let original = serde_json::to_value(configured(&record)).unwrap();
        let mut top = original.clone();
        top["unrecognized"] = serde_json::json!(true);
        assert!(serde_json::from_value::<GlossaryCoverage>(top).is_err());
        let mut nested = original;
        nested["glossaries"][0]["unrecognized"] = serde_json::json!(true);
        assert!(serde_json::from_value::<GlossaryCoverage>(nested).is_err());
        assert!(
            serde_json::from_value::<GlossaryImportCoverage>(serde_json::json!({
                "additive_to": "urn:lex-mx:test:original", "term_count": 1,
                "unrecognized": true
            }))
            .is_err()
        );
    }

    #[test]
    fn the_schema_rejects_unknown_methods_and_fraction_labels() {
        let record = lritf();
        let mut method = configured(&record);
        method.glossaries[0].method = "guessed".to_owned();
        assert!(check(&record, &method).is_err());

        let mut label = configured(&record);
        label.glossaries[0].fractions[0] = "I Quáter".to_owned();
        assert!(check(&record, &label).is_err());
    }
}
