#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum DriftIssue {
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
    ToolOutcomeMismatch {
        index: u32,
    },
    ToolCallCountMismatch {
        expected: u32,
        actual: u32,
    },
    UnexpectedToolRequest {
        index: u32,
    },
    // Add any other variants your tests reference
}
