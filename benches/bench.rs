use std::fs;
use std::path::{Path, PathBuf};
use std::sync::LazyLock;

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use eyre::{Context, Result};

use advent_of_code::meta::{Day, Year};
use advent_of_code::AOC;

static FIXTURES_DIR: LazyLock<PathBuf> = LazyLock::new(|| {
    let crate_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    crate_dir.join("tests").join("inputs")
});

fn read_input(year: Year, day: Day) -> Result<String> {
    let path = FIXTURES_DIR.join(year).join(day);
    fs::read_to_string(&path)
        .wrap_err_with(|| format!("While attempting to read {}", path.display()))
}

pub fn aoc(c: &mut Criterion) {
    AOC.years()
        .flat_map(|(year, problem_set)| {
            problem_set
                .days()
                .map(move |(day, problem)| (year, day, problem))
        })
        .for_each(|(year, day, problem)| {
            let mut group = c.benchmark_group(format!("{year}-{day}"));

            let input = read_input(year, day).unwrap();
            problem.parts().for_each(|(part, solution)| {
                group.bench_function(part.to_string(), |b| {
                    b.iter(|| solution.solve(black_box(&input)))
                });
            });

            group.finish();
        });
}

criterion_group!(benches, aoc);
criterion_main!(benches);
