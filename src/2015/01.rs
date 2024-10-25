use eyre::{OptionExt, Result};
use rayon::prelude::*;
use wide::u8x16;

use crate::meta::Problem;

/// https://adventofcode.com/2015/day/1
pub const NOT_QUITE_LISP: Problem = Problem::solved(&part_1, &part_2);

const UP: u8 = b'(';
const DOWN: u8 = b')';

pub(crate) fn part_1(input: &str) -> Result<isize> {
    #[inline]
    fn sum_chunk(chunk: &[u8]) -> isize {
        if chunk.len() == u8x16::LANES as usize {
            let simd_chunk = u8x16::from(chunk);

            // This is a little confusing, but the idea is to use SIMD to process 16 bytes at a time.
            // The first thing to do is to compare the values in the SIMD register with `b'('`.
            // Values that are equal to `b'('` will be set to `u8::MAX`, others to `0u8`.
            let max_or_zero = simd_chunk.cmp_eq(u8x16::splat(DOWN));

            // The next step is to `saturating_add` one, so the values will either be `1` or `255`.
            let max_or_one = max_or_zero.saturating_add(u8x16::ONE);

            // finally, we can extract the values, which are all either `1` or `255`.
            let arr = max_or_one.to_array();

            // This is the cool bit;
            // - `255u8` has the same bitwise representation as `-1i8`
            // - `1u8` has the same bitwise representation as `1i8`
            // so we can just reinterpret the bytes!
            // this could be faster if `wide` had a `reduce_add`
            let negative_one_or_one = unsafe { std::mem::transmute::<[u8; 16], [i8; 16]>(arr) };

            let inner_sum = negative_one_or_one.into_iter().sum::<i8>();
            inner_sum.into()
        } else {
            chunk
                .iter()
                .copied()
                .map(|byte| if byte == UP { 1 } else { -1 })
                .sum()
        }
    }

    Ok(input
        .as_bytes()
        .par_chunks(u8x16::LANES.into())
        .map(sum_chunk)
        .sum())
}

pub(crate) fn part_2(input: &str) -> Result<usize> {
    let mut floor = 0;

    input
        .as_bytes()
        .iter()
        .position(|&byte| {
            floor += if byte == UP { 1 } else { -1 };
            floor == -1
        })
        .map(|n| n + 1)
        .ok_or_eyre("Santa never visited the basement")
}
