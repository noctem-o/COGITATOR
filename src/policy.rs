//! Policy engine for pre-call tool-call interception.
//!
//! Loads a TOML policy file and evaluates each incoming `ToolRequest` before
//! dispatch.  Returns one of three verdicts:
//!
//! - `Allow`   — execute normally, record as a real `ToolCall`
//! - `Block`   — do not execute; record as a `PhantomEntry` with `blocked` disposition
//! - `Phantom` — do not execute; record as a `PhantomEntry` with `phantom` disposition
//!               (semantically: the agent tried, the harness observed, no side-effect)
//!
//! The policy file path is embedded as a SHA-256 digest into `WitnessedMetadata`
//! so the exact policy version is part of the witness root.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::path::Path;

use crate::tooling::ToolRequest;

// ─── Schema ────────────────────────────────────────────────────────────────

pub const POLICY_SCHEMA_VERSION: u32 = 1;

/// The verdict returned by `PolicyEngine::evaluate`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PolicyVerdict {
    /// Execute the tool call normally.
    Allow,
    /// Block the call.  Record a phantom entry with disposition `blocked`.
    Block,
    /// Phantom execution: observe the intent but produce no side-effect.
    /// Record a phantom entry with disposition `phantom`.
    Phantom,
}

/// Disposition stored in a `PhantomEntry`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PhantomDisposition {
    Blocked,
    Phantom,
}

// ─── Rule model ────────────────────────────────────────────────────────────

/// A single policy rule.  Rules are evaluated top-to-bottom; the first match wins.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PolicyRule {
    /// Human-readable identifier for this rule (appears in phantom entries).
    pub id: String,

    /// Glob-style pattern matched against `tool_name`.
    /// `*` matches any sequence of non-`.` characters.
    /// `**` matches any sequence including `.`.
    /// Example: `"trade.*"` matches `trade.buy` and `trade.sell`.
    pub tool_pattern: String,

    /// Optional: block if the agent has already called any tool matching
    /// `history_tool_pattern` more than `history_max_calls` times this run.
    #[serde(default)]
    pub history_tool_pattern: Option<String>,

    #[serde(default)]
    pub history_max_calls: Option<usize>,

    /// What to do when this rule matches.
    pub verdict: PolicyVerdict,

    /// Human-readable reason embedded in the phantom entry.
    pub reason: String,
}

/// The top-level policy document (deserialised from TOML).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PolicyDocument {
    pub schema_version: u32,
    #[serde(default)]
    pub rules: Vec<PolicyRule>,
}

impl Default for PolicyDocument {
    fn default() -> Self {
        Self {
            schema_version: POLICY_SCHEMA_VERSION,
            rules: Vec::new(),
        }
    }
}

// ─── Call history ───────────────────────────────────────────────────────────

/// Rolling record of tool calls already dispatched or phantomed this run.
/// Passed into every policy evaluation so rules can reason about sequences.
#[derive(Debug, Clone, Default)]
pub struct CallHistory {
    entries: Vec<HistoryEntry>,
}

#[derive(Debug, Clone)]
struct HistoryEntry {
    tool_name: String,
    verdict: PolicyVerdict,
}

impl CallHistory {
    pub fn new() -> Self {
        Self::default()
    }

    /// Record an outcome (called after verdict is decided).
    pub fn record(&mut self, tool_name: &str, verdict: PolicyVerdict) {
        self.entries.push(HistoryEntry {
            tool_name: tool_name.to_string(),
            verdict,
        });
    }

    /// Count how many times tools matching `pattern` have been called
    /// (regardless of verdict) this run.
    pub fn count_matching(&self, pattern: &str) -> usize {
        self.entries
            .iter()
            .filter(|e| tool_name_matches(pattern, &e.tool_name))
            .count()
    }
}

// ─── Engine ─────────────────────────────────────────────────────────────────

/// Loaded policy, ready to evaluate requests.
pub struct PolicyEngine {
    pub document: PolicyDocument,
    /// SHA-256 hex digest of the raw policy file bytes (empty string if no file).
    pub digest: String,
}

impl PolicyEngine {
    /// Load from a TOML file.  Returns an allow-all engine if the file does
    /// not exist (so the harness works without a policy file).
    pub fn load(path: &Path) -> Result<Self> {
        if !path.exists() {
            return Ok(Self::allow_all());
        }

        let raw = std::fs::read(path)
            .with_context(|| format!("failed to read policy file: {}", path.display()))?;

        let digest = {
            let mut h = Sha256::new();
            h.update(&raw);
            crate::hex::encode(&h.finalize())
        };

        let text = std::str::from_utf8(&raw)
            .with_context(|| format!("policy file is not valid UTF-8: {}", path.display()))?;

        let document: PolicyDocument = toml::from_str(text)
            .with_context(|| format!("failed to parse policy file: {}", path.display()))?;

        Ok(Self { document, digest })
    }

    /// An engine that allows everything (used when no policy file is present).
    pub fn allow_all() -> Self {
        Self {
            document: PolicyDocument::default(),
            digest: String::new(),
        }
    }

    /// Evaluate a request against the policy rules.
    ///
    /// Returns `(verdict, matched_rule_id, reason)`.
    pub fn evaluate(
        &self,
        request: &ToolRequest,
        history: &CallHistory,
    ) -> (PolicyVerdict, Option<String>, String) {
        for rule in &self.document.rules {
            if !tool_name_matches(&rule.tool_pattern, &request.tool_name) {
                continue;
            }

            // History guard: check if the agent has exceeded a call budget.
            if let (Some(hist_pattern), Some(max)) =
                (&rule.history_tool_pattern, rule.history_max_calls)
            {
                let count = history.count_matching(hist_pattern);
                if count <= max {
                    // Under budget — this history guard doesn't trigger.
                    continue;
                }
            }

            return (
                rule.verdict.clone(),
                Some(rule.id.clone()),
                rule.reason.clone(),
            );
        }

        (PolicyVerdict::Allow, None, String::new())
    }
}

