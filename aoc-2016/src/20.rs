use std::{ops::RangeInclusive, str::FromStr};

use eyre::{OptionExt, Report, Result};
use itertools::Itertools;
use rayon::prelude::*;

use aoc_meta::Problem;

pub const FIREWALL_RULES: Problem = Problem::solved(
    &|input| input.parse().and_then(Firewall::lowest_allowed_ip),
    &|input| input.parse().map(Firewall::count_allowed_ips),
);

#[derive(Debug, Clone)]
struct Firewall {
    inner: Vec<RangeInclusive<u32>>,
}

impl Firewall {
    fn first(&self) -> Option<RangeInclusive<u32>> {
        self.inner.first().cloned()
    }

    fn lowest_allowed_ip(self) -> Result<u32> {
        if self.first().is_none_or(|range| *range.start() > 0) {
            return Ok(0);
        }

        self.into_iter()
            .next()
            .ok_or_eyre("empty firewall?")?
            .end()
            .checked_add(1)
            .ok_or_eyre("no allowed ips")
    }

    fn count_allowed_ips(self) -> u32 {
        let Some(allowed_count) = self.first().map(|r| *r.start()) else {
            return u32::MAX;
        };

        self.into_iter()
            .tuple_windows()
            .fold(allowed_count, |n, (a, b)| {
                n + b.start() - a.end().saturating_add(1)
            })
    }
}

impl FromStr for Firewall {
    type Err = Report;

    fn from_str(s: &str) -> Result<Self> {
        let mut ranges = s
            .par_lines()
            .map(|line| {
                let parts = line.split_once('-').ok_or_eyre("no '-' in input")?;
                let start = parts.0.parse()?;
                let end = parts.1.parse()?;

                Ok(start..=end)
            })
            .collect::<Result<Vec<RangeInclusive<u32>>>>()?;

        ranges.par_sort_unstable_by_key(|range| *range.start());

        Ok(Self { inner: ranges })
    }
}

impl IntoIterator for Firewall {
    type IntoIter = MergedFirewall;
    type Item = RangeInclusive<u32>;

    fn into_iter(self) -> Self::IntoIter {
        MergedFirewall {
            range: None,
            inner: self.inner.into_iter(),
        }
    }
}

struct MergedFirewall {
    range: Option<RangeInclusive<u32>>,
    inner: std::vec::IntoIter<RangeInclusive<u32>>,
}

impl Iterator for MergedFirewall {
    type Item = RangeInclusive<u32>;

    fn next(&mut self) -> Option<Self::Item> {
        let mut old = self.range.clone().or_else(|| self.inner.next())?;
        let mut new = None;

        for range in self.inner.by_ref() {
            if old.end().saturating_add(1) < *range.start() {
                new = Some(range);
                break;
            }

            let start = u32::min(*old.start(), *range.start());
            let end = u32::max(*old.end(), *range.end());
            old = start..=end;
        }

        self.range = new;
        Some(old)
    }
}
