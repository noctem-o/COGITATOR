#!/usr/bin/env bash
set -euo pipefail

# Resolve and print the current commit for rustsec/audit-check v2.x.y tags.
# Intended for maintainer hygiene when rotating SHA-pinned actions.
# This utility is best-effort and is not used by CI.
repo="https://github.com/rustsec/audit-check.git"
pattern='refs/tags/v2.*'

if ! tags="$(git ls-remote --tags --refs "$repo" "$pattern" 2>/dev/null)"; then
  echo "warning: unable to list rustsec/audit-check tags via git protocol" >&2
  exit 0
fi

if [[ -z "$tags" ]]; then
  echo "warning: no v2.x.y tags found for rustsec/audit-check" >&2
  exit 0
fi

chosen_tag="$(printf '%s\n' "$tags" | awk '{print $2}' | sed 's#refs/tags/##' | sort -V | tail -n1)"

# Try annotated-tag dereference first; fall back to direct tag OID.
sha="$(git ls-remote "$repo" "refs/tags/${chosen_tag}^{}" 2>/dev/null | awk 'NR==1{print $1}')"
if [[ -z "$sha" ]]; then
  sha="$(git ls-remote "$repo" "refs/tags/${chosen_tag}" 2>/dev/null | awk 'NR==1{print $1}')"
fi

if [[ -z "$sha" || ! "$sha" =~ ^[0-9a-f]{40}$ ]]; then
  echo "warning: failed to resolve commit for tag ${chosen_tag}" >&2
  exit 0
fi

echo "chosen_tag=${chosen_tag}"
echo "resolved_sha=${sha}"
echo "uses: rustsec/audit-check@${sha}  # ${chosen_tag}"

if [[ -n "${GITHUB_STEP_SUMMARY:-}" ]]; then
  {
    echo "### rustsec/audit-check ref"
    echo
    echo "- Tag: \`${chosen_tag}\`"
    echo "- Resolved commit: \`${sha}\`"
    echo "- Uses line: \`uses: rustsec/audit-check@${sha}  # ${chosen_tag}\`"
  } >> "$GITHUB_STEP_SUMMARY"
fi
