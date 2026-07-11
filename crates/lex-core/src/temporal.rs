use std::collections::{HashMap, HashSet};

use chrono::NaiveDate;
use sha2::{Digest, Sha256};
use thiserror::Error;
use url::Url;

use crate::{
    Basis, ReviewItem, ReviewItemStatus, ReviewResolution, ReviewStatus, SCHEMA_VERSION,
    TemporalAnalysisMetadata, TemporalAnalysisRequest, TemporalAnalysisResult,
    TemporalBoundaryType, TemporalDetermination, TemporalEvidence, TemporalModelBatch,
    TemporalModelDetermination, TemporalReviewResolution, TemporalStatus,
    TemporalVerificationStatus, TransitoryApplicationRule,
};

const AUTO_ACCEPT_MIN_CONFIDENCE: f32 = 0.92;

#[must_use]
pub fn evidence_sha256(text: &str) -> String {
    format!("{:x}", Sha256::digest(text.as_bytes()))
}

#[derive(Debug)]
pub struct RoutedTemporalAnalysis {
    pub result: TemporalAnalysisResult,
    pub review_items: Vec<ReviewItem>,
}

#[derive(Debug, Error)]
pub enum TemporalRoutingError {
    #[error("duplicate model determination for {0}")]
    DuplicateDetermination(String),
    #[error("model omitted determination for {0}")]
    MissingDetermination(String),
    #[error("model returned unknown evidence identifier {0}")]
    UnknownEvidence(String),
    #[error("confidence for {0} is outside 0..=1")]
    InvalidConfidence(String),
    #[error("supporting text for {0} is empty or is not an exact source substring")]
    UnsupportedCitation(String),
    #[error("effective_to precedes effective_from for {0}")]
    InvalidDateRange(String),
    #[error("model used an effect classification as the provision status for {0}")]
    InvalidProvisionStatus(String),
    #[error("model returned no transitory effects for {0}")]
    MissingEffects(String),
    #[error("model returned a malformed transitory effect for {0}")]
    InvalidEffect(String),
}

#[derive(Debug, Error)]
pub enum TemporalReviewOpenError {
    #[error("a review item already exists for {0}; audit history is never overwritten")]
    AlreadyExists(String),
    #[error("temporal result does not contain a determination for {0}")]
    DeterminationNotFound(String),
    #[error("temporal request does not contain evidence for {0}")]
    EvidenceNotFound(String),
    #[error("review reason cannot be empty")]
    EmptyReason,
}

#[derive(Debug, Error)]
pub enum TemporalReviewResolutionError {
    #[error("review item is already resolved")]
    AlreadyResolved,
    #[error("reviewer identity cannot be empty")]
    EmptyReviewer,
    #[error("lawyer_override requires a non-empty --note and at least one changed field")]
    IncompleteLawyerOverride,
    #[error("override fields are only valid with lawyer_override")]
    UnexpectedOverrideFields,
    #[error("effective_to precedes effective_from")]
    InvalidDateRange,
    #[error("effective lawyer override requires --effective-from")]
    MissingEffectiveFrom,
    #[error("future_effective lawyer override requires a future --effective-from")]
    InvalidFutureEffectiveDate,
    #[error("lawyer_override contains a malformed transitory effect")]
    InvalidOverrideEffect,
    #[error("temporal result does not contain determination for {0}")]
    DeterminationNotFound(String),
}

pub fn route_temporal_analysis(
    request: &TemporalAnalysisRequest,
    batch: TemporalModelBatch,
    metadata: TemporalAnalysisMetadata,
    camara_source_url: &Url,
) -> Result<RoutedTemporalAnalysis, TemporalRoutingError> {
    let evidence_by_id: HashMap<&str, &TemporalEvidence> = request
        .relevant_provisions
        .iter()
        .map(|evidence| (evidence.provision_id.as_str(), evidence))
        .collect();
    let mut model_by_id = HashMap::new();
    for determination in batch.determinations {
        let id = determination.provision_id.clone();
        if !evidence_by_id.contains_key(id.as_str()) {
            return Err(TemporalRoutingError::UnknownEvidence(id));
        }
        if model_by_id.insert(id.clone(), determination).is_some() {
            return Err(TemporalRoutingError::DuplicateDetermination(id));
        }
    }

    let mut determinations = Vec::with_capacity(request.relevant_provisions.len());
    let mut review_items = Vec::new();
    for evidence in &request.relevant_provisions {
        let model_output = model_by_id.remove(&evidence.provision_id).ok_or_else(|| {
            TemporalRoutingError::MissingDetermination(evidence.provision_id.clone())
        })?;
        validate_model_output(evidence, &model_output)?;
        let review_reasons = review_reasons(&model_output, metadata.analyzed_at.date_naive());
        let review_required = !review_reasons.is_empty();
        let determination = TemporalDetermination {
            provision_id: evidence.provision_id.clone(),
            temporal_status: model_output.temporal_status,
            publication_date: publication_date_for(evidence, request.publication_date),
            effective_from: model_output.effective_from,
            effective_to: model_output.effective_to,
            confidence: model_output.confidence,
            basis: Basis::LlmInference,
            supporting_text: model_output.supporting_text,
            review_required,
            review_reason: review_required.then(|| review_reasons.join("; ")),
            model: metadata.model.clone(),
            prompt_version: request.prompt_version.clone(),
            effects: model_output.effects,
            evidence_sha256: evidence_sha256(&evidence.text),
        };
        if review_required {
            review_items.push(review_item(
                request,
                evidence,
                &determination,
                camara_source_url.clone(),
            ));
        }
        determinations.push(determination);
    }

    Ok(RoutedTemporalAnalysis {
        result: TemporalAnalysisResult {
            schema_version: SCHEMA_VERSION.to_owned(),
            instrument_id: request.instrument_id.clone(),
            request_sha256: metadata.request_sha256,
            response_sha256: metadata.response_sha256,
            response_id: metadata.response_id,
            model: metadata.model,
            prompt_version: request.prompt_version.clone(),
            analyzed_at: metadata.analyzed_at,
            determinations,
        },
        review_items,
    })
}

