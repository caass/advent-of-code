use eyre::{OptionExt, bail};
use rayon::prelude::*;

use aoc_meta::Problem;

pub const GIFT_SHOP: Problem = Problem::partially_solved(&|input| {
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

                let this_sum = range
                    .into_par_iter()
                    .filter(|&n| {
                        let mut buf = itoa::Buffer::new();

                        let s = buf.format(n);
                        let midpoint = s.len() / 2;

                        s[..midpoint] == s[midpoint..]
                    })
                    .sum::<u64>();

                total_sum
                    .checked_add(this_sum)
                    .ok_or_eyre("attempt to add with overflow")
            },
        )
        .try_reduce(
            || 0,
            |a, b| a.checked_add(b).ok_or_eyre("attempt to add with overflow"),
        )
});
