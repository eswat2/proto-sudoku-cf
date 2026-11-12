#!/bin/bash
# Prints the id the page shows in its toolbar ("id <hash>"): the first
# 8 hex characters of a SHA-256 of the page's inline <script>. Usage:
#
#   bash scripts/build-id.sh                            # public/index.html
#   bash scripts/build-id.sh http://localhost:8787/     # what a server serves
#   bash scripts/build-id.sh https://sudoku.eswat2.app/
set -euo pipefail

src="${1:-public/index.html}"
if [[ "$src" == http* ]]; then
  html=$(curl -fsS "$src")
else
  html=$(cat "$src")
fi

printf '%s' "$html" | python3 -I -c '
import hashlib, sys
html = sys.stdin.read()
start = html.rindex("<script>") + len("<script>")
end = html.index("</script>", start)
print(hashlib.sha256(html[start:end].encode()).hexdigest()[:8])
'
