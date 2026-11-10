#!/usr/bin/env bash
# ---------------------------------------------------------------------
# NOTE:  build locally and deploy to Cloudflare, but only what's in git:
#        on main, nothing uncommitted, and in sync with origin/main.
#        The deploy is tagged with the commit, so the dashboard shows
#        which commit is live...
#
#   bash scripts/deploy.sh             # build + deploy
#   bash scripts/deploy.sh --dry-run   # build + check, no upload
set -euo pipefail

cd "$(git rev-parse --show-toplevel)"

fail () { echo "-- deploy refused: $1" >&2; exit 1; }

BRANCH=$(git rev-parse --abbrev-ref HEAD)
[ "$BRANCH" = "main" ] || fail "on '$BRANCH', not main"

[ -z "$(git status --porcelain)" ] || fail "uncommitted changes (git status)"

git fetch -q origin main
LOCAL=$(git rev-parse HEAD)
REMOTE=$(git rev-parse origin/main)
[ "$LOCAL" = "$REMOTE" ] || {
  AHEAD=$(git rev-list --count origin/main..HEAD)
  BEHIND=$(git rev-list --count HEAD..origin/main)
  fail "main is $AHEAD ahead / $BEHIND behind origin/main (push or pull first)"
}

SHA=$(git rev-parse --short HEAD)
MSG="$SHA $(git log -1 --format=%s)"
echo "-- deploying $MSG"

# wrangler runs the build from wrangler.jsonc (worker-build --release)
npx wrangler deploy --message "${MSG:0:100}" "$@"
