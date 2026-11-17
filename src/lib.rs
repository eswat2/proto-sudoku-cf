use base64::{engine::general_purpose, Engine as _};
use serde_json::json;
use sudoku::Sudoku;
use worker::*;

const PLATFORM: &str = "cloudflare";

/// A generated puzzle and its solution, both as 81-character lines.
/// Plain Rust with no worker types, so benches/puzzle.rs can time it natively.
pub struct Puzzle {
    pub puzzle: String,
    pub solution: String,
    pub blanks: usize,
}

pub fn new_puzzle() -> Puzzle {
    // NOTE:  api has changed from generate_unique to generate...
    let puzzle = Sudoku::generate().to_str_line().to_string();
    let blanks = puzzle.matches('.').count();

    // NOTE:  api changed from solve_unique to solution
    let solution = Sudoku::from_str_line(&puzzle)
        .ok()
        .and_then(|s| s.solution())
        .map(|s| s.to_string())
        .unwrap_or_default();

    Puzzle { puzzle, solution, blanks }
}

// The API is for the game page only. A browser's fetch() from the page
// sends `Sec-Fetch-Site: same-origin`; a tab, another site or curl doesn't.
// Browsers too old to send Sec-Fetch-* fall back to a same-host Referer.
// Headers can be forged, so this keeps casual callers out; it isn't auth.
fn from_the_page(req: &Request) -> bool {
    let h = req.headers();
    match h.get("Sec-Fetch-Site").ok().flatten() {
        Some(site) => site == "same-origin",
        None => {
            let host = req.url().ok().and_then(|u| u.host_str().map(str::to_owned));
            let referer = h.get("Referer").ok().flatten().and_then(|r| Url::parse(&r).ok());
            matches!((host, referer), (Some(host), Some(r)) if r.host_str() == Some(host.as_str()))
        }
    }
}

// Any path that isn't a file in public/, and any API call that didn't come
// from the page, gets a real 404, so a probe like /env.js or a direct
// request for /api/puzzle doesn't look like something that exists. The body is
// public/404.html, which the assets binding serves for a missing path
// (not_found_handling in wrangler.jsonc).
async fn not_found_page(req: &Request, env: &Env) -> Result<Response> {
    let mut asset = env.assets("ASSETS")?.fetch(req.url()?.to_string(), None).await?;
    let body = asset.text().await?;
    let mut resp = Response::from_html(body)?.with_status(404);
    let h = resp.headers_mut();
    h.set("Cache-Control", "no-store")?;
    h.set("X-Robots-Tag", "noindex")?;
    Ok(resp)
}

fn puzzle() -> Result<Response> {
    let p = new_puzzle();
    // Timings were dropped: Workers freezes the clock while code runs, so
    // they always read 0. Use `cargo bench` for performance numbers.
    let data = json!({
        "metrics": {
            "counts": { "blanks": p.blanks, "clues": 81 - p.blanks }
        },
        "puzzle": p.puzzle,
        "ref": general_purpose::STANDARD.encode(p.solution),
        "tag": PLATFORM
    });
    Response::from_json(&data)
}

#[event(fetch)]
async fn fetch(req: Request, env: Env, _ctx: Context) -> Result<Response> {
    match req.method() {
        Method::Get | Method::Head => {
            if req.path() == "/api/puzzle" && from_the_page(&req) {
                puzzle()
            } else {
                not_found_page(&req, &env).await
            }
        }
        _ => {
            let mut resp = Response::empty()?.with_status(405);
            resp.headers_mut().set("Allow", "GET, HEAD")?;
            Ok(resp)
        }
    }
}