fn validate_model_output(
    evidence: &TemporalEvidence,
    output: &TemporalModelDetermination,
) -> Result<(), TemporalRoutingError> {
    if !(0.0..=1.0).contains(&output.confidence) {
        return Err(TemporalRoutingError::InvalidConfidence(
            evidence.provision_id.clone(),
        ));
    }
    if output.supporting_text.is_empty()
        || output
            .supporting_text
            .iter()
            .any(|quote| quote.trim().is_empty() || !evidence.text.contains(quote))
    {
        return Err(TemporalRoutingError::UnsupportedCitation(
            evidence.provision_id.clone(),
        ));
    }
    if let (Some(from), Some(to)) = (output.effective_from, output.effective_to)
        && to < from
    {
        return Err(TemporalRoutingError::InvalidDateRange(
            evidence.provision_id.clone(),
        ));
    }
    if matches!(
        output.temporal_status,
        TemporalStatus::PartiallyEffective
            | TemporalStatus::ConditionallyEffective
            | TemporalStatus::RepealedWithSurvival
            | TemporalStatus::TemporarilyApplicable
            | TemporalStatus::PendingConsolidation
    ) {
        return Err(TemporalRoutingError::InvalidProvisionStatus(
            evidence.provision_id.clone(),
        ));
    }
    if output.effects.is_empty() {
        return Err(TemporalRoutingError::MissingEffects(
            evidence.provision_id.clone(),
        ));
    }
    for effect in &output.effects {
        if !valid_effect(effect) {
            return Err(TemporalRoutingError::InvalidEffect(
                evidence.provision_id.clone(),
            ));
        }
    }
    Ok(())
}

fn review_reasons(output: &TemporalModelDetermination, today: NaiveDate) -> Vec<String> {
    let mut reasons = Vec::new();
    if output.confidence < AUTO_ACCEPT_MIN_CONFIDENCE {
        reasons.push(format!(
            "confidence {:.2} is below {:.2}",
            output.confidence, AUTO_ACCEPT_MIN_CONFIDENCE
        ));
    }
    match output.temporal_status {
        TemporalStatus::Unknown => reasons.push("evidence was insufficient".to_owned()),
        TemporalStatus::FutureEffective => match output.effective_from {
            Some(date) if date > today => {}
            _ => reasons
                .push("future-effective status lacks a future effective_from date".to_owned()),
        },
        TemporalStatus::Effective if output.effective_from.is_none() => {
            reasons.push("effective status lacks effective_from".to_owned());
        }
        _ => {}
    }
    for effect in &output.effects {
        if effect.application_rule == TransitoryApplicationRule::Unknown {
            reasons.push("application rule is unknown".to_owned());
        }
        if effect.trigger.boundary_type == TemporalBoundaryType::Unknown
            || effect.end_condition.boundary_type == TemporalBoundaryType::Unknown
        {
            reasons.push("a material temporal boundary is unknown".to_owned());
        }
        if effect.verification_status == TemporalVerificationStatus::UnknownMaterial {
            reasons.push("material information needed for classification is unknown".to_owned());
        }
    }
    reasons.sort();
    reasons.dedup();
    reasons
}

fn publication_date_for(evidence: &TemporalEvidence, default: NaiveDate) -> NaiveDate {
    evidence
        .provision_id
        .split_once(":amendment:")
        .and_then(|(_, suffix)| suffix.get(..10))
        .and_then(|date| NaiveDate::parse_from_str(date, "%Y-%m-%d").ok())
        .unwrap_or(default)
}

fn review_item(
    request: &TemporalAnalysisRequest,
    evidence: &TemporalEvidence,
    determination: &TemporalDetermination,
    camara_source_url: Url,
) -> ReviewItem {
    ReviewItem {
        id: format!("review:temporal:{}", evidence.provision_id),
        instrument_id: request.instrument_id.clone(),
        provision_id: evidence.provision_id.clone(),
        exact_issue: determination
            .review_reason
            .clone()
            .unwrap_or_else(|| "temporal conclusion requires review".to_owned()),
        proposed_machine_conclusion: determination.clone(),
        evidence: evidence.clone(),
        camara_source_url,
        formal_source_url: None,
        provision_diff: None,
        resolution_options: vec![
            ReviewResolution::AcceptMachineConclusion,
            ReviewResolution::SetUnknown,
            ReviewResolution::LawyerOverride,
        ],
        status: ReviewItemStatus::Pending,
        reviewer_note: None,
        resolution: None,
        resolved_by: None,
        resolved_at: None,
    }
}

/// Open a review item for a determination that did not route to review on
/// its own — typically a machine-accepted conclusion that the designated
/// legal reviewer wants to correct or enrich. The machine conclusion is
/// preserved verbatim as the item's proposal, and existing items (pending
/// or resolved) are never replaced: a resolved review is immutable. The
/// determination itself is marked `review_required` with the reviewer's
/// reason, so the corpus and dashboards reflect the pending review until
/// it is resolved.
pub fn open_temporal_review(
    items: &mut Vec<ReviewItem>,
    result: &mut TemporalAnalysisResult,
    request: &TemporalAnalysisRequest,
    provision_id: &str,
    reason: &str,
    camara_source_url: &Url,
) -> Result<(), TemporalReviewOpenError> {
    let reason = reason.trim();
    if reason.is_empty() {
        return Err(TemporalReviewOpenError::EmptyReason);
    }
    let id = format!("review:temporal:{provision_id}");
    if items.iter().any(|item| item.id == id) {
        return Err(TemporalReviewOpenError::AlreadyExists(
            provision_id.to_owned(),
        ));
    }
    let determination = result
        .determinations
        .iter_mut()
        .find(|item| item.provision_id == provision_id)
        .ok_or_else(|| TemporalReviewOpenError::DeterminationNotFound(provision_id.to_owned()))?;
    let evidence = request
        .relevant_provisions
        .iter()
        .find(|item| item.provision_id == provision_id)
        .ok_or_else(|| TemporalReviewOpenError::EvidenceNotFound(provision_id.to_owned()))?;
    // Snapshot the proposal before flagging the determination, so the
    // audit record preserves the machine conclusion exactly as accepted.
    let mut item = review_item(request, evidence, determination, camara_source_url.clone());
    reason.clone_into(&mut item.exact_issue);
    determination.review_required = true;
    determination.review_reason = Some(reason.to_owned());
    items.push(item);
    Ok(())
}

