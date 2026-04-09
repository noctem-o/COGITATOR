# COGITATOR Witness Protocol

**Version:** 1.0  
**Status:** Draft  
**Date:** 2026-04-09  
**Author:** noctem-o  
**Licence:** Apache License 2.0 (see footer)

---

## Abstract

This document specifies the COGITATOR Witness Protocol -- a scheme for producing cryptographically verifiable, tamper-evident records of AI agent execution. A conforming implementation produces a single witness root value that any third party can independently recompute from the same inputs to verify the record was not altered after the fact.

The protocol is implementation-agnostic. The reference implementation is COGITATOR (Rust), but any language or framework can produce conforming witness bundles.

---

## Motivation

AI agents deployed in regulated or high-stakes environments issue tool calls with real-world consequences. Existing audit approaches -- log ingestion, post-hoc summaries, model cards -- are reconstructive. They describe what probably happened, not what provably happened. Logs are mutable. Post-hoc summaries are interpretations.

The COGITATOR Witness Protocol takes the position that agent execution should be as auditable as a compiled binary in a reproducible build system. The witness root is the runtime equivalent of a content-addressed store path: a cryptographic commitment that ties a specific output to a specific, verifiable execution.

---

## Definitions

| Term | Definition |
|---|---|
| **Run** | A single end-to-end execution of an agent against a set of inputs |
| **Tool call** | A discrete action dispatched by the agent to an external or internal tool |
| **Phantom entry** | A record of a tool call that was intercepted and blocked before dispatch |
| **Witness root** | A single BLAKE3 hex digest committing the entire run record |
| **Policy digest** | A SHA-256 hex digest of the policy file in effect during the run |
| **Canonical JSON** | JSON serialised according to RFC 8785 (deterministic key ordering, no insignificant whitespace) |
| **Witness bundle** | The complete set of artefact files for a single run |

---

## Witness Bundle Structure

A conforming witness bundle MUST contain the following files:

```
run_<id>/
+-- agent_trace.json        # Agent steps: inputs, tool requests, outputs
+-- tool_transcript.json    # All tool calls (real and phantom) with outcomes
+-- chaos_profile.json      # Fault injection schedule (may be empty)
+-- drift_report.json       # Replay mismatch report (empty if no drift)
+-- hash_chain.txt          # Per-call BLAKE3 hashes, one per line
+-- meta.json               # Witnessed metadata
+-- witness_manifest.json   # Per-file hashes and bundle hash
+-- witness_root.txt        # Single hex string -- the tamper-evident root
```

All JSON files MUST be serialised as RFC 8785 canonical JSON before hashing.

---

## Schema Definitions

### meta.json

```json
{
  "schema_version": 4,
  "run_id": "<string>",
  "agent_id": "<string>",
  "seed": "<uint64>",
  "policy_digest": "<sha256-hex | null>",
  "started_at": "<ISO 8601 datetime>",
  "finished_at": "<ISO 8601 datetime>",
  "cogitator_version": "<semver string>"
}
```

- `schema_version` MUST be `4` for this version of the protocol.
- `policy_digest` MUST be the SHA-256 hex digest of the policy file bytes, or `null` if no policy was in effect.
- `seed` MUST be the fixed random seed used for the run, enabling deterministic replay.

### tool_transcript.json

```json
{
  "schema_version": 4,
  "entries": [ /* ToolCall[] */ ],
  "phantom_entries": [ /* PhantomEntry[] */ ],
  "policy_digest": "<sha256-hex | null>"
}
```

#### ToolCall object

```json
{
  "step": "<uint>",
  "tool_call_idx": "<uint>",
  "tool_name": "<string>",
  "request": { /* arbitrary JSON */ },
  "response": { /* arbitrary JSON */ },
  "chaos_fault": "<string | null>",
  "call_hash": "<blake3-hex>"
}
```

- `call_hash` MUST be the BLAKE3 digest of the RFC 8785 canonical JSON of this object (with `call_hash` set to the empty string before hashing).

#### PhantomEntry object

```json
{
  "step": "<uint>",
  "tool_call_idx": "<uint>",
  "tool_name": "<string>",
  "request": { /* arbitrary JSON */ },
  "disposition": "Blocked | Phantom",
  "rule_id": "<string | null>",
  "reason": "<string | null>",
  "entry_hash": "<blake3-hex>"
}
```

- `entry_hash` MUST be the BLAKE3 digest of the RFC 8785 canonical JSON of this object (with `entry_hash` set to the empty string before hashing).
- `disposition` MUST be one of `Blocked` (tool call explicitly denied by policy) or `Phantom` (tool call silently observed but not dispatched).

### hash_chain.txt

A newline-delimited text file. Each line is the `call_hash` or `entry_hash` of one record in the order they were produced, interleaving real calls and phantom entries chronologically.

