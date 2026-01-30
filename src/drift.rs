use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fs::File;
use std::path::Path;

use crate::canonical_json;
use crate::report::DriftIssue;
use crate::tooling::ToolTranscriptRecord;

pub const DRIFT_REPORT_SCHEMA_VERSION: u32 = 1;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DriftReport {
    pub schema_version: u32,
    pub baseline_hash: String,
    pub candidate_hash: String,
    pub issues: Vec<DriftIssue>,
}

impl DriftReport {
    pub fn new(baseline_hash: String, candidate_hash: String, issues: Vec<DriftIssue>) -> Self {
        Self {
            schema_version: DRIFT_REPORT_SCHEMA_VERSION,
            baseline_hash,
            candidate_hash,
            issues,
        }
    }

    pub fn has_drift(&self) -> bool {
        !self.issues.is_empty()
    }
}

pub fn compare_transcripts(
    baseline: &ToolTranscriptRecord,
    candidate: &ToolTranscriptRecord,
) -> Result<DriftReport> {
    let baseline_hash = transcript_hash(baseline)?;
    let candidate_hash = transcript_hash(candidate)?;

    let mut issues = Vec::new();

    if baseline.entries.len() != candidate.entries.len() {
        issues.push(DriftIssue::ToolCountMismatch {
            expected: baseline.entries.len() as u32,
            actual: candidate.entries.len() as u32,
        });
    }

    let min_len = baseline.entries.len().min(candidate.entries.len());
    for i in 0..min_len {
        let b_call = &baseline.entries[i];
        let c_call = &candidate.entries[i];

        if b_call.step != c_call.step {
            issues.push(DriftIssue::ToolStepMismatch {
                index: i as u32,
                expected: b_call.step,
                actual: c_call.step,
            });
        }

        if b_call.tool_call_idx != c_call.tool_call_idx {
            issues.push(DriftIssue::ToolCallIndexMismatch {
                index: i as u32,
                expected: b_call.tool_call_idx,
                actual: c_call.tool_call_idx,
            });
        }

        if b_call.tool_name != c_call.tool_name || b_call.request != c_call.request {
            issues.push(DriftIssue::ToolRequestMismatch { index: i as u32 });
        }

        if b_call.outcome != c_call.outcome {
            issues.push(DriftIssue::ToolResponseMismatch { index: i as u32 });
        }

        if b_call.fault != c_call.fault {
            issues.push(DriftIssue::ChaosStateMismatch {
                index: i as u32,
                detail: format!("fault mismatch at index {}", i),
            });
        }
    }

    Ok(DriftReport::new(baseline_hash, candidate_hash, issues))
}

pub fn write_drift_report(path: &Path, report: &DriftReport) -> Result<()> {
    canonical_json::write_json(path, report, "drift report")?;
    Ok(())
}

pub fn read_drift_report(path: &Path) -> Result<DriftReport> {
    let file = File::open(path).with_context(|| "failed to open drift report")?;
    let report: DriftReport =
        serde_json::from_reader(file).with_context(|| "failed to parse drift report")?;
    Ok(report)
}

fn transcript_hash(record: &ToolTranscriptRecord) -> Result<String> {
    let bytes = canonical_json::to_vec(record).context("serialize transcript")?;
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    let digest = hasher.finalize();
    Ok(crate::hex::encode(&digest))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tooling::{ToolCall, ToolMode, ToolOutcome};

    #[test]
    fn test_transcript_hash_determinism() {
        let record = ToolTranscriptRecord {
            schema_version: 3,
            mode: ToolMode::Live,
            entries: vec![ToolCall {
                step: 0,
                tool_call_idx: 0,
                tool_name: "test.tool".to_string(),
                request: serde_json::json!({"key": "value"}),
                outcome: ToolOutcome::Ok {
                    output: serde_json::json!({"result": "ok"}),
                    simulated_latency_ms: None,
                },
                fault: None,
            }],
        };

        let hash1 = transcript_hash(&record).unwrap();
        let hash2 = transcript_hash(&record).unwrap();

        assert_eq!(hash1, hash2);
        assert_eq!(hash1.len(), 64); // SHA256 hex = 64 chars
    }

    #[test]
    fn test_compare_identical_transcripts() {
        let record = ToolTranscriptRecord {
            schema_version: 3,
            mode: ToolMode::Live,
            entries: vec![],
        };

        let report = compare_transcripts(&record, &record).unwrap();
        assert!(!report.has_drift());
        assert_eq!(report.baseline_hash, report.candidate_hash);
    }

    #[test]
    fn test_compare_different_length() {
        let baseline = ToolTranscriptRecord {
            schema_version: 3,
            mode: ToolMode::Live,
            entries: vec![ToolCall {
                step: 0,
                tool_call_idx: 0,
                tool_name: "test.tool".to_string(),
                request: serde_json::json!({}),
                outcome: ToolOutcome::Ok {
                    output: serde_json::json!({}),
                    simulated_latency_ms: None,
                },
                fault: None,
            }],
        };

        let candidate = ToolTranscriptRecord {
            schema_version: 3,
            mode: ToolMode::Live,
            entries: vec![],
        };

        let report = compare_transcripts(&baseline, &candidate).unwrap();
        assert!(report.has_drift());
        assert_eq!(report.issues.len(), 1);
        assert!(matches!(
            report.issues[0],
            DriftIssue::ToolCountMismatch { .. }
        ));
    }
}
