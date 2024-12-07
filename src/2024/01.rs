use eyre::{eyre, OptionExt, Report, Result};
use rayon::prelude::*;

use crate::meta::Problem;

pub const HISTORIAN_HYSTERIA: Problem = Problem::partially_solved(&|input| -> Result<usize> {
    let (lhs, rhs) = input
        .lines()
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
        .try_fold((Vec::new(), Vec::new()), |(mut lhs, mut rhs), res| {
            let (a, b) = res?;

            lhs.push(a);
            rhs.push(b);

            Ok::<_, Report>((lhs, rhs))
        })
        .map(|(mut lhs, mut rhs)| {
            lhs.sort_unstable();
            rhs.sort_unstable();

            (lhs, rhs)
        })?;

    Ok(lhs
        .into_par_iter()
        .zip(rhs)
        .map(|(a, b)| a.abs_diff(b))
        .sum())
});
