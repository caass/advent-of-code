use std::str::FromStr;

use bitvec::prelude::*;
use eyre::{Report, Result, eyre};

use aoc_meta::Problem;

pub const DRAGON_CHECKSUM: Problem = Problem::solved(
    &|input| {
        input
            .parse::<Data>()
            .map(|data| data.checksum_for_size(272))
    },
    &|input| {
        input
            .parse::<Data>()
            .map(|data| data.checksum_for_size(35651584))
    },
);

#[derive(Debug, Clone)]
struct Data(BitVec<usize, Msb0>);

impl Data {
    fn checksum_for_size(self, n: usize) -> String {
        let mut a = self.0;
        while a.len() < n {
            let mut b = a.clone();
            b.reverse();
            b = !b;

            a.push(false);
            a.extend_from_bitslice(&b);
            b.clear();
        }

        let mut checksum = a[..n]
            .chunks_exact(2)
            .map(|chunk| chunk[0] == chunk[1])
            .collect::<BitVec<usize, Msb0>>();

        while checksum.len().is_multiple_of(2) {
            checksum = checksum
                .chunks_exact(2)
                .map(|chunk| chunk[0] == chunk[1])
                .collect();
        }

        checksum
            .into_iter()
            .map(|bit| if bit { '1' } else { '0' })
            .collect()
    }
}

impl FromStr for Data {
    type Err = Report;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        s.chars()
            .map(|ch| match ch {
                '0' => Ok(false),
                '1' => Ok(true),
                other => Err(eyre!("invalid binary character {other}")),
            })
            .collect()
    }
}

impl<I> FromIterator<I> for Data
where
    BitVec<usize, Msb0>: FromIterator<I>,
{
    fn from_iter<T: IntoIterator<Item = I>>(iter: T) -> Self {
        Self(BitVec::from_iter(iter))
    }
}
