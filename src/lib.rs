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

fn with_cors(mut resp: Response) -> Result<Response> {
    let h = resp.headers_mut();
    h.set("Access-Control-Allow-Origin", "*")?;
    h.set("Access-Control-Allow-Headers", "*")?;
    h.set("Access-Control-Allow-Methods", "GET,OPTIONS")?;
    Ok(resp)
}

fn not_found_json() -> Result<Response> {
    Ok(Response::from_json(&json!({ "error": "not found" }))?.with_status(404))
}

// Any path that isn't a file in public/ or /api/puzzle gets a real 404, so a
// probe like /env.js doesn't look like something that exists. The body is
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
    let path = req.path();
    let resp = match req.method() {
        Method::Options => Response::empty()?,
        Method::Get | Method::Head => {
            if path == "/api/puzzle" {
                puzzle()?
            } else if path == "/api" || path.starts_with("/api/") {
                not_found_json()?
            } else {
                not_found_page(&req, &env).await?
            }
        }
        _ => Response::empty()?.with_status(405),
    };
    // One place for CORS, so 404s and 405s get the headers too.
    with_cors(resp)
}
