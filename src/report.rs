use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum DriftIssue {
    ToolCallCountMismatch {
        expected: u32,
        actual: u32,
    },
    ToolStepMismatch {
        index: u32,
        expected: u32,
        actual: u32,
    },
    ToolRequestMismatch {
        index: u32,
    },
    ToolCallIndexMismatch {
        index: u32,
        expected: u32,
        actual: u32,
    },
    ToolFaultMismatch {
        index: u32,
    },
    ToolResponseHashMismatch {
        index: u32,
    },
    UnexpectedToolRequest {
        index: u32,
    },
    #[serde(alias = "gauntlet_output_mismatch")]
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