pub fn resolve_temporal_review(
    item: &mut ReviewItem,
    determinations: &mut [TemporalDetermination],
    resolution: TemporalReviewResolution,
) -> Result<(), TemporalReviewResolutionError> {
    if item.status != ReviewItemStatus::Pending {
        return Err(TemporalReviewResolutionError::AlreadyResolved);
    }
    let reviewer = resolution.reviewer.trim();
    if reviewer.is_empty() {
        return Err(TemporalReviewResolutionError::EmptyReviewer);
    }
    if let (Some(from), Some(to)) = (resolution.effective_from, resolution.effective_to)
        && to < from
    {
        return Err(TemporalReviewResolutionError::InvalidDateRange);
    }

    let mut verified = item.proposed_machine_conclusion.clone();
    match resolution.resolution {
        ReviewResolution::AcceptMachineConclusion => {
            reject_override_fields(&resolution)?;
        }
        ReviewResolution::SetUnknown => {
            reject_override_fields(&resolution)?;
            verified.temporal_status = TemporalStatus::Unknown;
            verified.effective_from = None;
            verified.effective_to = None;
        }
        ReviewResolution::LawyerOverride => {
            if resolution
                .note
                .as_deref()
                .is_none_or(|note| note.trim().is_empty())
                || (resolution.temporal_status.is_none()
                    && resolution.effective_from.is_none()
                    && resolution.effective_to.is_none()
                    && resolution.effects.is_none())
            {
                return Err(TemporalReviewResolutionError::IncompleteLawyerOverride);
            }
            if let Some(status) = resolution.temporal_status.clone() {
                verified.temporal_status = status;
            }
            if resolution.effective_from.is_some() {
                verified.effective_from = resolution.effective_from;
            }
            if resolution.effective_to.is_some() {
                verified.effective_to = resolution.effective_to;
            }
            if let Some(effects) = resolution.effects.clone() {
                if effects.is_empty() {
                    return Err(TemporalReviewResolutionError::IncompleteLawyerOverride);
                }
                if effects.iter().any(|effect| !valid_effect(effect)) {
                    return Err(TemporalReviewResolutionError::InvalidOverrideEffect);
                }
                verified.effects = effects;
            }
            match verified.temporal_status {
                TemporalStatus::Effective if verified.effective_from.is_none() => {
                    return Err(TemporalReviewResolutionError::MissingEffectiveFrom);
                }
                TemporalStatus::FutureEffective
                    if verified
                        .effective_from
                        .is_none_or(|date| date <= resolution.resolved_at.date_naive()) =>
                {
                    return Err(TemporalReviewResolutionError::InvalidFutureEffectiveDate);
                }
                _ => {}
            }
        }
    }
    verified.basis = Basis::LawyerVerified;
    verified.confidence = 1.0;
    verified.review_required = false;
    verified.review_reason = None;

    let determination = determinations
        .iter_mut()
        .find(|candidate| candidate.provision_id == item.provision_id)
        .ok_or_else(|| {
            TemporalReviewResolutionError::DeterminationNotFound(item.provision_id.clone())
        })?;
    *determination = verified;
    item.status = ReviewItemStatus::Resolved;
    item.reviewer_note = resolution.note;
    item.resolution = Some(resolution.resolution);
    item.resolved_by = Some(reviewer.to_owned());
    item.resolved_at = Some(resolution.resolved_at);
    Ok(())
}

fn valid_effect(effect: &crate::TransitoryEffect) -> bool {
    let description_required = |boundary_type: &TemporalBoundaryType| {
        matches!(
            boundary_type,
            TemporalBoundaryType::RelativePeriod
                | TemporalBoundaryType::ExternalEvent
                | TemporalBoundaryType::CohortExhaustion
                | TemporalBoundaryType::AuthorityAction
                | TemporalBoundaryType::Unknown
        )
    };
    let invalid_boundary = |boundary: &crate::TemporalBoundary| {
        (boundary.boundary_type == TemporalBoundaryType::FixedDate && boundary.date.is_none())
            || (description_required(&boundary.boundary_type)
                && boundary
                    .description
                    .as_deref()
                    .is_none_or(|value| value.trim().is_empty()))
    };
    !effect.affected_scope.trim().is_empty()
        && !invalid_boundary(&effect.trigger)
        && !invalid_boundary(&effect.end_condition)
        && (effect.verification_status != TemporalVerificationStatus::ExternallyVerified
            || (effect.verification_source_url.is_some()
                && effect.verified_event_date.is_some()
                && effect
                    .verification_note
                    .as_deref()
                    .is_some_and(|note| !note.trim().is_empty())))
}

