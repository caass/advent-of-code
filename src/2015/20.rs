use eyre::OptionExt;
use rayon::prelude::*;

use crate::common::U32_MAX;
use crate::meta::Problem;

/// <https://adventofcode.com/2015/day/20>
pub const INFINITE_ELVES_AND_INFINITE_HOUSES: Problem = Problem::solved(
    &|input| {
        let n = input.trim().parse::<usize>()?;
        (0..U32_MAX)
            .into_par_iter()
            .map(|address| House { address })
            .by_exponential_blocks()
            .find_first(|house| {
                house
                    .presents_with_infinite_visitors()
                    .is_some_and(|p| p >= n)
            })
            .ok_or_eyre("no houses got enough presents")
            .map(|house| house.address)
    },
    &|input| {
        let n = input.trim().parse::<usize>()?;
        (0..U32_MAX)
            .into_par_iter()
            .map(|address| House { address })
            .by_exponential_blocks()
            .find_first(|house| {
                house
                    .presents_with_finite_visitors()
                    .is_some_and(|p| p >= n)
            })
            .ok_or_eyre("no houses got enough presents")
            .map(|house| house.address)
    },
);

// the number of presents a house gets is (10 * factor for each factor of that houses address)

#[derive(Debug)]
struct House {
    address: usize,
}

impl House {
    fn presents_with_infinite_visitors(&self) -> Option<usize> {
        // use `checked_mul` because if we're getting out of `usize` range, we're already too high.
        self.address
            .factors()
            .map(|n| n.checked_mul(10))
            .try_fold(1usize, |a, opt| opt.and_then(move |b| a.checked_add(b)))
    }

    fn presents_with_finite_visitors(&self) -> Option<usize> {
        self.address
            .factors()
            .filter(|&fac| self.address / fac < 50)
            .map(|n| n.checked_mul(11))
            .try_fold(1usize, |a, opt| opt.and_then(move |b| a.checked_add(b)))
    }
}

trait Factorable {
    fn factors(&self) -> Factors;
}

impl Factorable for usize {
    fn factors(&self) -> Factors {
        Factors::of(*self)
    }
}

struct Factors {
    of: usize,
    limit: usize,
    current: usize,
    extra: Option<usize>,
}

impl Factors {
    fn of(n: usize) -> Self {
        // limit is sqrt of `n`, since sqrt(`n`) * sqrt(`n`) == of
        #[allow(
            clippy::cast_sign_loss,
            clippy::cast_precision_loss,
            clippy::cast_possible_truncation,
            reason = "clippy has a lot of complaints about this line, but they're mostly overblown:
                - cast_sign_loss: we're guaranteed to have a positive number
                - cast_precision_loss: the 51 bits of the mantissa in `n` can fit in the 64 bits of `usize`
                - cast_possible_truncation: the square root of `usize` always fits in a `usize`"
        )]
        let limit = (n as f64).sqrt().trunc() as usize;

        Self {
            of: n,
            limit,
            current: 0,
            extra: None,
        }
    }
}

impl Iterator for Factors {
    type Item = usize;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(other_factor) = self.extra.take() {
            return Some(other_factor);
        }

        while self.current < self.limit {
            self.current += 1;

            let a = self.of / self.current;
            let b = self.of / a;

            if a * b == self.of {
                if a != b {
                    self.extra = Some(b);
                }

                return Some(a);
            }
        }

        None
    }
}
