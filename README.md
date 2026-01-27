```text
   ██████╗  ██████╗  ██████╗ ██╗████████╗ █████╗ ████████╗ ██████╗ ██████╗
  ██╔════╝ ██╔═══██╗██╔════╝ ██║╚══██╔══╝██╔══██╗╚══██╔══╝██╔═══██╗██╔══██╗
  ██║      ██║   ██║██║  ███╗██║   ██║   ███████║   ██║   ██║   ██║██████╔╝
  ██║      ██║   ██║██║   ██║██║   ██║   ██╔══██║   ██║   ██║   ██║██╔══██╗
  ╚██████╗ ╚██████╔╝╚██████╔╝██║   ██║   ██║  ██║   ██║   ╚██████╔╝██║  ██║
   ╚═════╝  ╚═════╝  ╚═════╝ ╚═╝   ╚═╝   ╚═╝  ╚═╝   ╚═╝    ╚═════╝ ╚═╝  ╚═╝


Cogitator is a reproducible, deterministic evaluation harness designed for agent testing,
parallel execution, and trace-level inspection.

It provides:

- Seeded deterministic evaluation runs
- Parallel execution with Rayon
- CSV output for analysis pipelines
- Optional interactive TUI inspection cockpit (ratatui)

---

## Usage

### Run sequentially

```bash
cargo run -- --runs 1000
```

### Run in parallel

```bash
cargo run -- --runs 5000 --parallel true
```

### Enable the full TUI cockpit

```bash
cargo run --features tui -- --runs 2000
```

Keybinds:

* ↑ ↓ navigate runs
* f filter (ALL/PASS/FAIL)
* s sort (RUN/SCORE/DIFF)
* j k scroll thoughts
* q quit

---

## Output

Results are written to:

```
results.csv
```

Columns:

* run_id
* case_id
* difficulty
* score
* passed

---

## Vision

Cogitator is designed to evolve into a HAL-style agent harness with:

* full tool-call trace auditing
* adversarial evaluation suites
* multi-node orchestration
* reproducible cyber-agent science

---

MIT licensed.