/// Freeze every previous review item — pending or resolved — across a
/// model rerun. Resolution requires an identified human reviewer, and a
/// review already awaiting one cannot be cleared merely because a fresh
/// model run produced a higher-confidence or unambiguous result: the
/// determination and its review item are restored exactly as they were
/// until a human actually resolves them — but only while the evidence is
/// exactly what that review was made against.
///
/// When the evidence has changed, restoring the old determination onto the
/// corpus would silently reinstate a decision made about different text,
/// so it is never applied. The review item itself is never discarded
/// either: `AGENTS.md` requires preserving reviewer identity, timestamp,
/// rationale, source links, and prior machine proposal for every
/// legal-review resolution, and that applies regardless of whether the
/// evidence underneath later changed. The old item is archived verbatim
/// under a version-qualified ID scoped to the evidence it concerns
/// (`…:evidence:<hash>`, or `…:evidence:legacy` for a record that predates
/// evidence hashing), so it cannot collide with a fresh review opened
/// under the canonical ID for the current evidence. Returns the provision
/// IDs archived this way, so the caller can tell the operator a review
/// needs a fresh look at the new text.
///
/// An already-archived item (its ID already carries an `:evidence:`
/// suffix) is immutable history: it is carried forward into `review_items`
/// verbatim on every later call, never re-compared against a
/// determination or re-archived under a further-nested ID. Only the one
/// live item under a provision's canonical ID is ever evaluated for
/// restoration or archival.
#[must_use]
pub fn preserve_temporal_review_history(
    result: &mut TemporalAnalysisResult,
    review_items: &mut Vec<ReviewItem>,
    previous_result: &TemporalAnalysisResult,
    previous_items: &[ReviewItem],
) -> Vec<String> {
    let mut superseded = Vec::new();
    for previous_item in previous_items {
        if is_archived_review_item_id(&previous_item.id) {
            if !review_items.iter().any(|item| item.id == previous_item.id) {
                review_items.push(previous_item.clone());
            }
            continue;
        }
        let Some(previous_determination) = previous_result
            .determinations
            .iter()
            .find(|item| item.provision_id == previous_item.provision_id)
        else {
            continue;
        };
        let Some(current_determination) = result
            .determinations
            .iter_mut()
            .find(|item| item.provision_id == previous_item.provision_id)
        else {
            continue;
        };
        // `current_determination` was just computed by this rerun from
        // the current evidence, so its own (pre-overwrite) hash is the
        // current hash.
        if previous_determination.evidence_sha256 != current_determination.evidence_sha256 {
            let version = if previous_determination.evidence_sha256.is_empty() {
                "legacy".to_owned()
            } else {
                previous_determination.evidence_sha256.clone()
            };
            let mut archived = previous_item.clone();
            archived.id = format!("{}:evidence:{version}", previous_item.id);
            if !review_items.iter().any(|item| item.id == archived.id) {
                review_items.push(archived);
            }
            superseded.push(previous_item.provision_id.clone());
            continue;
        }
        *current_determination = previous_determination.clone();
        if let Some(current_item) = review_items
            .iter_mut()
            .find(|item| item.id == previous_item.id)
        {
            *current_item = previous_item.clone();
        } else {
            review_items.push(previous_item.clone());
        }
    }
    superseded
}

fn is_archived_review_item_id(id: &str) -> bool {
    id.starts_with("review:temporal:") && id.contains(":evidence:")
}

fn reject_override_fields(
    resolution: &TemporalReviewResolution,
) -> Result<(), TemporalReviewResolutionError> {
    if resolution.temporal_status.is_some()
        || resolution.effective_from.is_some()
        || resolution.effective_to.is_some()
        || resolution.effects.is_some()
    {
        Err(TemporalReviewResolutionError::UnexpectedOverrideFields)
    } else {
        Ok(())
    }
}

/// Outcome of reapplying a persisted temporal result after a reparse.
pub struct ReappliedTemporalState {
    /// Determinations that remain grounded, in persisted order, unchanged
    /// from the input.
    pub current: Vec<TemporalDetermination>,
    /// Provision IDs whose evidence text has materially changed, is no
    /// longer present in `current_evidence` at all, or was never hashed in
    /// the first place, and were therefore not re-applied to the corpus.
    /// Temporal analysis must be rerun for these.
    pub stale: Vec<String>,
}

/// Re-apply a persisted temporal result to freshly reparsed provisions, so
/// a reparse never silently erases applied temporal state — including
/// audited lawyer-verified decisions.
///
/// `current_evidence` must be built the same way the temporal-analysis
/// request is built (ordinary transitory text plus any relevant reform
/// evidence, keyed by provision ID): an amendment-event determination's
/// provision ID never appears among canonical `provisions`, only among
/// reform evidence, so a bare provisions-only lookup would always call it
/// stale.
///
/// A determination is re-applied only when the current evidence text
/// hashes identically to `evidence_sha256` — a supporting quotation merely
/// remaining a substring of materially different text is not sufficient,
/// since unrelated edits nearby could still contain the quoted fragment. A
/// determination recorded before this field existed (empty
/// `evidence_sha256`) has no verifiable provenance and is conservatively
/// treated as stale rather than grandfathered in through a substring
/// check.
#[must_use]
#[allow(clippy::implicit_hasher)]
pub fn reapply_temporal_determinations(
    provisions: &mut [crate::Provision],
    determinations: &[TemporalDetermination],
    current_evidence: &HashMap<String, String>,
) -> ReappliedTemporalState {
    let mut current = Vec::new();
    let mut stale = Vec::new();
    for determination in determinations {
        // No verifiable provenance is treated as changed, not grounded: a
        // legacy record predating evidence hashing is marked stale rather
        // than grandfathered in through a substring check, which cannot
        // rule out a materially different surrounding text.
        let grounded = !determination.evidence_sha256.is_empty()
            && current_evidence
                .get(determination.provision_id.as_str())
                .is_some_and(|text| determination.evidence_sha256 == evidence_sha256(text));
        if grounded {
            current.push(determination.clone());
        } else {
            stale.push(determination.provision_id.clone());
        }
    }
    apply_temporal_determinations(provisions, &current);
    ReappliedTemporalState { current, stale }
}

pub fn apply_temporal_determinations(
    provisions: &mut [crate::Provision],
    determinations: &[TemporalDetermination],
) {
    let by_id: HashMap<&str, &TemporalDetermination> = determinations
        .iter()
        .map(|determination| (determination.provision_id.as_str(), determination))
        .collect();
    let mut applied = HashSet::new();
    for provision in provisions {
        if let Some(determination) = by_id.get(provision.id.as_str()) {
            provision.effective_from = determination.effective_from;
            provision.effective_to = determination.effective_to;
            provision.temporal_status = determination.temporal_status.clone();
            provision.temporal_basis = Some(determination.basis.clone());
            provision.temporal_confidence = Some(determination.confidence);
            provision.review_status = if determination.basis == Basis::LawyerVerified {
                ReviewStatus::LawyerVerified
            } else if determination.review_required {
                ReviewStatus::ReviewRequired
            } else {
                ReviewStatus::MachineAccepted
            };
            provision
                .transitory_effects
                .clone_from(&determination.effects);
            applied.insert(provision.id.clone());
        }
    }
}

#[cfg(test)]
mod tests {
    use chrono::{TimeZone, Utc};

    use super::*;
    use crate::{TemporalAnalysisRequest, TemporalEvidence, TransitoryEffectType};

    #[test]
    fn routes_materially_unknown_effect_to_review() {
        let routed = routed_review();
        assert_eq!(routed.review_items.len(), 1);
        assert!(routed.result.determinations[0].review_required);
    }

