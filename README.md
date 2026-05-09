# Cogitator

[![CI](https://img.shields.io/github/actions/workflow/status/noctem-o/cogitator/ci.yml?branch=main&label=CI&style=flat-square)](https://github.com/noctem-o/cogitator/actions/workflows/ci.yml)
[![License: Apache-2.0](https://img.shields.io/badge/License-Apache--2.0-blue.svg?style=flat-square)](LICENSE)
[![Rust](https://img.shields.io/badge/Rust-stable-orange.svg?style=flat-square)](https://www.rust-lang.org/)
[![Release](https://img.shields.io/github/v/release/noctem-o/cogitator?style=flat-square)](https://github.com/noctem-o/cogitator/releases)
[![Spec](https://img.shields.io/badge/Spec-Apache--2.0-blue?style=flat-square)](spec/COGITATOR_WITNESS_PROTOCOL.md)

Cogitator is a Rust harness for producing tamper-evident records of AI-agent execution.

It exists to make agent runs independently auditable: each run produces a witness bundle with a recomputable root, plus enough replay data to validate what happened and under what policy constraints.

## What it is

- Tamper-evident witness bundles with BLAKE3 witness roots.
- Canonical JSON commitments (RFC 8785/JCS-style target) for stable verification.
- Deterministic replay and verify workflows.
- Pre-call policy interception with explicit allow/block/phantom outcomes.
- Phantom entries and policy digests committed into witness data.
- Drift demos and CI checks for witness-root stability.

## What it is not

- Not a general-purpose agent framework.
- Not a compliance product by itself.
- Not proof that a run originally occurred unless the root is externally anchored.
- Not a substitute for legal, regulatory, or security review.

## Quick start

```bash
cargo build --release

# Run a deterministic drift demo
cargo run --release -- demo drift --seed 42 --threads 1 --fault-profile stress --out-dir demo_out --clean

# Verify the baseline bundle
cargo run --release -- verify --witness demo_out/drift/baseline_faults

# Semantic recompute check
cargo run --release -- verify --witness demo_out/drift/baseline_faults --recompute-witness-root
```

## Verify a bundle

```bash
# Record a run
cargo run --release -- run --agent ordeal --runs 1 --out-dir out --clean

# Verify witness/manifest consistency
cargo run --release -- verify --witness out/run_0000

# Verify by recomputing committed components
cargo run --release -- verify --witness out/run_0000 --recompute-witness-root
```

## Policy interception

```toml
schema_version = 1

[[rules]]
id = "trade-budget"
tool_pattern = "trade.*"
history_tool_pattern = "trade.*"
history_max_calls = 2
verdict = "block"
reason = "trade call budget exceeded"

[[rules]]
id = "research-phantom"
tool_pattern = "research.**"
verdict = "phantom"
reason = "observe only"
```

Rules are evaluated top-to-bottom; first match wins.

## Artifact layout

```text
out/
├── meta.json
├── trace.jsonl
├── results.csv / results.json
├── summary.json
├── analysis.json
├── witness_root.txt
└── run_0000/
    ├── agent_trace.json
    ├── tool_transcript.json
    ├── chaos_profile.json
    ├── drift_report.json
    ├── hash_chain.txt
    ├── meta.json
    ├── witness_manifest.json
    ├── witness_root.txt
    └── verify_report.json
```

## Design notes

- **Witness/provenance split:** provenance metadata is recorded but not committed into the witness root.
- **Deterministic replay:** same inputs, seed, policy, and profile should reproduce the same root.
- **Canonicalization target:** witness commitments rely on canonical JSON for stable byte-level hashing.
- **Phantom entries:** blocked/phantomed calls are still committed as auditable intent records.

## Protocol specification

The protocol is documented in [`spec/COGITATOR_WITNESS_PROTOCOL.md`](spec/COGITATOR_WITNESS_PROTOCOL.md). The implementation and the protocol spec are both Apache-2.0.

## Release and provenance

- Release artifacts are produced with `cargo-dist` via GitHub Actions.
- GitHub artifact attestations are emitted in the release workflow.
- CI includes format/clippy/test gates plus deterministic replay and verify checks.
- The repo includes a no-git build gate for release reproducibility hygiene.

## Development

```bash
cargo fmt --check
cargo clippy --locked --all-targets -- -D warnings
cargo test --locked
```

## Nix

A Nix dev shell is available:

```bash
nix develop
```

Nix details are provenance-only and do not alter witness roots.

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md).

## License

Apache-2.0.
