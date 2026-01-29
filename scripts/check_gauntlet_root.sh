#!/usr/bin/env bash
set -euo pipefail

# Always run relative to repo root (works from CI or anywhere)
cd "$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)/.."

BIN="./target/release/cogitator"
OUT_DIR="out_ci"
RUN_DIR="${OUT_DIR}/run_0000"
GOLDEN="goldens/gauntlet_witness_root.txt"

cargo build --release

rm -rf "$OUT_DIR"
"$BIN" run \
  --agent gauntlet \
  --seed 42 \
  --runs 1 \
  --case 0 \
  --out-dir "$OUT_DIR" \
  --clean \
  --no-tui \
  --parallel=false \
  --faults off \
  --fault-profile none \
  --nix-provenance off \
  --pass-threshold 0.5

ROOT="$(tr -d '\r\n' < "${RUN_DIR}/witness_root.txt")"

if [[ ! -f "$GOLDEN" ]]; then
  echo "Missing golden file: $GOLDEN"
  echo "Generate it with:"
  echo "  cp ${RUN_DIR}/witness_root.txt $GOLDEN"
  exit 2
fi

EXPECTED="$(tr -d '\r\n' < "$GOLDEN")"

if [[ "$ROOT" != "$EXPECTED" ]]; then
  echo "Witness root changed!"
  echo "expected: $EXPECTED"
  echo "actual:   $ROOT"
  echo
  echo "Environment:"
  pwd
  git rev-parse HEAD
  rustc -Vv
  cargo -V
  echo
  echo "Checksums:"
  sha256sum "$GOLDEN" "${RUN_DIR}/witness_root.txt"
  echo
  echo "Byte-level diff (golden):"
  od -An -tx1 -c "$GOLDEN"
  echo
  echo "Byte-level diff (actual):"
  od -An -tx1 -c "${RUN_DIR}/witness_root.txt"
  echo
  if [[ -f "${RUN_DIR}/witness_manifest.json" ]]; then
    echo "Witness manifest:"
    cat "${RUN_DIR}/witness_manifest.json"
    echo
  fi
  if [[ -f "${RUN_DIR}/meta.json" ]]; then
    echo "Meta:"
    cat "${RUN_DIR}/meta.json"
    echo
  fi
  echo
  echo "Drift report:"
  cat "${RUN_DIR}/drift_report.json"
  exit 1
fi

echo "OK: witness root matches $ROOT"