    #[test]
    fn accepts_open_ended_procedural_survival_without_review() {
        let request = request();
        let batch = model_batch(procedural_survival_effect(
            TemporalVerificationStatus::OpenEndedByDesign,
        ));
        let routed = route(&request, batch).unwrap();

        assert!(routed.review_items.is_empty());
        assert!(!routed.result.determinations[0].review_required);
        assert_eq!(
            routed.result.determinations[0].effects[0]
                .end_condition
                .boundary_type,
            TemporalBoundaryType::CohortExhaustion
        );
    }

    #[test]
    fn external_fact_verification_is_not_legal_ambiguity() {
        let request = request();
        let mut effect =
            procedural_survival_effect(TemporalVerificationStatus::ExternalVerificationRequired);
        effect.effect_type = TransitoryEffectType::StagedCommencement;
        effect.application_rule = TransitoryApplicationRule::NewRuleProspectively;
        effect.end_condition = crate::TemporalBoundary {
            boundary_type: TemporalBoundaryType::ExternalEvent,
            date: None,
            description: Some("publicación de la declaratoria aplicable".to_owned()),
        };
        let routed = route(&request, model_batch(effect)).unwrap();

        assert!(routed.review_items.is_empty());
        assert_eq!(
            routed.result.determinations[0].effects[0].verification_status,
            TemporalVerificationStatus::ExternalVerificationRequired
        );
    }

    #[test]
    fn resolves_review_with_audited_lawyer_override() {
        let mut routed = routed_review();
        let verified_effect =
            procedural_survival_effect(TemporalVerificationStatus::OpenEndedByDesign);
        resolve_temporal_review(
            &mut routed.review_items[0],
            &mut routed.result.determinations,
            TemporalReviewResolution {
                resolution: ReviewResolution::LawyerOverride,
                reviewer: "Lic. Ejemplo".to_owned(),
                note: Some("La condición permanece vigente hasta la autorización.".to_owned()),
                temporal_status: None,
                effective_from: None,
                effective_to: None,
                effects: Some(vec![verified_effect.clone()]),
                resolved_at: Utc.with_ymd_and_hms(2026, 6, 29, 1, 0, 0).unwrap(),
            },
        )
        .unwrap();

        let item = &routed.review_items[0];
        let determination = &routed.result.determinations[0];
        assert_eq!(item.status, ReviewItemStatus::Resolved);
        assert_eq!(item.resolved_by.as_deref(), Some("Lic. Ejemplo"));
        assert_eq!(determination.basis, Basis::LawyerVerified);
        assert_eq!(determination.temporal_status, TemporalStatus::Effective);
        assert_eq!(determination.effects, vec![verified_effect]);
        assert!(!determination.review_required);
    }

    #[test]
    fn rejects_unaudited_lawyer_override() {
        let mut routed = routed_review();
        let error = resolve_temporal_review(
            &mut routed.review_items[0],
            &mut routed.result.determinations,
            TemporalReviewResolution {
                resolution: ReviewResolution::LawyerOverride,
                reviewer: "Lic. Ejemplo".to_owned(),
                note: None,
                temporal_status: Some(TemporalStatus::Effective),
                effective_from: Some(NaiveDate::from_ymd_opt(2027, 4, 1).unwrap()),
                effective_to: None,
                effects: None,
                resolved_at: Utc::now(),
            },
        )
        .unwrap_err();
        assert!(matches!(
            error,
            TemporalReviewResolutionError::IncompleteLawyerOverride
        ));
    }

    #[test]
    fn changed_evidence_does_not_restore_a_stale_reviewed_decision() {
        // A resolved review must not be blindly reinstated over a rerun
        // whose determination reflects materially different evidence: the
        // human decision was made about different text and no longer
        // applies. Only when the evidence hash is unchanged is restoration
        // safe.
        let mut previous = routed_review();
        resolve_temporal_review(
            &mut previous.review_items[0],
            &mut previous.result.determinations,
            TemporalReviewResolution {
                resolution: ReviewResolution::SetUnknown,
                reviewer: "Lic. Ejemplo".to_owned(),
                note: Some("Se requiere fuente formal adicional.".to_owned()),
                temporal_status: None,
                effective_from: None,
                effective_to: None,
                effects: None,
                resolved_at: Utc.with_ymd_and_hms(2026, 6, 29, 1, 0, 0).unwrap(),
            },
        )
        .unwrap();
        assert!(!previous.result.determinations[0].evidence_sha256.is_empty());

        // The rerun is against materially different evidence text (still
        // containing the old quoted fragment, so a bare substring check
        // would wrongly call it unchanged) — its own evidence_sha256
        // therefore differs from the resolved determination's.
        let mut different_request = request();
        different_request.relevant_provisions[0].text = format!(
            "Disposición reformada con requisitos adicionales. {} Nuevo texto final.",
            different_request.relevant_provisions[0].text
        );
        let mut rerun = route(
            &different_request,
            model_batch(procedural_survival_effect(
                TemporalVerificationStatus::UnknownMaterial,
            )),
        )
        .unwrap();
        assert_ne!(
            rerun.result.determinations[0].evidence_sha256,
            previous.result.determinations[0].evidence_sha256
        );

        let superseded = preserve_temporal_review_history(
            &mut rerun.result,
            &mut rerun.review_items,
            &previous.result,
            &previous.review_items,
        );

        // The stale resolved decision is not restored onto the corpus: the
        // fresh determination (and its own fresh routing) stands instead.
        assert_ne!(rerun.result.determinations[0].basis, Basis::LawyerVerified);
        assert_eq!(
            superseded,
            vec![previous.review_items[0].provision_id.clone()]
        );

        // But it is not discarded either: reviewer identity, timestamp,
        // rationale, and the prior machine proposal survive, archived
        // under a versioned ID scoped to the evidence it concerned so it
        // cannot collide with a fresh review of the current text.
        let archived = rerun
            .review_items
            .iter()
            .find(|item| item.status == ReviewItemStatus::Resolved)
            .expect("the resolved decision is archived, not dropped");
        assert_eq!(archived.resolved_by.as_deref(), Some("Lic. Ejemplo"));
        assert_eq!(
            archived.reviewer_note.as_deref(),
            Some("Se requiere fuente formal adicional.")
        );
        assert_ne!(archived.id, previous.review_items[0].id);
        assert!(archived.id.starts_with(&previous.review_items[0].id));
        assert!(archived.id.contains(":evidence:"));

        // A later rerun must preserve the archived record verbatim rather
        // than nesting another evidence suffix onto it.
        let mut second_rerun = route(
            &different_request,
            model_batch(procedural_survival_effect(
                TemporalVerificationStatus::UnknownMaterial,
            )),
        )
        .unwrap();
        let second_superseded = preserve_temporal_review_history(
            &mut second_rerun.result,
            &mut second_rerun.review_items,
            &rerun.result,
            &rerun.review_items,
        );

        assert!(
            second_superseded.is_empty(),
            "the archived item is carried forward verbatim, not superseded again"
        );
        let archived_again = second_rerun
            .review_items
            .iter()
            .find(|item| item.status == ReviewItemStatus::Resolved)
            .expect("the archived decision remains present");
        assert_eq!(archived_again.id, archived.id);
        assert_eq!(archived_again.resolved_by.as_deref(), Some("Lic. Ejemplo"));
        assert_eq!(
            archived_again.reviewer_note.as_deref(),
            Some("Se requiere fuente formal adicional.")
        );
    }

