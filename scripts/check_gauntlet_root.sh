#!/usr/bin/env bash
set -euo pipefail

echo "Warning: scripts/check_gauntlet_root.sh is deprecated; use scripts/check_ordeal_root.sh." >&2
exec "$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)/check_ordeal_root.sh" "$@"
