use std::io::{stdin, IsTerminal, Read};

use clap::Parser;
use eyre::{bail, eyre, Result};

use advent_of_code::meta::{Day, Part, Year};
use advent_of_code::AOC;

#[derive(Debug, Parser)]
struct AocArgs {
    year: Year,
    day: Day,
    part: Part,
}

fn main() -> Result<()> {
    let AocArgs { year, day, part } = AocArgs::parse();

    let input = if stdin().is_terminal() {
        bail!("Expected input on stdin, try `cargo run --release -- {year} {day} {part} < path/to/input");
    } else {
        let mut buf = String::new();
        stdin().read_to_string(&mut buf)?;
        buf
    };

    let f = AOC
        .get(year, day, part)
        .ok_or_else(|| eyre!("haven't solved part {part} of day {day} of {year} yet"))?;

    let output = f.solve(input.trim())?;
    println!("{}", output);

    Ok(())
}
