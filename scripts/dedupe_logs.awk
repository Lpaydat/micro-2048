#!/usr/bin/awk -f
# Dedupe log lines by ignoring the leading timestamp and keeping first occurrence.
# Preserves original order of first appearances.
# Usage: awk -f scripts/dedupe_logs.awk INPUT.log > 001-log.txt

{
    raw = $0

    # Derive a message key by removing the leading timestamp (ISO 8601 Z) if present,
    # otherwise fall back to removing the first whitespace-separated token.
    key = raw

    # Try strict ISO 8601 Z format: 2025-09-28T06:11:47.407127Z
    if (key ~ /^[0-9]{4}-[0-9]{2}-[0-9]{2}T[0-9:.]+Z[ \t]+/) {
        sub(/^[0-9]{4}-[0-9]{2}-[0-9]{2}T[0-9:.]+Z[ \t]+/, "", key)
    } else {
        # Fallback: strip the first token (common for timestamp-first logs)
        sub(/^[^ ]+[ \t]+/, "", key)
    }

    if (!(key in seen)) {
        print raw
        seen[key] = 1
    }
}

