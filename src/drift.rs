use anyhow::{Context, Result};
use blake3::Hasher;
use serde::{Deserialize, Serialize};
use std::path::Path;

use crate::agent::AgentTraceEntry;
use crate::model::WitnessManifest;
use crate::tooling::{ToolCall, ToolTranscriptRecord};

pub const DRIFT_SCHEMA_VERSION: u32 = 1;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DriftReport {
    pub schema_version: u32,
    pub drifted: bool,
    pub issues: Vec<String>,
}

pub fn detect_transcript_drift(
    expected: &ToolTranscriptRecord,
    actual: &ToolTranscriptRecord,
) -> DriftReport {
    let mut issues = Vec::new();

    if expected.entries.len() != actual.entries.len() {
        issues.push(format!(
            "tool call count mismatch: expected {}, got {}",
            expected.entries.len(),
            actual.entries.len()
        ));
    }

    for (index, (exp, act)) in expected
        .entries
        .iter()
        .zip(actual.entries.iter())
        .enumerate()
    {
        if exp.step != act.step {
            issues.push(format!(
                "tool call step mismatch at {}: expected {}, got {}",
                index, exp.step, act.step
            ));
        }
        if exp.request != act.request {
            issues.push(format!("tool request mismatch at {}", index));
        }
        let exp_hash = response_hash(&exp.response);
        let act_hash = response_hash(&act.response);
        if exp_hash != act_hash {
            issues.push(format!("tool response hash mismatch at {}", index));
        }
    }

    DriftReport {
        schema_version: DRIFT_SCHEMA_VERSION,
        drifted: !issues.is_empty(),
        issues,
    }
}

pub fn build_hash_chain(
    agent_trace: &[AgentTraceEntry],
    tool_calls: &[ToolCall],
) -> Result<Vec<String>> {
    let mut chain = Vec::new();
    let mut current = initial_hash();

    for entry in agent_trace {
        let payload = serde_json::json!({
            "kind": "agent_trace",
            "payload": entry,
        });
        current = chained_hash(&current, &payload)?;
        chain.push(hex_string(&current));

        for call in tool_calls.iter().filter(|call| call.step == entry.step) {
            let payload = serde_json::json!({
                "kind": "tool_call",
                "payload": call,
            });
            current = chained_hash(&current, &payload)?;
            chain.push(hex_string(&current));
        }
    }

    Ok(chain)
}

pub fn verify_witness_bundle(dir: &Path) -> Result<()> {
    let manifest_path = dir.join("witness_manifest.json");
    let manifest_file = std::fs::File::open(&manifest_path)
        .with_context(|| "failed to open witness_manifest.json")?;
    let manifest: WitnessManifest = serde_json::from_reader(manifest_file)
        .with_context(|| "failed to parse witness_manifest.json")?;

    let agent_trace_file = std::fs::File::open(&manifest.agent_trace_json)
        .with_context(|| "failed to open agent_trace.json")?;
    let agent_trace: Vec<AgentTraceEntry> = serde_json::from_reader(agent_trace_file)
        .with_context(|| "failed to parse agent_trace.json")?;

    let tool_file = std::fs::File::open(&manifest.tool_transcript_json)
        .with_context(|| "failed to open tool_transcript.json")?;
    let tool_transcript: ToolTranscriptRecord = serde_json::from_reader(tool_file)
        .with_context(|| "failed to parse tool_transcript.json")?;

    let chain = build_hash_chain(&agent_trace, &tool_transcript.entries)?;
    let expected_chain = std::fs::read_to_string(&manifest.hash_chain_txt)
        .with_context(|| "failed to read hash_chain.txt")?;
    let expected_lines: Vec<&str> = expected_chain
        .lines()
        .map(|line| line.trim())
        .filter(|line| !line.is_empty())
        .collect();

    if chain.len() != expected_lines.len() {
        anyhow::bail!(
            "hash chain length mismatch: expected {}, got {}",
            expected_lines.len(),
            chain.len()
        );
    }

    for (idx, (expected, actual)) in expected_lines.iter().zip(chain.iter()).enumerate() {
        if expected != actual {
            anyhow::bail!(
                "hash chain mismatch at line {}: expected {}, got {}",
                idx + 1,
                expected,
                actual
            );
        }
    }

    Ok(())
}

fn response_hash(response: &crate::tooling::ToolResponse) -> String {
    let payload = serde_json::json!({
        "tool_name": response.tool_name,
        "output": response.output,
        "success": response.success,
    });
    let mut hasher = Hasher::new();
    if let Ok(bytes) = serde_json::to_vec(&payload) {
        hasher.update(&bytes);
    }
    hex_string(hasher.finalize().as_bytes())
}

fn initial_hash() -> [u8; 32] {
    let mut hasher = Hasher::new();
    hasher.update(b"COGITATOR_WITNESS_CHAIN");
    *hasher.finalize().as_bytes()
}

fn chained_hash(previous: &[u8; 32], payload: &serde_json::Value) -> Result<[u8; 32]> {
    let mut hasher = Hasher::new();
    hasher.update(previous);
    let bytes = serde_json::to_vec(payload).context("serialize hash payload")?;
    hasher.update(&bytes);
    Ok(*hasher.finalize().as_bytes())
}

fn hex_string(bytes: &[u8]) -> String {
    let mut out = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        out.push_str(&format!("{:02x}", byte));
    }
    out
}
