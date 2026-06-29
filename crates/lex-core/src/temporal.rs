use std::collections::{HashMap, HashSet};

use chrono::NaiveDate;
use thiserror::Error;
use url::Url;

use crate::{
    Basis, ReviewItem, ReviewItemStatus, ReviewResolution, ReviewStatus, SCHEMA_VERSION,
    TemporalAnalysisMetadata, TemporalAnalysisRequest, TemporalAnalysisResult,
    TemporalDetermination, TemporalEvidence, TemporalModelBatch, TemporalModelDetermination,
    TemporalReviewResolution, TemporalStatus,
};

const AUTO_ACCEPT_MIN_CONFIDENCE: f32 = 0.92;

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
}

#[derive(Debug, Error)]
pub enum TemporalReviewResolutionError {
    #[error("review item is already resolved")]
    AlreadyResolved,
    #[error("reviewer identity cannot be empty")]
    EmptyReviewer,
    #[error("lawyer_override requires --temporal-status and a non-empty --note")]
    IncompleteLawyerOverride,
    #[error("override fields are only valid with lawyer_override")]
    UnexpectedOverrideFields,
    #[error("effective_to precedes effective_from")]
    InvalidDateRange,
    #[error("effective lawyer override requires --effective-from")]
    MissingEffectiveFrom,
    #[error("future_effective lawyer override requires a future --effective-from")]
    InvalidFutureEffectiveDate,
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
        TemporalStatus::PartiallyEffective => {
            reasons.push("partial effectiveness requires human review".to_owned());
        }
        TemporalStatus::ConditionallyEffective => {
            reasons.push("conditional effectiveness requires human review".to_owned());
        }
        TemporalStatus::RepealedWithSurvival => {
            reasons.push("survival after repeal requires human review".to_owned());
        }
        TemporalStatus::Repealed => {
            reasons.push("repeal classification requires human review".to_owned());
        }
        TemporalStatus::Superseded => {
            reasons.push("supersession requires human review".to_owned());
        }
        TemporalStatus::TemporarilyApplicable => {
            reasons.push("temporary applicability requires human review".to_owned());
        }
        TemporalStatus::PendingConsolidation => {
            reasons.push("pending consolidation requires human review".to_owned());
        }
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
            let status = resolution
                .temporal_status
                .clone()
                .ok_or(TemporalReviewResolutionError::IncompleteLawyerOverride)?;
            if resolution
                .note
                .as_deref()
                .is_none_or(|note| note.trim().is_empty())
            {
                return Err(TemporalReviewResolutionError::IncompleteLawyerOverride);
            }
            verified.temporal_status = status;
            verified.effective_from = resolution.effective_from;
            verified.effective_to = resolution.effective_to;
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

pub fn preserve_temporal_review_history(
    result: &mut TemporalAnalysisResult,
    review_items: &mut Vec<ReviewItem>,
    previous_result: &TemporalAnalysisResult,
    previous_items: &[ReviewItem],
) {
    for previous_item in previous_items
        .iter()
        .filter(|item| item.status == ReviewItemStatus::Resolved)
    {
        if let Some(previous_determination) = previous_result
            .determinations
            .iter()
            .find(|item| item.provision_id == previous_item.provision_id)
            && let Some(current_determination) = result
                .determinations
                .iter_mut()
                .find(|item| item.provision_id == previous_item.provision_id)
        {
            *current_determination = previous_determination.clone();
        }
        if let Some(current_item) = review_items
            .iter_mut()
            .find(|item| item.id == previous_item.id)
        {
            *current_item = previous_item.clone();
        } else {
            review_items.push(previous_item.clone());
        }
    }
}

fn reject_override_fields(
    resolution: &TemporalReviewResolution,
) -> Result<(), TemporalReviewResolutionError> {
    if resolution.temporal_status.is_some()
        || resolution.effective_from.is_some()
        || resolution.effective_to.is_some()
    {
        Err(TemporalReviewResolutionError::UnexpectedOverrideFields)
    } else {
        Ok(())
    }
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
            applied.insert(provision.id.clone());
        }
    }
}

#[cfg(test)]
mod tests {
    use chrono::{TimeZone, Utc};

    use super::*;
    use crate::{TemporalAnalysisRequest, TemporalEvidence};

    #[test]
    fn routes_conditional_effectiveness_to_review() {
        let routed = routed_review();
        assert_eq!(routed.review_items.len(), 1);
        assert!(routed.result.determinations[0].review_required);
    }

    #[test]
    fn resolves_review_with_audited_lawyer_override() {
        let mut routed = routed_review();
        resolve_temporal_review(
            &mut routed.review_items[0],
            &mut routed.result.determinations,
            TemporalReviewResolution {
                resolution: ReviewResolution::LawyerOverride,
                reviewer: "Lic. Ejemplo".to_owned(),
                note: Some("La condición permanece vigente hasta la autorización.".to_owned()),
                temporal_status: Some(TemporalStatus::TemporarilyApplicable),
                effective_from: Some(NaiveDate::from_ymd_opt(2027, 4, 1).unwrap()),
                effective_to: None,
                resolved_at: Utc.with_ymd_and_hms(2026, 6, 29, 1, 0, 0).unwrap(),
            },
        )
        .unwrap();

        let item = &routed.review_items[0];
        let determination = &routed.result.determinations[0];
        assert_eq!(item.status, ReviewItemStatus::Resolved);
        assert_eq!(item.resolved_by.as_deref(), Some("Lic. Ejemplo"));
        assert_eq!(determination.basis, Basis::LawyerVerified);
        assert_eq!(
            determination.temporal_status,
            TemporalStatus::TemporarilyApplicable
        );
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
                resolved_at: Utc.with_ymd_and_hms(2026, 6, 29, 1, 0, 0).unwrap(),
            },
        )
        .unwrap();
        let mut rerun = routed_review();

        preserve_temporal_review_history(
            &mut rerun.result,
            &mut rerun.review_items,
            &previous.result,
            &previous.review_items,
        );

        assert_eq!(rerun.review_items[0].status, ReviewItemStatus::Resolved);
        assert_eq!(
            rerun.result.determinations[0].temporal_status,
            TemporalStatus::Unknown
        );
        assert_eq!(rerun.result.determinations[0].basis, Basis::LawyerVerified);
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
            prompt_version: "temporal-v1".to_owned(),
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
        let batch = TemporalModelBatch {
            determinations: vec![TemporalModelDetermination {
                provision_id: request.relevant_provisions[0].provision_id.clone(),
                temporal_status: TemporalStatus::ConditionallyEffective,
                effective_from: Some(NaiveDate::from_ymd_opt(2027, 4, 1).unwrap()),
                effective_to: None,
                confidence: 0.98,
                supporting_text: vec![
                    "entrará en vigor cuando se emita la declaratoria".to_owned(),
                ],
            }],
        };
        route_temporal_analysis(
            &request,
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
        .unwrap()
    }
}
