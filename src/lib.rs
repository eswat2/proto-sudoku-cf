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

fn greet(name: Option<&String>) -> Result<Response> {
    let name = name.map(String::as_str).unwrap_or("World");
    Response::from_json(&json!({ "hello": name }))
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
    let resp = if req.method() == Method::Options {
        Response::empty()?
    } else {
        Router::new()
            .get("/api/puzzle", |_, _| puzzle())
            .get("/api/:name", |_, ctx| greet(ctx.param("name")))
            .get("/:name", |_, ctx| greet(ctx.param("name")))
            .run(req, env)
            .await?
    };
    // One place for CORS, so 404s from the router get the headers too.
    with_cors(resp)
}
