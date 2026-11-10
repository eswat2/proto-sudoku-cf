#!/usr/bin/env bash
# ---------------------------------------------------------------------
# NOTE:  build + ship for Cloudflare Workers Builds (the dashboard's
#        deploy / preview commands). The build image has no Rust, so
#        install it, build the wasm with worker-build, then hand off to
#        wrangler...
#
#   bash scripts/cf-build.sh deploy    # production (main)
#   bash scripts/cf-build.sh preview   # other branches: upload a version
#   bash scripts/cf-build.sh check     # build + wrangler deploy --dry-run
set -euo pipefail

MODE="${1:-deploy}"

if ! command -v cargo >/dev/null 2>&1; then
  if [ ! -f "$HOME/.cargo/env" ]; then
    echo "-- installing rust (minimal) + wasm32 target"
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs |
      sh -s -- -y --profile minimal --target wasm32-unknown-unknown
  fi
  # shellcheck disable=SC1091
  . "$HOME/.cargo/env"
fi

rustup target add wasm32-unknown-unknown >/dev/null
cargo --version

echo "-- installing worker-build"
cargo install -q "worker-build@^0.8"

echo "-- building (release)"
worker-build --release

case "$MODE" in
  deploy)  npx wrangler deploy ;;
  preview) npx wrangler versions upload ;;
  check)   npx wrangler deploy --dry-run ;;
  *) echo "usage: $0 [deploy|preview|check]" >&2; exit 2 ;;
esac
