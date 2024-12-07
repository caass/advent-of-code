use std::str::FromStr;

use eyre::{eyre, OptionExt, Report, Result};
use nohash_hasher::IntMap;
use rayon::prelude::*;

use crate::meta::Problem;

pub const HISTORIAN_HYSTERIA: Problem = Problem::solved(
    &|input| input.parse().map(LocationIds::total_distance),
    &|input| input.parse().map(LocationIds::total_similarity),
);

#[derive(Debug, Default)]
struct LocationIds {
    lhs: Vec<usize>,
    rhs: Vec<usize>,
}

impl LocationIds {
    fn total_distance(self) -> usize {
        let LocationIds { mut lhs, mut rhs } = self;

        lhs.sort_unstable();
        rhs.sort_unstable();

        lhs.into_par_iter()
            .zip(rhs)
            .map(|(a, b)| a.abs_diff(b))
            .sum()
    }

    fn total_similarity(self) -> usize {
        let LocationIds { lhs, rhs } = self;
        let mut cache = IntMap::default();

        lhs.into_iter()
            .map(|id| {
                *cache
                    .entry(id)
                    .or_insert_with(|| rhs.par_iter().filter(|rhs_id| **rhs_id == id).count() * id)
            })
            .sum()
    }
}

impl FromStr for LocationIds {
    type Err = Report;

    fn from_str(s: &str) -> Result<Self> {
        s.lines()
            .map(|line| {
                let mut iter = line.split_ascii_whitespace();
                let a_str = iter.next().ok_or_eyre("empty line")?;
                let b_str = iter
                    .next()
                    .ok_or_else(|| eyre!("No whitespace on line: \"{line}\""))?;

                debug_assert!(iter.next().is_none());

                let a = a_str.parse()?;
                let b = b_str.parse()?;

                Ok::<(usize, usize), Report>((a, b))
            })
            .try_fold(LocationIds::default(), |mut ids, res| {
                let (a, b) = res?;

                ids.lhs.push(a);
                ids.rhs.push(b);

                Ok::<_, Report>(ids)
            })
    }
}
