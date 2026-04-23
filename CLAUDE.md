# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Rules of the Game

Implementing puzzle solutions is strictly off-limits for LLMs and AI agents — that defeats the purpose of the challenge. All LLM assistance must be purely explanatory (e.g. clarifying algorithmic concepts, explaining language features, or discussing approaches). Non-solution code such as `./x.py` and other tooling is fine to write or modify.

## Commands

The primary interface is `./x.py`, a Python CLI (runs via `uv`). Inputs are auto-downloaded and decrypted by `x.py` when needed.

```bash
# Run a solution
./x.py run <YEAR> <DAY> <PART>

# Test all solutions
./x.py test

# Test a specific year
./x.py test <YEAR>

# Test a specific day
./x.py test <YEAR> <DAY>

# Build
cargo build --release

# Lint Rust
cargo clippy

# Lint Python
ruff check x/
```

Tests use `cargo nextest` (via `./x.py test`) with the `fast-test` profile (optimized incremental builds). The `completion` and `ci` nextest profiles in `.config/nextest.toml` are for generating JUnit XML and CI use respectively.

## Architecture

This is a Cargo workspace where each year's solutions live in their own crate (`aoc-2015` through `aoc-2025`). The `aoc` crate is the runnable binary that wires everything together.

**Core types (in `aoc-meta`):**

- `Solution` trait — any `Fn(&str) -> R` where `R: ReturnValue` (i.e. `usize`, `isize`, or `Result<T, E>`) automatically implements it. Input is trimmed of trailing whitespace before being passed.
- `Problem` — holds up to two `&'static dyn Solution` references (one per part). Constructed via `Problem::solved(part1_fn, part2_fn)`, `Problem::partially_solved(part1_fn)`, or `Problem::unsolved()`.
- `ProblemSet` — array of 25 `Option<Problem>` entries, one per day.
- `AdventOfCode` — top-level registry mapping `Year` → `ProblemSet`.

**Adding a new solution:**

1. Create `aoc-<YEAR>/src/<DD>.rs` exporting a `pub const MY_PROBLEM: Problem = Problem::solved(&part1, &part2)`.
2. Register it in `aoc-<YEAR>/src/lib.rs` via the `PROBLEMS!` macro:

   ```rust
   PROBLEMS! {
       01 => MY_PROBLEM,
       // ...
   }
   ```

3. Add expected answers to `aoc-<YEAR>/tests/<YEAR>.rs` using the `aoc_test::tests!` macro:

   ```rust
   aoc_test::tests!(2025, {
       1: [part1_answer, part2_answer],
   });
   ```

   Omit the second answer if only part 1 is solved.

**`x/` Python CLI** (managed by `uv`, entry point `x.py`):

- `x/run.py` — invokes `cargo run --release --package aoc -- YEAR DAY PART <input_path>` with `RUSTFLAGS=-C target-cpu=native`
- `x/test.py` — invokes `cargo nextest run --cargo-profile=fast-test`; filters by package (`aoc-<YEAR>`) and test name (`day<N>::`)
- `x/inputs/` — handles downloading, encrypting, and decrypting puzzle inputs

**`aoc-common`** provides shared utilities used across multiple years: `Grid<N, T>` (2D square grid with rayon parallel iterators), `TryFromStr` (zero-copy string parsing), and `BoolExt`.
