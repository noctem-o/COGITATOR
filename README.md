```text
   ██████╗  ██████╗  ██████╗ ██╗████████╗ █████╗ ████████╗ ██████╗ ██████╗ 
  ██╔════╝ ██╔═══██╗██╔════╝ ██║╚══██╔══╝██╔══██╗╚══██╔══╝██╔═══██╗██╔══██╗
  ██║      ██║   ██║██║  ███╗██║   ██║   ███████║   ██║   ██║   ██║██████╔╝
  ██║      ██║   ██║██║   ██║██║   ██║   ██╔══██║   ██║   ██║   ██║██╔══██╗
  ╚██████╗ ╚██████╔╝╚██████╔╝██║   ██║   ██║  ██║   ██║   ╚██████╔╝██║  ██║
   ╚═════╝  ╚═════╝  ╚═════╝ ╚═╝   ╚═╝   ╚═╝  ╚═╝   ╚═╝    ╚═════╝ ╚═╝  ╚═╝


Cogitator is a deterministic evaluation harness with cryptographic witness roots that make
agent runs replayable, auditable, and verifiable. Cogitator makes agent behavior
reproducible the way git makes code reproducible. It captures full causal traces, tracks
entropy usage where applicable, and packages run artifacts so that third parties can
recompute the same witness root from the same inputs and environment.

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

## Feature list

- **Deterministic execution** with explicit entropy accounting (where applicable) and ordered
  trace emission.
- **Witness roots** (BLAKE3) that commit to every event in a run’s trace.
- **Deterministic agent mode** with tool transcript recording + replay for byte-stable
  re-execution.
- **Optional LLM-as-tool integration**: LLM inference is just another tool call. Live mode
  records responses into the transcript; replay reuses them. The default stub backend is
  fully offline and requires no model installation.
- **Drift detection** that compares replayed tool calls against recorded transcripts and
  emits machine-readable drift reports.
- **Witness bundles** that package agent traces, tool transcripts, hash chains, and
  manifests for offline verification workflows.
- **Hash-chain auditing** for agent traces + tool calls, separate from the global witness
  root.
- **Reproducible run metadata** capturing seed, run counts, parallel strategy, and provenance.
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
./target/debug/cogitator run \\
  --agent clawdbot \\
  --case 0 \\
  --faults on \\
  --fault-profile stress \\
  --fault-timeout-rate 0.01 \\
  --fault-corrupt-rate 0.001 \\
  --fault-drop-rate 0.001
```

## CLI overview

Build and run:

```bash
cargo build
./target/debug/cogitator --help
```

### Run deterministic evaluations

```bash
./target/debug/cogitator run --seed 42 --runs 100 --out-dir out
```

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

### Run agent mode (with tool transcripts)

```bash
./target/debug/cogitator run --agent clawdbot --runs 1 --out-dir out
```

Agent-only flags such as `--threads` and `--fault-*` are rejected in non-agent runs.

Agent-mode produces a per-run directory (`out/run_0000/`) with:

- `agent_trace.json` – agent decisions per step
- `tool_transcript.json` – tool calls and deterministic stub outputs
- `hash_chain.txt` – chained hashes over agent traces + tool calls
- `drift_report.json` – drift status and mismatches
- `witness_manifest.json` – pointers to all per-run artifacts
- `chaos_profile.json` – fault schedule declaration and rates
- `witness_root.txt` – witness root for the agent run
- `nix_provenance.json` – optional Nix metadata (provenance only)

The witness root commits to the run globally; the per-run hash chain provides local,
step-by-step provenance for drift analysis.

### Replay an agent run

```bash
./target/debug/cogitator run --agent clawdbot --case 0 --replay out/run_0000 --out-dir replay
```

Replay reuses the prior tool transcript and emits a drift report showing any deviations.
Replay is transcript playback only; deterministic chaos is not re-run during replay.

### Optional LLM tool (stubbed by default)

LLM calls are treated as tool invocations so their responses are recorded during live
runs and replayed verbatim during replays. By default, Cogitator uses a deterministic
stub backend and requires no model installation.

```bash
./target/debug/cogitator run --agent clawdbot --runs 1 --llm on --llm-model stub --out-dir out
```

### Verify witness roots

Verify a trace against an expected witness root:

```bash
./target/debug/cogitator verify --meta out/meta.json --trace out/trace.jsonl --expect <root>
```

Verify a witness bundle (agent mode):

```bash
./target/debug/cogitator verify --witness out/run_0000
```

Verification emits `verify_report.json` alongside the bundle with artifact hashes,
bundle hash recomputation, and (when possible) witness root verification. Tool transcripts
and witness bundles are fully verifiable offline once recorded.

### Drift demo (baseline vs regressed + faults)

```bash
./target/debug/cogitator demo drift --fault-profile stress --threads 4
```

This produces baseline/regressed pairs with and without deterministic fault injection,
showing how drift can be detected under DST-style chaos. The regressed scenarios should
report `drifted=true` because the agent emits a different tool request, and the
fault-injected regressed run will diverge under the deterministic chaos schedule.

## Agent threads (provenance-only)

Use `--threads` to control the Rayon pool in agent mode. The thread count is captured in
provenance but never enters the witness root.

```bash
./target/debug/cogitator run --agent clawdbot --case 0 --threads 4 --out-dir out
```

## Nix provenance (optional)

Use `--nix-provenance {auto,on,off}` to capture Nix/NixOS metadata without impacting
portability. In `auto`, capture is attempted when `NIX_STORE` is set or `nix --version`
works; `on` requires Nix to be available; `off` skips capture. When available,
`nix_provenance.json` is emitted and included in witness bundles (bundle hash only, not
the witness root).

## Nix workflow (offline-friendly)

For a pinned Rust toolchain and reproducible `created_at`, use the included `shell.nix`:

```bash
nix-shell --pure --run "cargo fmt && cargo test"
```

`shell.nix` sets `SOURCE_DATE_EPOCH` to keep `created_at` stable. For fully offline
builds, keep `Cargo.lock` checked in and optionally run `cargo vendor` to prefetch
dependencies into the repo.

## TUI support

The TUI is feature-gated. Enable it with:

```bash
cargo run --features tui -- run --runs 10
```

Use `--no-tui` to suppress the interface when running in CI or headless contexts.

## Project layout

- `src/main.rs` – CLI entrypoint and artifact orchestration
- `src/eval.rs` – deterministic evaluation harness
- `src/witness.rs` – witness root builder
- `src/verify.rs` – trace verification
- `src/agent.rs` – example deterministic agent implementation
- `src/tooling.rs` – tool transcript recording/replay
- `src/drift.rs` – drift detection + witness bundle verification
- `src/tui.rs` – terminal UI (feature gated)

## License

MIT