    #[test]
    fn pending_review_survives_a_model_rerun_even_at_high_confidence() {
        // A pending review — whether routed by low confidence or opened by
        // a reviewer — must not be cleared merely because a fresh model
        // run comes back confident and clean. Simulate the rerun producing
        // a *different* batch that would not have routed to review on its
        // own, and confirm the previous pending item and its determination
        // are restored exactly.
        let previous = routed_review();
        assert_eq!(previous.review_items[0].status, ReviewItemStatus::Pending);
        assert!(previous.result.determinations[0].review_required);

        let request = request();
        let mut clean_effect =
            procedural_survival_effect(TemporalVerificationStatus::OpenEndedByDesign);
        clean_effect.application_rule = TransitoryApplicationRule::PriorRuleForExistingMatters;
        let mut rerun = route(&request, model_batch(clean_effect)).unwrap();
        assert!(
            rerun.review_items.is_empty(),
            "the fresh run must not itself route to review, matching the scenario"
        );

        let superseded = preserve_temporal_review_history(
            &mut rerun.result,
            &mut rerun.review_items,
            &previous.result,
            &previous.review_items,
        );

        assert!(
            superseded.is_empty(),
            "evidence is unchanged in this scenario"
        );
        assert_eq!(rerun.review_items.len(), 1);
        assert_eq!(rerun.review_items[0].status, ReviewItemStatus::Pending);
        assert!(rerun.result.determinations[0].review_required);
        assert_eq!(
            rerun.result.determinations[0].effects,
            previous.result.determinations[0].effects
        );
    }

    #[test]
    fn preserves_lawyer_resolution_across_model_reruns() {
        let mut previous = routed_review();
        resolve_temporal_review(
            &mut previous.review_items[0],
            &mut previous.result.determinations,
            TemporalReviewResolution {
                resolution: ReviewResolution::SetUnknown,
                reviewer: "Lic. Ejemplo".to_owned(),
                note: Some("Se requiere fuente formal adicional.".to_owned()),
                temporal_status: None,
                effective_from: None,
                effective_to: None,
                effects: None,
                resolved_at: Utc.with_ymd_and_hms(2026, 6, 29, 1, 0, 0).unwrap(),
            },
        )
        .unwrap();
        let mut rerun = routed_review();

        let superseded = preserve_temporal_review_history(
            &mut rerun.result,
            &mut rerun.review_items,
            &previous.result,
            &previous.review_items,
        );

        assert!(
            superseded.is_empty(),
            "evidence is unchanged in this scenario"
        );
        assert_eq!(rerun.review_items[0].status, ReviewItemStatus::Resolved);
        assert_eq!(
            rerun.result.determinations[0].temporal_status,
            TemporalStatus::Unknown
        );
        assert_eq!(rerun.result.determinations[0].basis, Basis::LawyerVerified);
    }

    #[test]
    fn opens_review_on_machine_accepted_determination_and_preserves_proposal() {
        let request = request();
        let batch = model_batch(procedural_survival_effect(
            TemporalVerificationStatus::OpenEndedByDesign,
        ));
        let mut routed = route(&request, batch).unwrap();
        assert!(routed.review_items.is_empty(), "machine accepted");
        let provision_id = request.relevant_provisions[0].provision_id.clone();
        let source_url: Url = "https://example.com/source.pdf".parse().unwrap();

        open_temporal_review(
            &mut routed.review_items,
            &mut routed.result,
            &request,
            &provision_id,
            "El revisor designado corrige las autoridades responsables.",
            &source_url,
        )
        .unwrap();
        assert_eq!(routed.review_items.len(), 1);
        let item = &routed.review_items[0];
        assert_eq!(item.status, ReviewItemStatus::Pending);
        assert_eq!(
            item.exact_issue,
            "El revisor designado corrige las autoridades responsables."
        );
        // The machine conclusion is preserved verbatim as the proposal —
        // in particular, without the review flag the open sets afterwards.
        assert!((item.proposed_machine_conclusion.confidence - 0.98).abs() < f32::EPSILON);
        assert_eq!(item.proposed_machine_conclusion.basis, Basis::LlmInference);
        assert!(!item.proposed_machine_conclusion.review_required);
        // The determination itself now reflects the pending review, so the
        // corpus and dashboards stop reporting it as machine-accepted.
        assert!(routed.result.determinations[0].review_required);
        assert_eq!(
            routed.result.determinations[0].review_reason.as_deref(),
            Some("El revisor designado corrige las autoridades responsables.")
        );

        // A second open for the same provision must not touch the item.
        let error = open_temporal_review(
            &mut routed.review_items,
            &mut routed.result,
            &request,
            &provision_id,
            "duplicado",
            &source_url,
        )
        .unwrap_err();
        assert!(matches!(error, TemporalReviewOpenError::AlreadyExists(_)));

        // Resolving with a lawyer override keeps the original proposal in
        // the audit record while the determination becomes lawyer-verified.
        let mut corrected =
            procedural_survival_effect(TemporalVerificationStatus::OpenEndedByDesign);
        corrected.responsible_authorities =
            vec!["Comisión Nacional Bancaria y de Valores".to_owned()];
        resolve_temporal_review(
            &mut routed.review_items[0],
            &mut routed.result.determinations,
            TemporalReviewResolution {
                resolution: ReviewResolution::LawyerOverride,
                reviewer: "JRH".to_owned(),
                note: Some("Autoridad responsable identificada.".to_owned()),
                temporal_status: None,
                effective_from: None,
                effective_to: None,
                effects: Some(vec![corrected.clone()]),
                resolved_at: Utc.with_ymd_and_hms(2026, 7, 3, 0, 0, 0).unwrap(),
            },
        )
        .unwrap();
        let determination = &routed.result.determinations[0];
        assert_eq!(determination.basis, Basis::LawyerVerified);
        assert_eq!(determination.effects, vec![corrected]);
        assert_eq!(
            routed.review_items[0].proposed_machine_conclusion.basis,
            Basis::LlmInference
        );
        assert_eq!(routed.review_items[0].resolved_by.as_deref(), Some("JRH"));
    }

