use std::str::FromStr;

use eyre::{bail, OptionExt, Report, Result};
use rayon::prelude::*;

use crate::meta::Problem;

pub const PROBLEM: Problem = Problem::solved(
    &|input| {
        input
            .par_lines()
            .map(|line| line.parse().map(Dimensions::wrapping_paper))
            .try_reduce(|| 0, |a, b| Ok(a + b))
            .map(|n| n.to_string())
    },
    &|input| {
        input
            .par_lines()
            .map(|line| line.parse().map(Dimensions::ribbon))
            .try_reduce(|| 0, |a, b| Ok(a + b))
            .map(|n| n.to_string())
    },
);

struct Dimensions([usize; 3]);

impl Dimensions {
    /// Returns the amount of wrapping paper needed to wrap a present with dimensions `self`:
    /// enough to cover all six sides plus extra equal to the area of the smallest side.
    #[inline(always)]
    fn wrapping_paper(self) -> usize {
        let Dimensions([l, w, h]) = self;

        let a = l * w;
        let b = w * h;
        let c = h * l;

        2 * a + 2 * b + 2 * c + a.min(b).min(c)
    }

    /// Returns the amount of ribbon needed to tie off a present with dimensions `self`:
    /// enough to wrap around the smallest size, plus bow-material equal to the volume of the present.
    #[inline(always)]
    fn ribbon(self) -> usize {
        let Dimensions([l, w, h]) = self;

        let volume = l * w * h;

        let a = 2 * (l + w);
        let b = 2 * (w + h);
        let c = 2 * (h + l);
        let smallest_side = a.min(b).min(c);

        smallest_side + volume
    }
}

impl FromStr for Dimensions {
    type Err = Report;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut split = s.split('x');
        let a = split.next().ok_or_eyre("No length")?.parse()?;
        let b = split.next().ok_or_eyre("No width")?.parse()?;
        let c = split.next().ok_or_eyre("No height")?.parse()?;
        if let Some(other) = split.next() {
            bail!("Unexpected extra split: {other}");
        };

        Ok(Self([a, b, c]))
    }
}
