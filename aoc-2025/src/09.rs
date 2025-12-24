use std::str::FromStr;

use aoc_meta::Problem;
use eyre::{OptionExt, Report, Result, eyre};
use itertools::Itertools;
use rayon::prelude::*;

pub const MOVIE_THEATER: Problem = Problem::partially_solved(&largest_rectangle);

fn largest_rectangle(input: &str) -> Result<u64> {
    let coords: Vec<_> = input.lines().map(Coordinate::from_str).try_collect()?;

    coords
        .par_iter()
        .copied()
        .enumerate()
        .flat_map(|(i, a)| coords[i + 1..].par_iter().copied().map(move |b| (a, b)))
        .map(|(a, b)| a.area(b))
        .max()
        .ok_or_eyre("no red tiles!")
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct Coordinate {
    x: u32,
    y: u32,
}

impl Coordinate {
    fn area(self, other: Coordinate) -> u64 {
        (u64::from(self.x.abs_diff(other.x)) + 1) * (u64::from(self.y.abs_diff(other.y)) + 1)
    }
}

impl FromStr for Coordinate {
    type Err = Report;

    fn from_str(s: &str) -> Result<Self> {
        let (x_str, y_str) = s
            .split_once(',')
            .ok_or_else(|| eyre!("failed to parse line \"{s}\""))?;

        let x = x_str.parse()?;
        let y = y_str.parse()?;

        Ok(Self { x, y })
    }
}

#[test]
fn example() {
    use pretty_assertions::assert_eq;

    let input = "7,1
11,1
11,7
9,7
9,5
2,5
2,3
7,3
";

    assert_eq!(largest_rectangle(input).unwrap(), 50);
}
