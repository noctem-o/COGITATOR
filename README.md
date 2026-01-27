# COGITATOR — Deterministic Evaluation Harness for Agents


```text
   ██████╗  ██████╗  ██████╗ ██╗████████╗ █████╗ ████████╗ ██████╗ ██████╗
  ██╔════╝ ██╔═══██╗██╔════╝ ██║╚══██╔══╝██╔══██╗╚══██╔══╝██╔═══██╗██╔══██╗
  ██║      ██║   ██║██║  ███╗██║   ██║   ███████║   ██║   ██║   ██║██████╔╝
  ██║      ██║   ██║██║   ██║██║   ██║   ██╔══██║   ██║   ██║   ██║██╔══██╗
  ╚██████╗ ╚██████╔╝╚██████╔╝██║   ██║   ██║  ██║   ██║   ╚██████╔╝██║  ██║
   ╚═════╝  ╚═════╝  ╚═════╝ ╚═╝   ╚═╝   ╚═╝  ╚═╝   ╚═╝    ╚═════╝ ╚═╝  ╚═╝


Cogitator is a deterministic evaluation harness for agent-style workflows. It runs seeded evaluations, writes results to CSV, and can print a compact terminal UI that summarizes run health, telemetry, and hardware context.

## Features

- Deterministic case generation and scoring from a seed + run id.
- CSV output for downstream analysis.
- Optional terminal UI with:
  - High‑level, non‑sensitive thought telemetry.
  - Aggregate metrics (range, median/P90, volatility, correlation, entropy).
  - Hardware snapshot (CPU, memory, OS, accelerator hints).
  - Local config signals (NixOS, Home Manager, Hyprland) when present.

## Usage

Build and run:

```bash
cargo run --release -- --seed 42 --runs 5000 --output results.csv
```

Disable the terminal UI (e.g., for scripted runs):

```bash
cargo run --release -- --no-tui
```

## CLI Options

- `--seed <u64>`: Seed for deterministic evaluation (default: `42`).
- `--runs <u32>`: Number of evaluation runs (default: `5000`).
- `--output <path>`: CSV output path (default: `results.csv`).
- `--no-tui`: Disable terminal UI output.

## Output

The CSV contains the following columns:

- `run_id`
- `case_id`
- `difficulty`
- `score`
- `passed`

The terminal UI prints a summary table and supporting telemetry; it avoids emitting sensitive chain-of-thought and stays at a high level.

## Notes

- Hardware detection is best‑effort and intentionally lightweight to preserve portability.
- Config signal checks only report file existence and size, not contents.

## License

MIT / Apache-2.0
