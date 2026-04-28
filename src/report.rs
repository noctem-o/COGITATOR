use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum DriftIssue {
    TranscriptSchemaMismatch {
        expected: u32,
        actual: u32,
    },
    TranscriptModeMismatch {
        expected: String,
        actual: String,
    },
    TranscriptLengthMismatch {
        expected: u32,
        actual: u32,
    },
    ToolCallCountMismatch {
        expected: u32,
        actual: u32,
    },
    ToolStepMismatch {
        index: u32,
        expected: u32,
        actual: u32,
    },
    ToolCallIndexMismatch {
        index: u32,
        expected: u32,
        actual: u32,
    },
    ToolRequestMismatch {
        index: u32,
    },
    #[serde(rename = "tool_response_hash_mismatch")]
    ToolOutcomeMismatch {
        index: u32,
    },
    ToolFaultMismatch {
        index: u32,
    },
    UnexpectedToolRequest {
        index: u32,
    },
    OrdealOutputMismatch {
        step: u32,
        tool_call_idx: u32,
        tool_name: String,
        json_pointer: String,
        label: String,
        issue_kind: String,
        expected: String,
        actual: String,
    },
}

#[cfg(test)]
mod tests {
    use super::DriftIssue;

    #[test]
    fn drift_issue_serializes_with_kind_tag_and_fields() {
        let issue = DriftIssue::ToolStepMismatch {
            index: 3,
            expected: 10,
            actual: 11,
        };
        let value = serde_json::to_value(&issue).expect("serialize");
        assert_eq!(value["kind"], "tool_step_mismatch");
        assert_eq!(value["index"], 3);
        assert_eq!(value["expected"], 10);
        assert_eq!(value["actual"], 11);
    }

    #[test]
    fn tool_outcome_variant_uses_legacy_schema_kind_name() {
        let issue = DriftIssue::ToolOutcomeMismatch { index: 7 };
        let value = serde_json::to_value(&issue).expect("serialize");
        assert_eq!(value["kind"], "tool_response_hash_mismatch");
        assert_eq!(value["index"], 7);
    }
}
