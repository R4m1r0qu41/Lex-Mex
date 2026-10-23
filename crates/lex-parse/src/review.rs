//! Application of audited, operator-supplied review records to references.

use std::collections::HashSet;

use anyhow::Result;
use lex_core::{Provision, ReferenceEdge, ReferenceResolutionStatus};

/// Apply only explicit exact-span, exact-source human decisions. Every supplied
/// approval must match one missing target; stale approvals fail closed on rerun.
pub fn apply_outdated_reference_approvals<S: std::hash::BuildHasher>(
    references: &mut [ReferenceEdge],
    provisions: &[Provision],
    approvals: &[lex_core::OutdatedReferenceApproval],
    source_sha256: &str,
    known_targets: &HashSet<String, S>,
) -> Result<()> {
    for approval in approvals {
        approval.validate().map_err(anyhow::Error::msg)?;
        anyhow::ensure!(
            approval.source_sha256 == source_sha256,
            "outdated-reference approval acquisition hash mismatch"
        );
        let source = provisions
            .iter()
            .find(|p| p.id == approval.source_provision_id)
            .ok_or_else(|| anyhow::anyhow!("outdated-reference approval source missing"))?;
        let matches: Vec<_> = references
            .iter()
            .enumerate()
            .filter(|(_, edge)| approval.matches(edge, &source.text))
            .map(|(i, _)| i)
            .collect();
        anyhow::ensure!(
            matches.len() == 1,
            "outdated-reference approval must match exactly one extracted edge"
        );
        let edge = &mut references[matches[0]];
        anyhow::ensure!(
            !known_targets.contains(&edge.target_provision_id),
            "outdated-reference target now available; review/relink required"
        );
        anyhow::ensure!(
            edge.resolution_status == ReferenceResolutionStatus::Unresolved
                && edge.outdated_reference_approval.is_none(),
            "outdated-reference approval cannot overwrite a resolved or previously approved edge"
        );
        edge.resolution_status = ReferenceResolutionStatus::HistoricalSupersededReferenceOutdated;
        edge.outdated_reference_approval = Some(approval.clone());
    }
    Ok(())
}

/// Attach exact-span legal operator notes. A note never changes an edge's target
/// or status; every supplied note must match exactly one extracted edge, so a
/// note whose citation, span or provision text has changed fails closed.
pub fn apply_legal_operator_reviews(
    references: &mut [ReferenceEdge],
    provisions: &[Provision],
    reviews: &[lex_core::LegalOperatorReview],
) -> Result<()> {
    for review in reviews {
        review.validate().map_err(anyhow::Error::msg)?;
        let source = provisions
            .iter()
            .find(|p| p.id == review.source_provision_id)
            .ok_or_else(|| anyhow::anyhow!("legal operator review source missing"))?;
        let matches: Vec<_> = references
            .iter()
            .enumerate()
            .filter(|(_, edge)| review.matches(edge, &source.text))
            .map(|(i, _)| i)
            .collect();
        anyhow::ensure!(
            matches.len() == 1,
            "legal operator review must match exactly one extracted edge"
        );
        let edge = &mut references[matches[0]];
        anyhow::ensure!(
            edge.legal_operator_review.is_none(),
            "legal operator review cannot overwrite a previous review"
        );
        edge.legal_operator_review = Some(review.clone());
    }
    Ok(())
}
