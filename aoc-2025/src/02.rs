use eyre::{OptionExt, bail};
use itertools::Itertools;
use rayon::prelude::*;

use aoc_meta::Problem;

pub const GIFT_SHOP: Problem = Problem::solved(
    &|input| count_invalid_ids(input, doubled_numbers),
    &|input| count_invalid_ids(input, repeated_numbers),
);

fn count_invalid_ids<F>(input: &str, f: F) -> Result<u64, eyre::Error>
where
    F: Fn(u64) -> bool + Send + Sync,
{
    input
        .par_split(',')
        .map(|s| {
            let Some((a, b)) = s.split_once('-') else {
                bail!("unable to parse range {s}");
            };

            let start = a.parse::<u64>()?;
            let end = b.parse::<u64>()?;

            Ok(start..=end)
        })
        .try_fold(
            || 0u64,
            |total_sum, res| {
                let range = res?;

                // TODO: this is slow, we shouldn't have to check every item. But i'm not gonna sweat it for now
                let this_sum = range.into_par_iter().filter(|&n| f(n)).sum::<u64>();

                total_sum
                    .checked_add(this_sum)
                    .ok_or_eyre("attempt to add with overflow")
            },
        )
        .try_reduce(
            || 0,
            |a, b| a.checked_add(b).ok_or_eyre("attempt to add with overflow"),
        )
}

fn doubled_numbers(n: u64) -> bool {
    let mut buf = itoa::Buffer::new();

    let s = buf.format(n);
    let midpoint = s.len() / 2;

    s[..midpoint] == s[midpoint..]
}

fn repeated_numbers(n: u64) -> bool {
    let mut buf = itoa::Buffer::new();

    let s = buf.format(n);
    let midpoint = s.len() / 2;

    (1..=midpoint).any(|i| {
        let mut chunks = s.as_bytes().chunks_exact(i);
        chunks.remainder().is_empty() && chunks.all_equal()
    })
}
