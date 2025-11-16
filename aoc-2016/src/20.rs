use std::str::FromStr;

use eyre::{OptionExt, Report, Result};
use itertools::Itertools;
use rayon::prelude::*;

use aoc_meta::Problem;

pub const FIREWALL_RULES: Problem = Problem::solved(
    &|input| {
        let fw = input.parse::<Firewall>()?;
        fw.lowest_allowed_ip().ok_or_eyre("no IPs allowed!")
    },
    &|input| input.parse().map(Firewall::count_allowed_ips),
);

#[derive(Debug, Clone, Copy)]
struct IpRange {
    start: u32,
    end: u32,
}

impl FromStr for IpRange {
    type Err = Report;

    fn from_str(s: &str) -> Result<Self> {
        let parts = s.split_once('-').ok_or_eyre("no '-' in input")?;
        let start = parts.0.parse()?;
        let end = parts.1.parse()?;

        Ok(IpRange { start, end })
    }
}

#[derive(Debug, Clone)]
struct Firewall {
    inner: Vec<IpRange>,
}

impl Firewall {
    fn first(&self) -> Option<IpRange> {
        self.inner.first().copied()
    }

    fn lowest_allowed_ip(self) -> Option<u32> {
        if self.first().is_none_or(|range| range.start > 0) {
            return Some(0);
        }

        self.into_iter()
            .next()
            .expect("at least one firewall after first() check")
            .end
            .checked_add(1)
    }

    fn count_allowed_ips(self) -> u32 {
        let Some(k) = self.first().map(|r| r.start) else {
            return u32::MAX;
        };

        self.into_iter()
            .tuple_windows()
            .fold(k, |n, (a, b)| n + b.start - a.end.saturating_add(1))
    }
}

impl FromStr for Firewall {
    type Err = Report;

    fn from_str(s: &str) -> Result<Self> {
        let mut ranges = s
            .par_lines()
            .map(IpRange::from_str)
            .collect::<Result<Vec<_>>>()?;

        ranges.par_sort_unstable_by_key(|range| range.start);

        Ok(Self { inner: ranges })
    }
}

impl IntoIterator for Firewall {
    type IntoIter = MergedFirewall;
    type Item = IpRange;

    fn into_iter(self) -> Self::IntoIter {
        MergedFirewall {
            range: None,
            inner: self.inner.into_iter(),
        }
    }
}

struct MergedFirewall {
    range: Option<IpRange>,
    inner: std::vec::IntoIter<IpRange>,
}

impl Iterator for MergedFirewall {
    type Item = IpRange;

    fn next(&mut self) -> Option<Self::Item> {
        let mut old = self.range.or_else(|| self.inner.next())?;
        let mut new = None;

        for range in self.inner.by_ref() {
            if old.end.saturating_add(1) < range.start {
                new = Some(range);
                break;
            }

            let start = u32::min(old.start, range.start);
            let end = u32::max(old.end, range.end);
            old = IpRange { start, end }
        }

        self.range = new;
        Some(old)
    }
}
