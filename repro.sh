#!/bin/bash
SEED=${1:-42}
RUNS=${2:-5000}
RUST_LOG=info cargo run --release -- --seed $SEED --runs $RUNS
set -euo pipefail

SEED=42
RUNS=5000
OUTPUT=results.csv

while [[ $# -gt 0 ]]; do
  case "$1" in
    --seed)
      SEED="$2"
      shift 2
      ;;
    --runs)
      RUNS="$2"
      shift 2
      ;;
    --output)
      OUTPUT="$2"
      shift 2
      ;;
    *)
      echo "Unknown argument: $1" >&2
      exit 1
      ;;
  esac
done

RUST_LOG=info cargo run --release -- --seed "$SEED" --runs "$RUNS" --output "$OUTPUT"