    #[test]
    fn reapplies_persisted_determinations_after_reparse_unless_stale() {
        let request = request();
        let batch = model_batch(procedural_survival_effect(
            TemporalVerificationStatus::OpenEndedByDesign,
        ));
        let routed = route(&request, batch).unwrap();
        let provision_id = request.relevant_provisions[0].provision_id.clone();
        let original_text = request.relevant_provisions[0].text.clone();

        let make_provision = |text: &str| crate::Provision {
            schema_version: SCHEMA_VERSION.to_owned(),
            id: provision_id.clone(),
            instrument_id: "urn:lex-mx:test".to_owned(),
            provision_type: crate::ProvisionType::Transitory,
            label: "Segundo".to_owned(),
            number: "Segundo".to_owned(),
            heading_context: crate::HeadingContext {
                libro: None,
                title: None,
                chapter: None,
                section: None,
                apartado: None,
            },
            text: text.to_owned(),
            publication_date: NaiveDate::from_ymd_opt(2025, 11, 14).unwrap(),
            effective_from: None,
            effective_to: None,
            temporal_status: TemporalStatus::Unknown,
            temporal_basis: None,
            temporal_confidence: None,
            review_status: ReviewStatus::NotAnalyzed,
            transitory_effects: Vec::new(),
            amendment_marks: Vec::new(),
        };
        let evidence_map = |text: &str| HashMap::from([(provision_id.clone(), text.to_owned())]);

        // route_temporal_analysis already stamps evidence_sha256.
        assert!(!routed.result.determinations[0].evidence_sha256.is_empty());

        // A reparse that reproduces the same text keeps the determination.
        let mut reparsed = vec![make_provision(&original_text)];
        let outcome = reapply_temporal_determinations(
            &mut reparsed,
            &routed.result.determinations,
            &evidence_map(&original_text),
        );
        assert_eq!((outcome.current.len(), outcome.stale.len()), (1, 0));
        assert_eq!(reparsed[0].temporal_status, TemporalStatus::Effective);
        assert_eq!(reparsed[0].review_status, ReviewStatus::MachineAccepted);

        // Changed text no longer grounds the supporting quotation: the
        // determination is reported stale, not silently applied.
        let mut changed = vec![make_provision("Texto reformado sin la cita original.")];
        let outcome = reapply_temporal_determinations(
            &mut changed,
            &routed.result.determinations,
            &evidence_map("Texto reformado sin la cita original."),
        );
        assert_eq!((outcome.current.len(), outcome.stale.len()), (0, 1));
        assert_eq!(outcome.stale[0], provision_id);
        assert_eq!(changed[0].temporal_status, TemporalStatus::Unknown);
        assert_eq!(changed[0].review_status, ReviewStatus::NotAnalyzed);

        // A provision absent from current_evidence altogether (removed, or
        // renamed) is stale, not silently dropped without explanation.
        let mut orphaned = vec![make_provision(&original_text)];
        let outcome = reapply_temporal_determinations(
            &mut orphaned,
            &routed.result.determinations,
            &HashMap::new(),
        );
        assert_eq!((outcome.current.len(), outcome.stale.len()), (0, 1));
    }

    #[test]
    fn strict_hash_catches_a_substring_that_survives_a_material_change() {
        // A determination that already carries its evidence hash must not
        // be re-applied just because its quoted substring happens to still
        // appear somewhere in materially different text — the exact defect
        // a pure substring check misses.
        let mut determination = sample_determination();
        determination.supporting_text = vec!["a partir de la entrada en vigor".to_owned()];
        determination.evidence_sha256 =
            evidence_sha256("Plazo de seis meses a partir de la entrada en vigor de la ley.");
        let determinations = vec![determination];

        let mut provisions = vec![sample_provision_with_text(
            "Plazo de dieciocho meses, no de seis, a partir de la entrada en vigor de la ley, \
             con requisitos adicionales.",
        )];
        let current_evidence =
            HashMap::from([(provisions[0].id.clone(), provisions[0].text.clone())]);

        let outcome =
            reapply_temporal_determinations(&mut provisions, &determinations, &current_evidence);
        assert_eq!(outcome.current.len(), 0);
        assert_eq!(outcome.stale.len(), 1);
        assert_eq!(provisions[0].review_status, ReviewStatus::NotAnalyzed);
    }

    #[test]
    fn legacy_record_without_a_hash_is_marked_stale_not_grandfathered() {
        // A record predating evidence hashing has no verifiable
        // provenance. Even though the old quotation still substring-matches
        // exactly, it must not be silently reinstated — that heuristic is
        // exactly what let a stale reviewed decision survive unrelated
        // text changes. It is reported stale so temporal analysis reruns.
        let mut determination = sample_determination();
        determination.supporting_text = vec!["texto de apoyo".to_owned()];
        determination.evidence_sha256 = String::new();
        let determinations = vec![determination];

        let mut provisions = vec![sample_provision_with_text(
            "Contiene el texto de apoyo citado.",
        )];
        let current_evidence =
            HashMap::from([(provisions[0].id.clone(), provisions[0].text.clone())]);

        let outcome =
            reapply_temporal_determinations(&mut provisions, &determinations, &current_evidence);
        assert!(outcome.current.is_empty());
        assert_eq!(outcome.stale.len(), 1);
        assert_eq!(provisions[0].review_status, ReviewStatus::NotAnalyzed);
    }

