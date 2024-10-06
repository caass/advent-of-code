use eyre::OptionExt;
use rayon::prelude::*;

use crate::meta::{problem, Problem};

pub const INFINITE_ELVES_AND_INFINITE_HOUSES: Problem = problem!(
    |input: &str| {
        let n = input.trim().parse::<u32>()?;
        (0..=u32::MAX)
            .into_par_iter()
            .map(|address| House { address })
            .find_first(|house| house.presents_with_infinite_visitors() >= n)
            .ok_or_eyre("no houses got enough presents")
            .map(|house| house.address)
    },
    |input: &str| {
        let n = input.trim().parse::<u32>()?;
        (0..=u32::MAX)
            .into_par_iter()
            .map(|address| House { address })
            .find_first(|house| house.presents_with_finite_visitors() >= n)
            .ok_or_eyre("no houses got enough presents")
            .map(|house| house.address)
    }
);

// the number of presents a house gets is (10 * factor for each factor of that houses address)

#[derive(Debug)]
struct House {
    address: u32,
}

impl House {
    fn presents_with_infinite_visitors(&self) -> u32 {
        self.address.factors().map(|n| n * 10).sum()
    }

    fn presents_with_finite_visitors(&self) -> u32 {
        self.address
            .factors()
            .filter(|&fac| self.address / fac < 50)
            .map(|n| n * 11)
            .sum()
    }
}

trait Factorable {
    fn factors(&self) -> Factors;
}

impl Factorable for u32 {
    fn factors(&self) -> Factors {
        Factors::of(*self)
    }
}

struct Factors {
    of: u32,
    limit: u32,
    current: u32,
    extra: Option<u32>,
}

impl Factors {
    fn of(n: u32) -> Self {
        // limit is sqrt of `n`, since sqrt(`n`) * sqrt(`n`) == of
        let limit = (n as f64).sqrt().trunc() as u32;

        Self {
            of: n,
            limit,
            current: 0,
            extra: None,
        }
    }
}

impl Iterator for Factors {
    type Item = u32;

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
