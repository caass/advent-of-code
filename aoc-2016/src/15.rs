use std::str::FromStr;

use eyre::{OptionExt, Report, eyre};

use aoc_meta::Problem;
use rayon::prelude::*;
use regex::Regex;

pub const TIMING_IS_EVERYTHING: Problem = Problem::solved(
    &|input| {
        let s: Sculpture = input.parse()?;
        s.time_to_press().ok_or_eyre("parts never line up")
    },
    &|input| {
        let mut s: Sculpture = input.parse()?;
        s.discs.push(Disc::new(0, 11));
        s.time_to_press().ok_or_eyre("parts never line up")
    },
);

#[derive(Debug, Clone)]
struct Sculpture {
    discs: Vec<Disc>,
}

impl Sculpture {
    fn time_to_press(&self) -> Option<usize> {
        (0..usize::MAX)
            .into_par_iter()
            .by_exponential_blocks()
            .find_first(|&n| {
                self.discs
                    .iter()
                    .enumerate()
                    .all(|(i, d)| d.after(n + i + 1) == 0)
            })
    }
}

impl FromStr for Sculpture {
    type Err = Report;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let re = Regex::new(r"Disc #\d+ has (\d+) positions; at time=0, it is at position (\d+).")?;

        s.lines()
            .enumerate()
            .map(|(i, line)| {
                let caps = re
                    .captures(line)
                    .ok_or_else(|| eyre!("line {} didn't match regex", i + 1))?;
                let num_positions = caps
                    .get(1)
                    .ok_or_else(|| eyre!("no match for # of positions in line {}", i + 1))?
                    .as_str()
                    .parse()?;
                let start = caps
                    .get(2)
                    .ok_or_else(|| eyre!("no match for start location in line {}", i + 1))?
                    .as_str()
                    .parse()?;

                Ok(Disc::new(start, num_positions))
            })
            .collect()
    }
}

impl FromIterator<Disc> for Sculpture {
    fn from_iter<T: IntoIterator<Item = Disc>>(iter: T) -> Self {
        Self {
            discs: Vec::from_iter(iter),
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct Disc {
    current_position: usize,
    num_positions: usize,
}

impl Disc {
    fn new(start: usize, num_positions: usize) -> Self {
        Self {
            current_position: start,
            num_positions,
        }
    }

    fn after(&self, ticks: usize) -> usize {
        (self.current_position + ticks) % self.num_positions
    }
}

#[test]
fn example() {
    let s: Sculpture = "Disc #1 has 5 positions; at time=0, it is at position 4.
Disc #2 has 2 positions; at time=0, it is at position 1."
        .parse()
        .unwrap();

    assert_eq!(s.time_to_press().unwrap(), 5)
}
