use std::io::{stdin, IsTerminal, Read};

use eyre::{OptionExt, Report};
use thiserror::Error;

use advent_of_code::types::{Day, Part, Year};
use advent_of_code::AOC;

#[derive(Debug, Error)]
#[error("usage: cat <FILE> | aoc <YEAR> <DAY> <PART>")]
pub struct ParseError;
fn main() -> eyre::Result<()> {
    let mut args = std::env::args();

    let _exe_name = args.next().ok_or(ParseError)?;
    let year: Year = args.next().ok_or(ParseError)?.parse()?;
    let day: Day = args.next().ok_or(ParseError)?.parse()?;
    let part: Part = args.next().ok_or(ParseError)?.parse()?;

    let input = args.next().map(Ok::<_, Report>).unwrap_or_else(|| {
        if stdin().is_terminal() {
            Err(ParseError.into())
        } else {
            let mut buf = String::new();
            stdin().read_to_string(&mut buf)?;
            Ok(buf)
        }
    })?;

    let f = AOC
        .get(year, day, part)
        .ok_or_eyre("haven't solved part {part} of day {day} of {year} yet")?;

    let output = (f)(input.trim());
    println!("{}", output);

    Ok(())
}
