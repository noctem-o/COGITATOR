#!/usr/bin/env bash
set -euo pipefail

cd "$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)/.."

cargo build --release

./target/release/cogitator run \
  --agent gauntlet \
  --seed 42 \
  --runs 1 \
  --case 0 \
  --out-dir out_ci \
  --clean \
  --no-tui \
  --faults off \
  --fault-profile none

root_file="out_ci/run_0000/witness_root.txt"
expected_file="goldens/gauntlet_witness_root.txt"

if [[ ! -f "${expected_file}" ]]; then
  echo "ERROR: Missing ${expected_file}." >&2
  echo "Generate it by running:" >&2
  echo "  ./target/release/cogitator run --agent gauntlet --seed 42 --runs 1 --case 0 --out-dir out_ci --clean --no-tui --faults off --fault-profile none" >&2
  echo "Then copy ${root_file} to ${expected_file}." >&2
  exit 1
fi

if [[ ! -f "${root_file}" ]]; then
  echo "ERROR: Missing ${root_file} after gauntlet run." >&2
  exit 1
fi

actual_root=$(tr -d '\r\n' < "${root_file}")
expected_root=$(tr -d '\r\n' < "${expected_file}")

if [[ "${actual_root}" != "${expected_root}" ]]; then
  echo "Witness root mismatch." >&2
  echo "Expected: ${expected_root}" >&2
  echo "Actual:   ${actual_root}" >&2
  if [[ -f "out_ci/run_0000/drift_report.json" ]]; then
    echo "Drift report:" >&2
    cat out_ci/run_0000/drift_report.json >&2
  else
    echo "Drift report missing at out_ci/run_0000/drift_report.json" >&2
  fi
  exit 1
fi

echo "OK: gauntlet witness root matches ${actual_root}"
