use eyre::{OptionExt, Result};
use md5::{digest::Output, Digest, Md5};
use rayon::prelude::*;

use crate::meta::problem;

problem!(part_1, part_2);

fn part_1(input: &str) -> Result<usize> {
    find(input, |hash| hash[0] == 0 && hash[1] == 0 && hash[2] < 0x10)
        .ok_or_eyre(format!("No hashes in u{} start with 5 zeros", usize::BITS))
}

fn part_2(input: &str) -> Result<usize> {
    find(input, |hash| hash[0] == 0 && hash[1] == 0 && hash[2] == 0)
        .ok_or_eyre(format!("no hashes in u{} start with 6 zeros", usize::BITS))
}

#[inline(always)]
fn find<F: Sync + Fn(Output<Md5>) -> bool>(input: &str, f: F) -> Option<usize> {
    let mut base = Md5::new();
    base.update(input);

    (0..=usize::MAX).into_par_iter().find_first(|&n| {
        let mut buf = itoa::Buffer::new();
        let slice = buf.format(n);

        let mut hasher: Md5 = Md5::clone(&base);
        Digest::update(&mut hasher, slice);
        let result = Digest::finalize(hasher);

        (f)(result)
    })
}