// ─── Glob matching ──────────────────────────────────────────────────────────

/// Minimal glob matcher for tool name patterns.
/// `**` matches anything (including `.`).  `*` matches non-`.` sequences.
fn tool_name_matches(pattern: &str, tool_name: &str) -> bool {
    glob_match(pattern, tool_name)
}

fn glob_match(pattern: &str, input: &str) -> bool {
    let pattern: Vec<char> = pattern.chars().collect();
    let input: Vec<char> = input.chars().collect();
    glob_match_chars(&pattern, &input)
}

fn glob_match_chars(pat: &[char], inp: &[char]) -> bool {
    match (pat, inp) {
        ([], []) => true,
        ([], _) => false,
        ([.., '*'], _) if pat.len() >= 2 && pat[pat.len() - 2] == '*' => {
            // trailing ** — matches remainder
            glob_match_chars(&pat[..pat.len() - 2], inp)
                || (!inp.is_empty() && glob_match_chars(pat, &inp[1..]))
        }
        (['*', '*', rest @ ..], _) => {
            // ** matches zero or more chars (including .)
            glob_match_chars(rest, inp)
                || (!inp.is_empty() && glob_match_chars(pat, &inp[1..]))
        }
        (['*', rest @ ..], _) => {
            // * matches zero or more non-. chars
            glob_match_chars(rest, inp)
                || (!inp.is_empty() && inp[0] != '.' && glob_match_chars(pat, &inp[1..]))
        }
        ([p, pr @ ..], [i, ir @ ..]) if p == i => glob_match_chars(pr, ir),
        _ => false,
    }
}

// ─── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tooling::ToolRequest;

    fn req(name: &str) -> ToolRequest {
        ToolRequest {
            tool_name: name.to_string(),
            arguments: serde_json::Value::Null,
        }
    }

    #[test]
    fn allow_all_engine_permits_everything() {
        let engine = PolicyEngine::allow_all();
        let history = CallHistory::new();
        let (verdict, rule_id, _) = engine.evaluate(&req("trade.buy"), &history);
        assert_eq!(verdict, PolicyVerdict::Allow);
        assert!(rule_id.is_none());
    }

    #[test]
    fn block_rule_matches_tool_pattern() {
        let engine = PolicyEngine {
            document: PolicyDocument {
                schema_version: 1,
                rules: vec![PolicyRule {
                    id: "no-trade".to_string(),
                    tool_pattern: "trade.*".to_string(),
                    history_tool_pattern: None,
                    history_max_calls: None,
                    verdict: PolicyVerdict::Block,
                    reason: "trading disabled".to_string(),
                }],
            },
            digest: String::new(),
        };
        let history = CallHistory::new();
        let (verdict, rule_id, reason) = engine.evaluate(&req("trade.buy"), &history);
        assert_eq!(verdict, PolicyVerdict::Block);
        assert_eq!(rule_id.as_deref(), Some("no-trade"));
        assert_eq!(reason, "trading disabled");
    }

    #[test]
    fn wildcard_does_not_cross_dot_boundary() {
        assert!(tool_name_matches("trade.*", "trade.buy"));
        assert!(!tool_name_matches("trade.*", "trade.buy.v2"));
        assert!(tool_name_matches("trade.**", "trade.buy.v2"));
    }

    #[test]
    fn history_guard_triggers_after_budget_exceeded() {
        let engine = PolicyEngine {
            document: PolicyDocument {
                schema_version: 1,
                rules: vec![PolicyRule {
                    id: "trade-limit".to_string(),
                    tool_pattern: "trade.*".to_string(),
                    history_tool_pattern: Some("trade.*".to_string()),
                    history_max_calls: Some(2),
                    verdict: PolicyVerdict::Block,
                    reason: "exceeded trade budget".to_string(),
                }],
            },
            digest: String::new(),
        };

        let mut history = CallHistory::new();
        // Under budget: 2 calls, max=2 — rule does NOT fire (count <= max)
        history.record("trade.buy", PolicyVerdict::Allow);
        history.record("trade.sell", PolicyVerdict::Allow);
        let (verdict, _, _) = engine.evaluate(&req("trade.buy"), &history);
        assert_eq!(verdict, PolicyVerdict::Allow);

        // Over budget: 3 calls, max=2 — rule fires (count > max)
        history.record("trade.buy", PolicyVerdict::Allow);
        let (verdict, rule_id, _) = engine.evaluate(&req("trade.buy"), &history);
        assert_eq!(verdict, PolicyVerdict::Block);
        assert_eq!(rule_id.as_deref(), Some("trade-limit"));
    }

    #[test]
    fn phantom_verdict_recorded() {
        let engine = PolicyEngine {
            document: PolicyDocument {
                schema_version: 1,
                rules: vec![PolicyRule {
                    id: "observe-only".to_string(),
                    tool_pattern: "research.**".to_string(),
                    history_tool_pattern: None,
                    history_max_calls: None,
                    verdict: PolicyVerdict::Phantom,
                    reason: "research tools are phantom-only".to_string(),
                }],
            },
            digest: String::new(),
        };
        let history = CallHistory::new();
        let (verdict, _, _) = engine.evaluate(&req("research.fetch"), &history);
        assert_eq!(verdict, PolicyVerdict::Phantom);
    }
}
