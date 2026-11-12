# CLAUDE.md

Guidance for Claude Code when working in this repository.

## What this is

`proto-sudoku` on Cloudflare Workers: a single-player sudoku page at
`sudoku.eswat2.app`, plus the Rust puzzle API it calls. The API is
`src/lib.rs`, compiled to WebAssembly with workers-rs. The page is the
`proto-sudoku-wc` web component (MIT, on npm), served from `public/` by
Workers Static Assets.

It was ported from the local Actix server, which continues as the public
`eswat2/rust-sudoku-api`. This repo's history starts at a snapshot of that
server (tag `actix-final`), followed by the port.

**This repo is public.** Don't name private repos, local paths or internal
links in anything committed here: code, docs, commit messages.

## Layout

- `src/lib.rs`: the whole Worker. It answers `/api/puzzle` and OPTIONS on
  any path. Any other `/api` path is a JSON 404, any other path a 404 with
  `public/404.html` as the body (`not_found_handling` in `wrangler.jsonc`),
  and methods other than GET, HEAD and OPTIONS get 405. CORS headers go on
  every response. The old `/:name` greeting route was removed because it
  answered 200 to every scanner probe.
- `public/`: everything the site serves. `index.html` is the game, `about.html` is
  styled like the poker and retro apps' about pages, `architecture.html`
  explains how it's built, and `llms.txt` describes the site and API for LLMs.
- `docs/cloudflare-port.html`: the runbook from the port, kept as a record.
  A copy is published as a private claude.ai artifact; keep the two in step.
- `benches/puzzle.rs`: Criterion benchmarks for generate, solve and a full
  puzzle (`cargo bench`), run natively.
- `scripts/deploy.sh`: the only way to deploy (see below).
- `scripts/update.sh`: the daily dependency update (see "Commands").
- `scripts/capture-about.mjs`: regenerates the About page screenshots from a
  running `npx wrangler dev`. Usage is in its header.
- `.template.html`, `.wc-loader` (`.update` is a symlink to it),
  `.wc-tags.json`, `.versions`, `.tags`: the web-component version bumper
  (see "The page").

## Commands

```
npx wrangler dev                    # local, on :8787
bash scripts/deploy.sh --dry-run    # build and check, no upload
bash scripts/deploy.sh              # deploy to production
cargo bench                         # puzzle timings, natively
cargo outdated --exclude getrandom  # see "Dependencies"
bash scripts/update.sh --dry-run    # pending updates, changes nothing
bash scripts/update.sh              # cargo update, check, commit + push
```

`scripts/update.sh` has the same clean-`main` guard as `deploy.sh`. It treats
a changed `Cargo.lock` as the signal, runs the wasm build
(`wrangler deploy --dry-run`) and the native one (`cargo test`,
`cargo bench --no-run`), then commits `» cargo update` and pushes. It only
reports major-version bumps and never deploys. `--quiet` prints nothing when
there's nothing to do.

The build needs the wasm target (`rustup target add wasm32-unknown-unknown`).
`wrangler.jsonc` runs `worker-build --release`, which the first build installs.

## Deploying

- **Pushing does not deploy.** Workers Builds was tried and dropped: with no
  Rust cache, each push took 6-7 minutes. Deploys are local and take about
  10-30 seconds.
- `scripts/deploy.sh` refuses unless you're on a clean `main` that matches
  `origin/main`, then tags the deploy with the commit
  (`wrangler deploy --message`). Commit and push first.
- A deploy is a production change. Ask before running it.
- **The Worker is named `proto-sudoku-cf`.** It was first deployed as
  `rust-sudoku-api` and renamed in the dashboard (Settings), which keeps its
  history and custom domain. Rename it the same way, and change `name` in
  `wrangler.jsonc` to match before the next deploy: a deploy under a name
  that doesn't exist creates a new Worker. The `workers.dev` address is off
  (`workers_dev: false`, `preview_urls: false`), as on the poker and retro
  apps.
- `sudoku.eswat2.app` is a Workers Custom Domain set in `wrangler.jsonc`.
  If it ever goes missing, `npx wrangler triggers deploy` re-attaches it
  without redeploying code.

## Dependencies

- **Keep `getrandom = { version = "0.2", features = ["js"] }`.** `sudoku` 0.8
  uses `rand` 0.8, which uses `getrandom` 0.2. The direct dependency exists
  only to turn on `js`, without which the wasm build fails. Raising it to
  0.3 or later adds a second `getrandom` and breaks the build.
- That pin is also why plain `cargo outdated` lists dozens of "Removed" rows:
  in its imagined newest tree, `getrandom` 0.4 has no `js` feature and drops
  the whole wasm-bindgen subtree. Use `--exclude getrandom`.
- `sudoku` 0.8.0 (2023) is the newest release, and the crate is quiet.

## The API

- Keep the `/api/puzzle` JSON shape: `metrics.counts`, `puzzle`, `ref`, `tag`.
  The page, `llms.txt` and outside callers depend on it.
- `metrics.nanos` was removed on purpose. Workers freezes the clock while
  code runs, so timers read 0, and `std::time::Instant` panics in wasm.
  Timings live in `cargo bench`.
- `tag` is `"cloudflare"`.

## The page

- `public/index.html` is generated. Edit `.template.html`, then run
  `bash .wc-loader` (it isn't executable). It reads a local cache of
  web-component versions, rewrites `public/index.html`, and runs
  `git commit -a` and `git push`, so commit other edits first.
- The component is pinned to an exact version on jsDelivr. Don't switch to
  `@latest`: Stencil lazy-loads chunks through the same URL, and jsDelivr
  caches `@latest` per file, so a release can mix versions and break.
- `platform="cloudflare"` makes the component call the relative `/api`, so
  the page works on any hostname that serves it.

## Licence

- The repo is **AGPL-3.0-or-later** because the Worker links the `sudoku`
  crate, which is AGPL-3.0. The README's "license" section explains it.
- The **"Source (AGPL-3.0)" link** in the About page footer and the "Source"
  section of `llms.txt` are the section 13 offer of source to network users.
  Keep both, and keep them pointing at this repo.
- New dependencies must be AGPL-compatible.

## Keeping the copy true

- The About page, `architecture.html`, `llms.txt` and the README describe the
  same facts (routes, response shape, hostnames, deploys). A change to one
  usually means updating the others.
- `eswat2.app` lists this app in its own `llms.txt` and app cards. If the
  description or URLs change, update that site too.
