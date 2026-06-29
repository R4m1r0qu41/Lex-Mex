use std::collections::{HashMap, HashSet};

use chrono::NaiveDate;
use thiserror::Error;
use url::Url;

use crate::{
    Basis, ReviewItem, ReviewItemStatus, ReviewResolution, ReviewStatus, SCHEMA_VERSION,
    TemporalAnalysisMetadata, TemporalAnalysisRequest, TemporalAnalysisResult,
    TemporalDetermination, TemporalEvidence, TemporalModelBatch, TemporalModelDetermination,
    TemporalStatus,
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
            provision.temporal_basis = Some(Basis::LlmInference);
            provision.temporal_confidence = Some(determination.confidence);
            provision.review_status = if determination.review_required {
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
        let routed = route_temporal_analysis(
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
        .unwrap();
        assert_eq!(routed.review_items.len(), 1);
        assert!(routed.result.determinations[0].review_required);
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
}
