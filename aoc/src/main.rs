use clap::Parser;
use clap_stdin::FileOrStdin;
use eyre::Result;

use aoc::AOC;
use aoc_meta::{Day, Part, Year};

#[derive(Debug, Parser)]
struct Args {
    /// The year of Advent of Code the problem is in
    year: Year,

    /// The day (1-indexed) to solve
    day: Day,

    /// The part of the puzzle to solve.
    part: Part,

    /// File containing puzzle input (can also be read from STDIN).
    input: FileOrStdin,
}

fn main() -> Result<()> {
    let Args {
        year,
        day,
        part,
        input,
    } = Args::parse();

    let output = AOC[year][day][part].solve(&input.contents()?)?;

    println!("{}", output);

    Ok(())
}
