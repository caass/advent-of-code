use std::{cmp::Ordering, num::ParseIntError, ops::RangeInclusive, str::FromStr};

use eyre::{Report, Result, bail};

use aoc_meta::Problem;
use rayon::prelude::*;

pub const CAFETERIA: Problem = Problem::solved(
    &|input| input.parse().map(Ingredients::kitchen_fresh),
    &|input| input.parse().map(Ingredients::system_fresh),
);

#[derive(Debug)]
struct Ingredients {
    fresh: Vec<RangeInclusive<u64>>,
    available: Vec<u64>,
}

impl Ingredients {
    /// Count the total number of fresh ingredients in the inventory system.
    fn system_fresh(self) -> u64 {
        self.fresh
            .into_par_iter()
            .map(|r| *r.end() - *r.start() + 1)
            .sum::<u64>()
    }

    /// Count the total number of fresh ingredients in the kitchen.
    fn kitchen_fresh(self) -> usize {
        self.available
            .par_iter()
            .copied()
            .filter(|&id| self.is_fresh(id))
            .count()
    }

    fn is_fresh(&self, id: u64) -> bool {
        self.fresh
            .binary_search_by(|range| match (*range.start() <= id, *range.end() >= id) {
                (true, true) => Ordering::Equal,
                (true, false) => Ordering::Less,
                (false, true) => Ordering::Greater,
                (false, false) => {
                    unreachable!("range cannot be both greater than and less than id")
                }
            })
            .is_ok()
    }
}

impl FromStr for Ingredients {
    type Err = Report;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let Some((fresh, available)) = s.split_once("\n\n") else {
            bail!(
                "unable to determine where fresh ingredients end and available ingredients start"
            );
        };

        let (fresh_res, available_res) = rayon::join(
            || {
                fresh
                    .par_lines()
                    .map(|line| {
                        let Some((start, end)) = line.split_once('-') else {
                            bail!("unable to split ingredient range")
                        };

                        Ok(start.parse()?..=end.parse()?)
                    })
                    .collect::<Result<Vec<RangeInclusive<u64>>>>()
            },
            || {
                available
                    .par_lines()
                    .map(u64::from_str)
                    .collect::<Result<Vec<u64>, ParseIntError>>()
            },
        );

        let mut fresh = fresh_res?;
        fresh.par_sort_unstable_by_key(|r| *r.start());

        // Collapse overlapping/adjacent ranges
        let fresh = fresh
            .into_iter()
            .fold(Vec::<RangeInclusive<u64>>::new(), |mut acc, range| {
                if let Some(last) = acc.last_mut()
                    && *last.end() + 1 >= *range.start()
                {
                    *last = *last.start()..=(*last.end()).max(*range.end());
                    return acc;
                }
                acc.push(range);
                acc
            });

        let mut available = available_res?;
        available.sort();

        Ok(Self { fresh, available })
    }
}