```
<blake3-hex>\n
<blake3-hex>\n
...
```

### witness_manifest.json

```json
{
  "files": {
    "agent_trace.json": "<blake3-hex>",
    "tool_transcript.json": "<blake3-hex>",
    "chaos_profile.json": "<blake3-hex>",
    "drift_report.json": "<blake3-hex>",
    "hash_chain.txt": "<blake3-hex>",
    "meta.json": "<blake3-hex>"
  },
  "bundle_hash": "<blake3-hex>"
}
```

- Each file hash is the BLAKE3 digest of the raw file bytes.
- `bundle_hash` is the BLAKE3 digest of the RFC 8785 canonical JSON of the `files` object.

### witness_root.txt

A single line containing the BLAKE3 hex digest of the RFC 8785 canonical JSON of the complete `witness_manifest.json` object (including `bundle_hash`).

```
<blake3-hex>\n
```

This is the only value that needs to be published for a third party to verify the entire bundle.

---

## Witness Root Computation

The witness root is computed as follows:

```
1. Serialise each bundle file to RFC 8785 canonical JSON (or raw bytes for text files).
2. Compute BLAKE3(file_bytes) for each file -> files map.
3. Compute BLAKE3(RFC8785(files)) -> bundle_hash.
4. Serialise witness_manifest.json including bundle_hash.
5. Compute BLAKE3(RFC8785(witness_manifest)) -> witness_root.
6. Write witness_root as a single lowercase hex string to witness_root.txt.
```

A verifier replays steps 1-6 from the bundle files and asserts the computed root matches the published `witness_root.txt`.

---

## Policy Protocol

The policy layer is optional. If no policy file is provided, all tool calls are implicitly allowed and `policy_digest` MUST be `null` in both `meta.json` and `tool_transcript.json`.

When a policy file is provided:

1. The policy file bytes MUST be SHA-256 digested before any run begins.
2. The digest MUST be committed to `meta.json` and `tool_transcript.json` before the first tool call.
3. Every tool call MUST be evaluated against the policy before dispatch.
4. Blocked or phantomed calls MUST produce a `PhantomEntry` committed to the witness chain.
5. The `CallHistory` (cumulative record of all verdicts in the current run) MUST be updated after every verdict, including blocked calls.

### Policy Verdicts

| Verdict | Tool dispatched? | Agent receives | Chain entry |
|---|---|---|---|
| `allow` | Yes | Real tool response | `ToolCall` |
| `block` | No | `{ "blocked": true, "reason": "<string>" }` | `PhantomEntry(Blocked)` |
| `phantom` | No | `{ "blocked": true, "reason": "<string>" }` | `PhantomEntry(Phantom)` |

---

## Deterministic Replay

A conforming implementation MUST support replay mode. Given:

- The original `agent_trace.json` (inputs)
- The original `chaos_profile.json` (fault schedule)
- The original policy file (identified by `policy_digest`)
- The original `seed`

A replay run MUST produce an identical `witness_root` to the original run. Any deviation MUST be reported as a `DriftIssue` in `drift_report.json`.

---

## Conformance

An implementation is conforming if:

1. It produces all required bundle files.
2. All JSON files are RFC 8785 canonical before hashing.
3. The witness root computation follows the algorithm above exactly.
4. Phantom entries are produced for all blocked and phantomed calls.
5. The policy digest is committed before the first tool call when a policy is in effect.
6. Replay of the same inputs produces an identical witness root.

---

## Relation to Regulatory Frameworks

The COGITATOR Witness Protocol is designed to satisfy the technical requirements of:

- **EU AI Act (2024) Articles 12 and 9** -- tamper-evident record-keeping and risk management systems for high-risk AI.
- **FCA AI and machine learning guidance** -- audit trails for automated decision systems in financial services.
- **NIST AI RMF (2023)** -- traceability and accountability requirements for AI systems.

The witness root provides a single publishable value that demonstrates a run record has not been altered. The policy digest demonstrates the constraints in effect at time of execution. The phantom entries demonstrate what the agent attempted but was not permitted to do.

---

## Versioning

This document describes protocol version 1.0, corresponding to `schema_version: 4` in bundle files. Breaking changes to the schema or hash computation algorithm will increment the protocol version and the schema version together.

---

## Licence

Copyright 2026 noctem-o

Licensed under the Apache License, Version 2.0 (the "License"); you may not use this file except in compliance with the License. You may obtain a copy of the License at:

    http://www.apache.org/licenses/LICENSE-2.0

Unless required by applicable law or agreed to in writing, software distributed under the License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied. See the License for the specific language governing permissions and limitations under the License.

Note: This specification document is licenced under Apache 2.0. The COGITATOR reference implementation is licenced separately under the Business Source License 1.1.
