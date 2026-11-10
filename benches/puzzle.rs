//! Native timings for puzzle generation and solving.
//! Workers can't measure these (its clock is frozen while code runs), so
//! run `cargo bench` locally instead. Native is typically 1.5-3x faster
//! than the wasm32 build that Workers runs.

use criterion::{criterion_group, criterion_main, BatchSize, Criterion};
use rust_sudoku_api::new_puzzle;
use std::hint::black_box;
use sudoku::Sudoku;

fn bench(c: &mut Criterion) {
    c.bench_function("generate", |b| b.iter(|| black_box(Sudoku::generate())));

    // A fresh puzzle per iteration, so the average covers a spread of
    // difficulties; only the solve is timed.
    c.bench_function("solve", |b| {
        b.iter_batched(
            Sudoku::generate,
            |s| black_box(s.solution()),
            BatchSize::SmallInput,
        )
    });

    // The whole /api/puzzle path minus JSON and HTTP.
    c.bench_function("new_puzzle", |b| b.iter(|| black_box(new_puzzle())));
}

criterion_group!(benches, bench);
criterion_main!(benches);
