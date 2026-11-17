# CLAUDE.md

Guidance for Claude Code when working in this repository.

## What this is

`proto-sudoku` on Cloudflare Workers: a single-player sudoku page at
`sudoku.eswat2.app`, plus the Rust puzzle API it calls. The API is
`src/lib.rs`, compiled to WebAssembly with workers-rs. The page is a
single-file vanilla web component in `public/index.html`, served by Workers
Static Assets.

It was ported from the local Actix server, which continues as the public
`eswat2/rust-sudoku-api`. This repo's history starts at a snapshot of that
server (tag `actix-final`), followed by the port.

**This repo is public.** Don't name private repos, local paths or internal
links in anything committed here: code, docs, commit messages.

## Layout

- `src/lib.rs`: the whole Worker. It answers `/api/puzzle`, but only to
  the game page: `from_the_page()` wants `Sec-Fetch-Site: same-origin`, or
  a same-host `Referer` from browsers without Sec-Fetch-*. Anything else (a
  tab, another site, plain curl, any other path) gets a 404 with
  `public/404.html` as the body (`not_found_handling` in `wrangler.jsonc`),
  and methods other than GET and HEAD get 405. There are no CORS headers.
  The API was public with open CORS until it was made page-only on purpose;
  don't reopen it. The old `/:name` greeting route was removed because it
  answered 200 to every scanner probe.
- No `run_worker_first`: a browser tab opening `/api/puzzle` is answered
  with `404.html` by the asset layer and never reaches the Worker. That's
  wanted; it was briefly added so tabs got JSON, then reverted.
- `public/api-sample.html` (`/api-sample`): a saved `/api/puzzle` response
  shown as a page, styled like `404.html`, with what each field means. About
  ("Sample response"), Architecture, `llms.txt` and the README link it in
  place of the live API. It's a page, not a `.json` file, because a WAF rule
  on the `eswat2.app` zone answers every `*.json` path (and `.php`, `.env`)
  with a 403 before it reaches the Worker.
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
- **Move `worker` and `worker-build` together.** The build command in
  `wrangler.jsonc` installs `worker-build@^0.9` to match `worker = "0.9"`
  in `Cargo.toml`. On a major bump of `worker`, raise both: `worker-build`
  refuses a `worker` from another major version, and the wasm build fails.
  Also update the workers-rs version named in `architecture.html`.
- `sudoku` 0.8.0 (2023) is the newest release, and the crate is quiet.

## The API

- Keep the `/api/puzzle` JSON shape: `metrics.counts`, `puzzle`, `ref`, `tag`.
  The page and saved games depend on it. If it changes, regenerate
  the JSON in `public/api-sample.html` and the examples in `llms.txt` and the README.
- To call it locally: `curl -H 'Sec-Fetch-Site: same-origin' localhost:8787/api/puzzle`.
- `metrics.nanos` was removed on purpose. Workers freezes the clock while
  code runs, so timers read 0, and `std::time::Instant` panics in wasm.
  Timings live in `cargo bench`.
- `tag` is `"cloudflare"`.

## The page

- `public/index.html` is the whole game: one `<proto-sudoku>` custom element
  with Shadow DOM, plain CSS and an inline script. No build step, no
  dependencies, no CDN. Edit it directly. The file is about 7 KB compressed;
  a cold load of the live page is about 8.5 KB with headers and favicon.
- It reproduces the Stencil component `proto-sudoku-wc` (MIT) that the page
  used to load from jsDelivr. **Keep it looking and behaving like that component**:
  - The look is Stencil's Tailwind classes written out as plain CSS. There's
    no CSS reset on purpose: buttons, the `hr` and the title start from
    browser defaults, as they did with Tailwind's preflight off.
  - Entries are only checked when you press Check ?, which erases wrong ones
    silently. A full board isn't solved until Check finds every cell right.
  - The keypad lists the digits no peer holds, worked out when the cell is
    selected and not refreshed after an entry.
  - A first visit shows an empty board; nothing is fetched until New Puzzle.
  - The fetch matches ky's: a 10s timeout across all tries, and two retries
    (300ms, then 600ms) on a network error or a retryable status.
- Saved state keeps Stencil's keys and formats, so games carry over:
  `proto-sudoku::data` (the API response), `::inputs` (81 characters) and
  `::pick` (a cell index or `null`).
- **One deliberate difference:** every entry is saved as it's made, so a
  reload keeps unchecked entries. Stencil saved `::inputs` only on Check.
- **A second deliberate difference:** About and Architecture links sit in
  the top-left corner, on the fingerprint's row (`.links`, `#4e5964`). The row
  sits 3px below the icon's top (`top:.1875rem`), so the capitals and
  underline centre on the fingerprint's ink (about 2-22px down its 24px box);
  centred line boxes looked dropped and top alignment looked high. Padding
  below keeps each link 24px tall. The underline is a light `#c5ccd3`, so it
  marks links on touch screens without weighing on the header; on hover the
  text and underline turn navy `#001f3f`, as the fingerprint does. They use space the title's top margin already
  reserves, so the page stays 627px tall and still fits a phone without
  scrolling. Don't move them to a footer below the board: that pushes the
  game into scrolling.
- The toolbar's bottom-right label, where Stencil showed its Tailwind
  version, reads `id-<hash>`: the first 8 hex characters of a SHA-256 of
  the inline script, computed in the browser. It changes whenever the code
  does, so it shows which version a browser is running. Compare it with
  `scripts/build-id.sh`.
- **Restart `npx wrangler dev` after editing anything in `public/`.**
  Wrangler 4.161 serves the edited file but keeps the ETag it computed at
  startup, so a browser that already has the page gets `304 Not Modified`
  and keeps running the old copy, even on a normal reload. If the id
  label doesn't match `bash scripts/build-id.sh`, this is why. Deploys
  aren't affected: assets are hashed again on upload.
- Cells keep Stencil's `cell-<n>` class; `scripts/capture-about.mjs` uses it.
- It fetches the relative `/api/puzzle`, so the page works on any hostname
  that serves it.

## Licence

- The repo is **AGPL-3.0-or-later** because the Worker links the `sudoku`
  crate, which is AGPL-3.0. The README's "license" section explains it.
- The **"Source (AGPL-3.0)" link** in the About page footer and the "Source"
  section of `llms.txt` are the section 13 offer of source to network users.
  Keep both, and keep them pointing at this repo.
- New dependencies must be AGPL-compatible.
- `public/index.html` adapts the author's own MIT-licensed code, including
  `proto-sudoku-wc`, and credits it in its header comment. Keep that credit.

## Keeping the copy true

- The About page, `architecture.html`, `llms.txt` and the README describe the
  same facts (routes, response shape, hostnames, deploys). A change to one
  usually means updating the others.
- `eswat2.app` lists this app in its own `llms.txt` and app cards. If the
  description or URLs change, update that site too.
