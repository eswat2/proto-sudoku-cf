# proto-sudoku-cf

simple sudoku api implemented in Rust, running on Cloudflare Workers as WebAssembly (via [workers-rs][workers-rs])...

- [sudoku.eswat2.app][app] -- _the page, a single-file vanilla web component (no build, no dependencies)_
- [sudoku.eswat2.app/api/puzzle][app-puzzle] -- _the api_

## dependencies:

[![worker][badge-worker]][worker]
[![base64][badge-base64]][base64]
[![serde_json][badge-serde-json]][serde-json]
[![sudoku][badge-sudoku]][sudoku]
[![criterion][badge-criterion]][criterion]

## usage:

`npx wrangler dev`

- [localhost:8787][local]
- [localhost:8787/api/puzzle][puzzle]

to deploy (builds locally, then uploads):

```
bash scripts/deploy.sh
```

the script only deploys a clean `main` that matches `origin/main`, and tags the deploy with the commit (`wrangler deploy --message`). `bash scripts/deploy.sh --dry-run` builds and checks without uploading.

the first build installs `worker-build` and needs the wasm target (`rustup target add wasm32-unknown-unknown`).

> there is no auto-deploy: Cloudflare Workers Builds was tried, but with no Rust cache it took 6-7 minutes per push. Local deploys take about 10-30 seconds (the first scripted deploy took 9 seconds).

a deploy publishes to one hostname, **sudoku.eswat2.app**: a Workers Custom Domain, set in `wrangler.jsonc` (`routes` with `custom_domain: true`). Cloudflare manages its DNS record and certificate, and every `wrangler deploy` re-applies it. A brand-new custom domain can take a few minutes before DNS answers. the workers.dev address and Preview URLs are off (`"workers_dev": false`, `"preview_urls": false`).

the page calls the api with the relative path `/api/puzzle`, so it works on any hostname that serves it.

to check which commit is live:

```
npx wrangler deployments list
```

