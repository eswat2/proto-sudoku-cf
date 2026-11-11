#!/usr/bin/env bash
# ---------------------------------------------------------------------
# NOTE:  daily dependency update for the Worker.  cargo update, and if
#        Cargo.lock changed, check both builds (wasm + native), then
#        commit and push.  Never deploys.  Major-version bumps are only
#        reported: they need a Cargo.toml change and a review...
#
#   bash scripts/update.sh             # update, check, commit + push
#   bash scripts/update.sh --dry-run   # report only, change nothing
#   bash scripts/update.sh --quiet     # no output when nothing changed
#
# why not plain `cargo outdated`?  the getrandom 0.2 "js" pin makes it
# list dozens of "Removed" rows, and it always lists major bumps that
# cargo update can't apply.  a changed Cargo.lock is the real signal.
set -euo pipefail

cd "$(git rev-parse --show-toplevel)"

DRY=0
QUIET=0
for arg in "$@"; do
  case "$arg" in
    --dry-run) DRY=1 ;;
    --quiet)   QUIET=1 ;;
    -h|--help) sed -n '3,14p' "$0" | sed 's/^# \{0,1\}//'; exit 0 ;;
    *) echo "-- unknown option: $arg" >&2; exit 2 ;;
  esac
done

say () { echo "-- $*"; }
fail () { echo "-- update refused: $1" >&2; exit 1; }

# report root deps with a newer major version (not applied)
majors () {
  local out
  out=$(cargo outdated --root-deps-only --exclude getrandom 2>/dev/null || true)
  if [ -z "$out" ] || grep -q "All dependencies are up to date" <<<"$out"; then
    say "no major-version bumps"
  else
    say "major-version bumps (not applied; edit Cargo.toml and review):"
    echo "$out"
  fi
}

# --- same guards as deploy.sh --------------------------------------
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

# --- dry run: report only ------------------------------------------
if [ "$DRY" = 1 ]; then
  say "cargo update --dry-run"
  cargo update --dry-run
  majors
  exit 0
fi

# --- cargo update; did Cargo.lock change? --------------------------
LOG=$(cargo update 2>&1) || { echo "$LOG" >&2; fail "cargo update failed"; }

if git diff --quiet -- Cargo.lock; then
  [ "$QUIET" = 1 ] && exit 0
  say "Cargo.lock unchanged, nothing to do"
  majors
  exit 0
fi

say "cargo update"
echo "$LOG"

# from here on, a failed check leaves Cargo.lock modified for a look
trap 'echo "-- check failed: Cargo.lock left modified (git checkout Cargo.lock to drop it)" >&2' ERR

# --- checks --------------------------------------------------------
# the wasm build, as deploy.sh --dry-run runs it.  deploy.sh itself
# would refuse here (dirty tree), so call wrangler directly.
say "wasm build (wrangler deploy --dry-run)"
npx wrangler deploy --dry-run

say "native build (cargo test, cargo bench --no-run)"
cargo test
cargo bench --no-run

trap - ERR

# --- commit + push -------------------------------------------------
git commit -q -m "» cargo update" -- Cargo.lock
git push -q origin main
say "pushed $(git rev-parse --short HEAD) » cargo update"

majors

say "not deployed.  to ship it:  bash scripts/deploy.sh"