    fn sample_provision_with_text(text: &str) -> crate::Provision {
        crate::Provision {
            schema_version: SCHEMA_VERSION.to_owned(),
            id: "urn:lex-mx:test:transitory:segundo".to_owned(),
            instrument_id: "urn:lex-mx:test".to_owned(),
            provision_type: crate::ProvisionType::Transitory,
            label: "Segundo".to_owned(),
            number: "Segundo".to_owned(),
            heading_context: crate::HeadingContext {
                libro: None,
                title: None,
                chapter: None,
                section: None,
                apartado: None,
            },
            text: text.to_owned(),
            publication_date: NaiveDate::from_ymd_opt(2025, 11, 14).unwrap(),
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

    fn sample_determination() -> TemporalDetermination {
        TemporalDetermination {
            provision_id: "urn:lex-mx:test:transitory:segundo".to_owned(),
            temporal_status: TemporalStatus::Effective,
            publication_date: NaiveDate::from_ymd_opt(2025, 11, 14).unwrap(),
            effective_from: Some(NaiveDate::from_ymd_opt(2025, 11, 15).unwrap()),
            effective_to: None,
            confidence: 0.98,
            basis: Basis::LlmInference,
            supporting_text: Vec::new(),
            review_required: false,
            review_reason: None,
            model: "gpt-test".to_owned(),
            prompt_version: "temporal-v2".to_owned(),
            effects: vec![procedural_survival_effect(
                TemporalVerificationStatus::OpenEndedByDesign,
            )],
            evidence_sha256: String::new(),
        }
    }

    #[test]
    fn rejects_non_source_supporting_text() {
        let request = request();
        let batch = TemporalModelBatch {
            determinations: vec![TemporalModelDetermination {
                provision_id: request.relevant_provisions[0].provision_id.clone(),
                temporal_status: TemporalStatus::Effective,
                effective_from: Some(NaiveDate::from_ymd_opt(2027, 4, 1).unwrap()),
                effective_to: None,
                confidence: 0.99,
                supporting_text: vec!["invented quote".to_owned()],
                effects: vec![procedural_survival_effect(
                    TemporalVerificationStatus::OpenEndedByDesign,
                )],
            }],
        };
        let error = route_temporal_analysis(
            &request,
            batch,
            TemporalAnalysisMetadata {
                request_sha256: "hash".to_owned(),
                response_sha256: "response-hash".to_owned(),
                response_id: None,
                model: "gpt-test".to_owned(),
                analyzed_at: Utc::now(),
            },
            &"https://example.com/source.pdf".parse().unwrap(),
        )
        .unwrap_err();
        assert!(matches!(
            error,
            TemporalRoutingError::UnsupportedCitation(_)
        ));
    }

    fn request() -> TemporalAnalysisRequest {
        TemporalAnalysisRequest {
            schema_version: SCHEMA_VERSION.to_owned(),
            prompt_version: "temporal-v2".to_owned(),
            instrument_id: "urn:lex-mx:test".to_owned(),
            publication_date: NaiveDate::from_ymd_opt(2025, 11, 14).unwrap(),
            latest_reform_date: None,
            relevant_provisions: vec![TemporalEvidence {
                provision_id: "urn:lex-mx:test:amendment:2025-11-14:transitory:second".to_owned(),
                label: "Segundo".to_owned(),
                text: "La aplicación entrará en vigor cuando se emita la declaratoria.".to_owned(),
            }],
            required_output_schema: "schema.json".to_owned(),
        }
    }

    fn routed_review() -> RoutedTemporalAnalysis {
        let request = request();
        let mut effect = procedural_survival_effect(TemporalVerificationStatus::UnknownMaterial);
        effect.application_rule = TransitoryApplicationRule::Unknown;
        route(&request, model_batch(effect)).unwrap()
    }

    fn route(
        request: &TemporalAnalysisRequest,
        batch: TemporalModelBatch,
    ) -> Result<RoutedTemporalAnalysis, TemporalRoutingError> {
        route_temporal_analysis(
            request,
            batch,
            TemporalAnalysisMetadata {
                request_sha256: "hash".to_owned(),
                response_sha256: "response-hash".to_owned(),
                response_id: Some("response".to_owned()),
                model: "gpt-test".to_owned(),
                analyzed_at: Utc.with_ymd_and_hms(2026, 6, 29, 0, 0, 0).unwrap(),
            },
            &"https://example.com/source.pdf".parse().unwrap(),
        )
    }

    fn model_batch(effect: crate::TransitoryEffect) -> TemporalModelBatch {
        let request = request();
        TemporalModelBatch {
            determinations: vec![TemporalModelDetermination {
                provision_id: request.relevant_provisions[0].provision_id.clone(),
                temporal_status: TemporalStatus::Effective,
                effective_from: Some(NaiveDate::from_ymd_opt(2025, 11, 15).unwrap()),
                effective_to: None,
                confidence: 0.98,
                supporting_text: vec![
                    "entrará en vigor cuando se emita la declaratoria".to_owned(),
                ],
                effects: vec![effect],
            }],
        }
    }

    fn procedural_survival_effect(
        verification_status: TemporalVerificationStatus,
    ) -> crate::TransitoryEffect {
        crate::TransitoryEffect {
            effect_type: TransitoryEffectType::ProceduralSurvival,
            affected_scope: "procedimientos iniciados antes de la reforma".to_owned(),
            application_rule: TransitoryApplicationRule::PriorRuleForExistingMatters,
            trigger: crate::TemporalBoundary {
                boundary_type: TemporalBoundaryType::EffectiveDate,
                date: Some(NaiveDate::from_ymd_opt(2025, 11, 15).unwrap()),
                description: None,
            },
            end_condition: crate::TemporalBoundary {
                boundary_type: TemporalBoundaryType::CohortExhaustion,
                date: None,
                description: Some("cuando concluyan los procedimientos existentes".to_owned()),
            },
            responsible_authorities: Vec::new(),
            verification_status,
            verification_source_url: None,
            verified_event_date: None,
            verification_note: None,
        }
    }
}
