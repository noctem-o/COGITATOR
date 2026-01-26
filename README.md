diff --git a/README.md b/README.md
index 69fd9fc87d1c8db16ce8674a2261ca14b1073ed2..871794a8e8b0ecc434ab5c3e8bb3c9ae2cc6df55 100644
--- a/README.md
+++ b/README.md
@@ -1,10 +1,34 @@
 # COGITATOR v2.1.3
 
 Deterministic evaluation harness for autonomous security agents.
 
 ## Reproducibility
 
 ```bash
 cargo build --release --locked
-./repro.sh --seed 42 --runs 5000
+./repro.sh --seed 42 --runs 5000 --output results.csv
+```
 
+## Usage
+
+Run the harness directly:
+
+```bash
+cargo run --release -- --seed 42 --runs 5000 --output results.csv
+```
+
+The CLI flags:
+
+- `--seed`: deterministic seed for generating runs.
+- `--runs`: number of evaluation runs to simulate.
+- `--output`: path to write the CSV results (default: `results.csv`).
+
+## Output
+
+The CSV contains one row per run with the following columns:
+
+- `run_id`
+- `case_id`
+- `difficulty`
+- `score`
+- `passed`
