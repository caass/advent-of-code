use clap::Parser;
use clap_stdin::FileOrStdin;
use eyre::Result;

use advent_of_code::meta::{Day, Part, Year};
use advent_of_code::AOC;

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

    let solution = AOC[year][day][part].solve(input.contents()?.trim())?;
    println!("{}", solution);

    Ok(())
}
