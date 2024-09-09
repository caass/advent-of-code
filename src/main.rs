use std::io::{self, stdin, IsTerminal, Read};

use thiserror::Error;

use advent_of_code::types::{Day, ParseDayErr, ParsePartError, ParseYearError, Part, Year};
use advent_of_code::AOC;

#[derive(Debug, Error)]
pub enum ParseError {
    #[error(transparent)]
    Year(#[from] ParseYearError),
    #[error(transparent)]
    Day(#[from] ParseDayErr),
    #[error(transparent)]
    Part(#[from] ParsePartError),

    #[error(transparent)]
    Io(#[from] io::Error),

    #[error("usage: cat <FILE> | aoc <YEAR> <DAY> <PART>")]
    NotEnoughArgs,
}

fn main() -> Result<(), ParseError> {
    let mut args = std::env::args();

    let _exe_name = args.next().ok_or(ParseError::NotEnoughArgs)?;
    let year: Year = args.next().ok_or(ParseError::NotEnoughArgs)?.parse()?;
    let day: Day = args.next().ok_or(ParseError::NotEnoughArgs)?.parse()?;
    let part: Part = args.next().ok_or(ParseError::NotEnoughArgs)?.parse()?;

    let input = args.next().map(Ok).unwrap_or_else(|| {
        if stdin().is_terminal() {
            Err(ParseError::NotEnoughArgs)
        } else {
            let mut buf = String::new();
            stdin().read_to_string(&mut buf)?;
            Ok(buf)
        }
    })?;

    let f = AOC[year][day]
        .as_ref()
        .and_then(|problem| problem[part])
        .unwrap_or_else(|| todo!("Haven't yet solved part {part} of {year}'s day {day}"));

    let output = (f)(input.trim());
    println!("{}", output);

    Ok(())
}
