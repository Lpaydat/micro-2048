#!/usr/bin/env bash
set -euo pipefail

if [[ $# -lt 1 || $# -gt 2 ]]; then
  echo "Usage: scripts/dedupe-logs.sh <input.log> [output.log]" >&2
  exit 1
fi

in="$1"
out="${2:-001-log.txt}"

if [[ ! -f "$in" ]]; then
  echo "Input file not found: $in" >&2
  exit 1
fi

awk -f "$(dirname "$0")/dedupe_logs.awk" "$in" > "$out"

echo "Input lines:  $(wc -l < "$in")"
echo "Output lines: $(wc -l < "$out")"
echo "Saved cleaned log to: $out"

