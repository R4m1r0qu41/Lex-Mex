use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use url::Url;

mod temporal;

pub use temporal::{
    RoutedTemporalAnalysis, TemporalReviewOpenError, TemporalReviewResolutionError,
    TemporalRoutingError, apply_temporal_determinations, open_temporal_review,
    preserve_temporal_review_history, resolve_temporal_review, route_temporal_analysis,
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
pub enum InstrumentStatus {
    InForce,
    PartiallyEffective,
    Repealed,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
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
    pub publication_date: NaiveDate,
    pub latest_reform_date: Option<NaiveDate>,
    pub retrieved_at: DateTime<Utc>,
    pub source_url: Url,
    pub source_sha256: String,
    pub extracted_text_sha256: String,
    pub parser_version: String,
    pub status: InstrumentStatus,
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
    pub heading_context: HeadingContext,
    pub text: String,
    pub publication_date: NaiveDate,
    pub effective_from: Option<NaiveDate>,
    pub effective_to: Option<NaiveDate>,
    pub temporal_status: TemporalStatus,
    pub temporal_basis: Option<Basis>,
    pub temporal_confidence: Option<f32>,
    pub review_status: ReviewStatus,
    #[serde(default)]
    pub transitory_effects: Vec<TransitoryEffect>,
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
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ReferenceResolutionStatus {
    Resolved,
    Unresolved,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ReferenceForm {
    Direct,
    RangeExpansion,
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
    /// Roman numeral of the defining fraction, for fraction-style
    /// glossaries such as LRITF Article 4.
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Corpus {
    pub instrument: Instrument,
    pub provisions: Vec<Provision>,
    #[serde(default)]
    pub references: Vec<ReferenceEdge>,
    #[serde(default)]
    pub terms: Vec<DefinedTerm>,
    #[serde(default)]
    pub term_usages: Vec<TermUsage>,
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
