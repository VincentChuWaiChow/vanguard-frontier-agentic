//! Owned governance observations composed from the existing report service.
//! Generations are caller invalidation tokens, not a freshness or trust grant.
//! Filesystem reads remain sequential; this is not an atomic filesystem snapshot.
use std::{collections::BTreeMap, sync::Arc};

use crate::headless::reporter::ReportService;
use crate::models::report::{ReportInputStatus, ReportRequest, ReportResult, ReportType};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GovernanceStatus {
    Current,
    NotConfigured,
    Unavailable,
    Partial,
    Stale,
}

/// Retains at most the current attempt and one last-good report. No predecessor
/// snapshot is held, so repeated failed refreshes cannot grow a history chain.
#[derive(Debug)]
pub struct GovernanceSnapshot {
    request: ReportRequest,
    source_generation: u64,
    request_generation: u64,
    status: GovernanceStatus,
    sections: BTreeMap<String, ReportInputStatus>,
    attempt: Arc<ReportResult>,
    last_good: Option<Arc<ReportResult>>,
}

impl GovernanceSnapshot {
    pub fn generate(
        request: &ReportRequest,
        source_generation: u64,
        request_generation: u64,
        previous: Option<&Self>,
    ) -> Self {
        let attempt = Arc::new(ReportService::generate(request));
        let reports = if request.report_types.is_empty() {
            vec![ReportType::Summary]
        } else {
            request.report_types.clone()
        };
        let reports: Vec<_> = reports
            .into_iter()
            .flat_map(|report| {
                if report == ReportType::All {
                    crate::headless::reporter::all_report_types()
                } else {
                    vec![report]
                }
            })
            .collect();
        let sections: BTreeMap<_, _> = reports
            .iter()
            .map(|report| {
                (
                    report.as_str().to_owned(),
                    attempt.quality.for_report(report),
                )
            })
            .collect();
        let inputs: Vec<_> = sections.values().copied().collect();
        let mut status = if inputs
            .iter()
            .all(|status| *status == ReportInputStatus::Current)
        {
            GovernanceStatus::Current
        } else if inputs
            .iter()
            .all(|status| *status == ReportInputStatus::Unavailable)
        {
            GovernanceStatus::Unavailable
        } else if inputs
            .iter()
            .all(|status| *status == ReportInputStatus::NotConfigured)
        {
            GovernanceStatus::NotConfigured
        } else {
            GovernanceStatus::Partial
        };
        let last_good = if status == GovernanceStatus::Current {
            Some(Arc::clone(&attempt))
        } else {
            previous
                .filter(|old| {
                    old.source_generation == source_generation
                        && old.request_generation == request_generation
                        && old.request == *request
                })
                .and_then(|old| old.last_good.clone())
        };
        if matches!(
            status,
            GovernanceStatus::Unavailable | GovernanceStatus::Partial
        ) && last_good.is_some()
        {
            status = GovernanceStatus::Stale;
        }
        Self {
            request: request.clone(),
            source_generation,
            request_generation,
            status,
            sections,
            attempt,
            last_good,
        }
    }

    /// Quality of each requested section in the latest attempt, not retained data.
    pub fn sections(&self) -> &BTreeMap<String, ReportInputStatus> {
        &self.sections
    }

    pub fn status(&self) -> GovernanceStatus {
        self.status
    }
    pub fn source_generation(&self) -> u64 {
        self.source_generation
    }
    pub fn request_generation(&self) -> u64 {
        self.request_generation
    }
    /// Original last-good payload/quality/provenance when explicitly stale.
    /// Adapters must display `status()` alongside this retained result.
    pub fn report(&self) -> &ReportResult {
        if self.status == GovernanceStatus::Stale {
            if let Some(last_good) = &self.last_good {
                return last_good;
            }
        }
        &self.attempt
    }
    /// Latest attempt quality and exclusions, including failed refresh inputs.
    pub fn attempt(&self) -> &ReportResult {
        &self.attempt
    }
}
