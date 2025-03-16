use clap::Parser;
use clap_stdin::FileOrStdin;
use eyre::{Result, eyre};

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

    let solution = AOC
        .year(year)
        .ok_or_else(|| eyre!("haven't solved year {year} yet"))?
        .day(day)
        .ok_or_else(|| eyre!("haven't solved day {day} of {year} yet"))?
        .part(part)
        .ok_or_else(|| eyre!("haven't solved part {part} of day {day} of {year} yet"))?
        .solve(input.contents()?.trim())?;

    println!("{}", solution);

    Ok(())
}
