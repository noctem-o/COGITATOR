
 # COGITATOR v2.1.3
 
 Deterministic evaluation harness for autonomous security agents.
 
 ## Reproducibility
 
```bash
cargo build --release --locked
./repro.sh --seed 42 --runs 5000 --output results.csv
```

## Usage

Run the harness directly:

```bash
cargo run --release -- --seed 42 --runs 5000 --output results.csv
```

The CLI flags:

- `--seed`: deterministic seed for generating runs.
- `--runs`: number of evaluation runs to simulate.
- `--output`: path to write the CSV results (default: `results.csv`).

## Output

The CSV contains one row per run with the following columns:

- `run_id`
- `case_id`
- `difficulty`
- `score`
- `passed`
