use md5::{digest::Output, Digest, Md5};
use rayon::prelude::*;

use crate::types::{problem, Problem};

pub const THE_IDEAL_STOCKING_STUFFER: Problem = problem!(part_1, part_2);

fn part_1(input: &str) -> usize {
    find(input, |hash| hash[0] == 0 && hash[1] == 0 && hash[2] < 0x10)
        .expect("to find at least one number that starts with 5 zeroes")
}

fn part_2(input: &str) -> usize {
    find(input, |hash| hash[0] == 0 && hash[1] == 0 && hash[2] == 0)
        .expect("to find at least one number that starts with 6 zeroes")
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
