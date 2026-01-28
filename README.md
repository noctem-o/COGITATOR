```text
   ██████╗  ██████╗  ██████╗ ██╗████████╗ █████╗ ████████╗ ██████╗ ██████╗
  ██╔════╝ ██╔═══██╗██╔════╝ ██║╚══██╔══╝██╔══██╗╚══██╔══╝██╔═══██╗██╔══██╗
  ██║      ██║   ██║██║  ███╗██║   ██║   ███████║   ██║   ██║   ██║██████╔╝
  ██║      ██║   ██║██║   ██║██║   ██║   ██╔══██║   ██║   ██║   ██║██╔══██╗
  ╚██████╗ ╚██████╔╝╚██████╔╝██║   ██║   ██║  ██║   ██║   ╚██████╔╝██║  ██║
   ╚═════╝  ╚═════╝  ╚═════╝ ╚═╝   ╚═╝   ╚═╝  ╚═╝   ╚═╝    ╚═════╝ ╚═╝  ╚═╝

Cogitator is a deterministic evaluation harness with cryptographic witness roots that
make agent runs replayable, auditable, and verifiable. It captures full causal traces,
tracks entropy usage where applicable, and packages run artifacts so that third parties
can recompute the same witness root from the same inputs and environment.

## Table of contents

- [Quickstart](#quickstart)
- [Requirements](#requirements)
- [Install prerequisites (by OS)](#install-prerequisites-by-os)
- [Build and run](#build-and-run)
- [CLI overview](#cli-overview)
- [Output artifacts](#output-artifacts)
- [Feature list](#feature-list)
- [Commitment boundaries (witness vs provenance)](#commitment-boundaries-witness-vs-provenance)
- [Deterministic Simulation Testing (DST)-style fault injection](#deterministic-simulation-testing-dst-style-fault-injection)

## Quickstart

```bash
cargo build
./target/debug/cogitator run --seed 42 --runs 10 --out-dir out
```

## Requirements

- Rust toolchain (via `rustup`) with Cargo.
- A C/C++ compiler toolchain for building Rust dependencies.
- Git (optional but recommended for cloning).

## Install prerequisites (by OS)

### Linux (Debian/Ubuntu)

```bash
sudo apt-get update
sudo apt-get install -y build-essential curl git
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

### Linux (Fedora/RHEL)

```bash
sudo dnf install -y gcc gcc-c++ make curl git
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

### Linux (Arch)

```bash
sudo pacman -S --needed base-devel curl git
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

### macOS

```bash
xcode-select --install
brew install rustup git
rustup-init
```

### Windows

**Option A: Native Windows**

1. Install the [Visual Studio Build Tools](https://visualstudio.microsoft.com/visual-cpp-build-tools/).
2. Install Rust via [rustup](https://rustup.rs/).
3. Open a new PowerShell and verify:

```powershell
rustc --version
cargo --version
```

**Option B: WSL2 (recommended for a Linux-like workflow)**

```bash
sudo apt-get update
sudo apt-get install -y build-essential curl git
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

## Build and run

```bash
cargo build
./target/debug/cogitator --help
```

## CLI overview

### Run deterministic evaluations

```bash
./target/debug/cogitator run --seed 42 --runs 100 --out-dir out
```

### Run agent mode (with tool transcripts)

```bash
./target/debug/cogitator run --agent clawdbot --runs 1 --out-dir out
```

Agent-only flags such as `--threads` and `--fault-*` are rejected in non-agent runs.

## Output artifacts

Outputs include:

- `meta.json` – run metadata (witnessed + provenance)
- `trace.jsonl` – canonical trace events
- `results.csv` / `results.json` – case-level results
- `summary.json` – aggregate metrics
- `analysis.json` – bundled metadata + summary + results
- `witness_root.txt` – final witness root for the run
- `nix_provenance.json` – optional Nix metadata (provenance only)

A typical output layout looks like:

```
out/
├── analysis.json
├── meta.json
├── nix_provenance.json
├── results.csv
├── results.json
├── summary.json
├── trace.jsonl
├── witness_root.txt
└── run_0000/
    ├── agent_trace.json
    ├── chaos_profile.json
    ├── drift_report.json
    ├── hash_chain.txt
    ├── tool_transcript.json
    ├── witness_root.txt
    └── witness_manifest.json
```

## Feature list

- **Deterministic execution** with explicit entropy accounting (where applicable) and
  ordered trace emission.
- **Witness roots** (BLAKE3) that commit to every event in a run’s trace.
- **Deterministic agent mode** with tool transcript recording + replay for byte-stable
  re-execution.
- **Optional LLM-as-tool integration**: LLM inference is just another tool call. Live
  mode records responses into the transcript; replay reuses them. The default stub
  backend is fully offline and requires no model installation.
- **Drift detection** that compares replayed tool calls against recorded transcripts and
  emits machine-readable drift reports.
- **Witness bundles** that package agent traces, tool transcripts, hash chains, and
  manifests for offline verification workflows.
- **Hash-chain auditing** for agent traces + tool calls, separate from the global witness
  root.
- **Reproducible run metadata** capturing seed, run counts, parallel strategy, and
  provenance.
- **Artifact manifests** for programmatic consumption of outputs.
- **Deterministic Simulation Testing (DST)-style fault injection** for reproducible chaos
  testing, with fault schedules committed to the witness metadata.
- **Witness/provenance split** so runtime environment details stay out of witness
  commitments while remaining discoverable.
- **Canonical JSON artifacts** to keep audit artifacts byte-stable across runs.
- **Optional TUI** for inspecting run summaries and drift status (feature-flagged).

## Commitment boundaries (witness vs provenance)

- **Witness root** commits to canonical trace entries plus agent traces + tool call witness
  views in deterministic order (agent step, then tool calls by `tool_call_idx`). Simulated
  latency and runtime environment details are excluded.
- **Provenance metadata** captures run-time context (timestamps, toolchain versions, agent
  thread count, optional Nix details) and is **not** part of the witness root.
- **Bundle hash** covers all artifacts listed in the witness manifest (including optional
  provenance artifacts like `nix_provenance.json`) for offline verification.

## Deterministic Simulation Testing (DST)-style fault injection

Cogitator can deterministically inject tool faults (timeouts, corruptions, drops, and
latency simulations). Faults are driven by a seeded schedule and recorded in tool
transcripts so that record + replay is byte-stable. Simulated latency is exposed to the
agent but excluded from witness commitments by default. Fault selection uses a single
deterministic draw with cumulative per-million weights (first matching bucket wins).

Example:

```bash
./target/debug/cogitator run \
  --agent clawdbot \
  --case 0 \
  --faults on \
  --fault-profile stress \
  --fault-timeout-rate 0.01 \
  --fault-corrupt-rate 0.001 \
  --fault-drop-rate 0.001
```
