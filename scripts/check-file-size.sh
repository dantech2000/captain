#!/usr/bin/env bash
# Fail if any Rust source file is over the hard line limit.
# Files under a tests/ directory are exempt. See docs/adr/0003-layered-workspace.md.
set -euo pipefail

LIMIT="${LIMIT:-500}"
WARN="${WARN:-300}"
status=0

while IFS= read -r file; do
  lines=$(wc -l < "$file")
  if (( lines > LIMIT )); then
    echo "error: $file has $lines lines (limit $LIMIT)"
    status=1
  elif (( lines > WARN )); then
    echo "warning: $file has $lines lines (split it before it reaches $LIMIT)"
  fi
done < <(find crates -name '*.rs' -not -path '*/tests/*' -not -path '*/target/*')

exit "$status"