if a custom domain ever goes missing (for example after a deploy made with a token that can't edit the `eswat2.app` zone), re-attach it without redeploying code:

```
npx wrangler triggers deploy
```

## routes:

| Route | Description |
| :--- | :--- |
| `/` | _`public/index.html`, served by Workers Static Assets_ |
| `/about` | _`public/about.html`, styled like the poker and retro about pages. `node scripts/capture-about.mjs` regenerates its screenshots in `public/images/`_ |
| `/architecture` | _`public/architecture.html`, styled like the poker and retro architecture pages_ |
| `/llms.txt` | _`public/llms.txt`, notes for AI clients_ |
| `/favicon.svg`, `/favicon.ico` | _the icon; the `.ico` is a 32px render of the svg_ |
| `/api/puzzle` | _a new puzzle and its solution_ |
| any other `/api` path | _404, `{"error": "not found"}`_ |
| any other path | _404, with `public/404.html` as the body, `no-store` and `noindex`_ |

static files win over the Worker: any path matching a file in `public/` never runs the Rust code. methods other than `GET`, `HEAD` and `OPTIONS` get 405. every Worker response carries permissive CORS headers, including 404s, 405s and `OPTIONS` preflights.

## output:

```
{
  "metrics": {
    "counts": {
      "blanks": 52,
      "clues": 29
    }
  },
  "puzzle": "51..6...28.2......6..3.8.9.4..9..8.........5.1564............192...9...8...54....",
  "ref": "NTE5NzY0MzgyODMyMTU5NjQ3Njc0MzI4MTk1NDI3OTM1ODYxOTgzMjE2NzU0MTU2NDg3OTIzMzY1ODcyNDE5MjQxNjkzNTc4Nzk4NTQxMjM2",
  "tag": "cloudflare"
}
```

| Property | Description |
| ---: | :--- |
| **metrics** | _insight into the puzzle counts_ |
| **puzzle** | _a string representing the puzzle board (dots are blank cells)_ |
| **ref** | _the base64 encoded solution to the puzzle_ |
| **tag** | _the platform serving the api_ |

> `metrics.nanos` was dropped in the Workers port: Workers freezes the clock while code runs, so timings always read 0. Use `cargo bench` instead.

## benchmarks:

```
cargo bench
```

times `generate`, `solve` and `new_puzzle` natively with [criterion][criterion] (`benches/puzzle.rs`). native is typically 1.5-3x faster than the wasm build Workers runs.

## developer notes:

the game is `public/index.html`: one `<proto-sudoku>` custom element with Shadow DOM, plain CSS and an inline script, about 6 KB gzipped. there's nothing to build or bump; edit the file and reload `npx wrangler dev`. it reproduces the look and behaviour of the [proto-sudoku-wc][proto-sudoku-wc] Stencil component the page used to load from jsDelivr (about 40 KB gzipped), down to its saved-game keys, so games in progress carry over.

to check for updates (changes nothing):

```
bash scripts/update.sh --dry-run
```

to update packages:

```
bash scripts/update.sh
```

this runs `cargo update`. if `Cargo.lock` changed, it checks the wasm build (`wrangler deploy --dry-run`) and the native one (`cargo test`, `cargo bench --no-run`), then commits `» cargo update` and pushes. it doesn't deploy, so run `bash scripts/deploy.sh` afterwards. it lists major-version bumps but doesn't apply them, because they need a `Cargo.toml` change. `--quiet` prints nothing when there's nothing to do, for a daily runner.

plain `cargo outdated` is noisy here because of the `getrandom` 0.2 pin. use `cargo outdated --exclude getrandom` instead.

## references:

- [workers-rs][workers-rs] - _Rust SDK for Cloudflare Workers_
- [cargo-outdated][out-dated] - _new cargo subcommand_
- [cargo-update][up-date] - _update dependencies_
- [docs/cloudflare-port.html](docs/cloudflare-port.html) - _the runbook for the Actix → Workers port_

## status:

Heroku's free tier ended in November 2022, and the Actix version of this repo became local-only. In November 2026 it was ported to Cloudflare Workers.

The serverless version on Vercel is still running...

- [puzzle][puzzle-io] -- _vercel_

## license:

This repo is licensed under [AGPL-3.0-or-later](LICENSE).

**Why AGPL?** Puzzles are generated and solved by the [sudoku][sudoku] crate,
which is licensed AGPL-3.0. The Worker compiles that crate into its WebAssembly,
so the Worker as a whole is a work based on it. The AGPL's network clause
(section 13) says that when such a program is offered to users over a network,
those users must be able to get its source under the same licence. This public
repo, linked from the footer of [sudoku.eswat2.app/about][about], is that offer.

**What that means for you:**

- You can read, run, change and redistribute this code.
- If you run a modified version as a public service, you have to offer its
  users your source under AGPL-3.0-or-later too.
- There's no warranty. See the [LICENSE](LICENSE) for the full terms.

**What it doesn't cover:**

- The game UI in `public/index.html` runs in the browser and only calls this
  API over HTTP, so it isn't part of the AGPL work. It adapts the author's
  own MIT-licensed code, including [proto-sudoku-wc][proto-sudoku-wc], and
  credits it in its header.
- The older local Actix version is in the public
  [eswat2/rust-sudoku-api][rust-sudoku-api] repo, which is licensed
  separately under AGPL-3.0-or-later as well.


[app]: https://sudoku.eswat2.app
[app-puzzle]: https://sudoku.eswat2.app/api/puzzle
[proto-sudoku-wc]: https://www.npmjs.com/package/proto-sudoku-wc
[workers-rs]: https://github.com/cloudflare/workers-rs
[worker]: https://crates.io/crates/worker
[base64]: https://crates.io/crates/base64
[serde-json]: https://crates.io/crates/serde_json
[sudoku]: https://crates.io/crates/sudoku
[about]: https://sudoku.eswat2.app/about
[rust-sudoku-api]: https://github.com/eswat2/rust-sudoku-api
[criterion]: https://crates.io/crates/criterion
[local]: http://localhost:8787
[puzzle]: http://localhost:8787/api/puzzle
[puzzle-io]: https://sudoku-rust-api.vercel.app/api/puzzle

[out-dated]: https://github.com/kbknapp/cargo-outdated
[up-date]: https://doc.rust-lang.org/cargo/commands/cargo-update.html

[badge-worker]: https://img.shields.io/badge/crates.io-worker-orange
[badge-base64]: https://img.shields.io/badge/crates.io-base64-orange
[badge-serde-json]: https://img.shields.io/badge/crates.io-serde__json-orange
[badge-sudoku]: https://img.shields.io/badge/crates.io-sudoku-orange
[badge-criterion]: https://img.shields.io/badge/crates.io-criterion-orange
