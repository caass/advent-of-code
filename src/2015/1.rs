use std::string::ToString;

use rayon::prelude::*;
use wide::u8x16;

use crate::types::Problem;

pub const NOT_QUITE_LISP: Problem = Problem {
    name: "Not Quite Lisp",
    part_1: Some(|input| part_1(input).to_string()),
    part_2: Some(|input| part_2(input).to_string()),
};

const UP: u8 = b'(';
const DOWN: u8 = b')';

/// Santa was hoping for a white Christmas, but his weather machine's "snow" function is powered by stars, and he's
/// fresh out! To save Christmas, he needs you to collect **fifty stars** by December 25th.
///
/// Collect stars by helping Santa solve puzzles.
/// Two puzzles will be made available on each day in the Advent calendar;
/// the second puzzle is unlocked when you complete the first. Each puzzle grants **one star**. Good luck!
///
/// Here's an easy puzzle to warm you up.
///
/// Santa is trying to deliver presents in a large apartment building, but he can't find the right floor -
/// the directions he got are a little confusing. He starts on the ground floor (floor 0) and then follows
/// the instructions one character at a time.
///
/// An opening parenthesis, `(`, means he should go up one floor, and a closing parenthesis, `)`,
/// means he should go down one floor.
///
/// The apartment building is very tall, and the basement is very deep; he will never find the top or bottom floors.
///
/// For example:
/// - `(())` and `()()` both result in floor `0`.
/// - `(((` and `(()(()(` both result in floor `3`.
/// - `))(((((` also results in floor `3`.
/// - `())` and `))(` both result in floor `-1` (the first basement level).
/// - `)))` and `)())())` both result in floor `-3`.
///
/// To _what floor_ do the instructions take Santa?
fn part_1(input: &str) -> isize {
    // The naive approach to solve this would be to iterate through the string and check each character;
    // if it's equal to `'('`, add one. If it's equal to `')'`, subtract one. Something like:
    // ```
    // input.chars().map(|char| if char == '(' { 1 } else { -1 }).sum()
    // ```
    //
    // We can move faster by splitting up the string and processing it in multiple threads:
    // ```
    // use rayon::prelude::*;
    //
    // input.par_chars().map(|char| if char == '(' { 1 } else { -1 }).sum()
    // ```
    //
    // Finally, we can move even faster by using SIMD instructions to process 16 characters at a time.
    // That solution is what we use here.
    #[inline(always)]
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

    input
        .as_bytes()
        .par_chunks(u8x16::LANES.into())
        .map(sum_chunk)
        .sum()
}

/// Now, given the same instructions, find the position of the first character that causes him to enter the basement
/// (floor `-1`). The first character in the instructions has position `1`, the second character has position `2`,
/// and so on.
///
/// For example:
/// - `)` causes him to enter the basement at character position `1`.
/// - `()())` causes him to enter the basement at character position `5`.
///
/// What is the position of the character that causes Santa to first enter the basement?
fn part_2(input: &str) -> usize {
    // Similarly, we could easily just iterate over the characters and find the index of the one
    // that leads Santa to the basement. But we should be able to parallelize the process. In theory...
    // TODO: Parallelize.

    let mut floor = 0;

    input
        .as_bytes()
        .iter()
        .position(|&byte| {
            floor += if byte == UP { 1 } else { -1 };
            floor == -1
        })
        .expect("Santa to visit the basement at least once")
        + 1
}

#[test]
fn part_1_examples() {
    assert_eq!(part_1("(())"), 0);
    assert_eq!(part_1("()()"), 0);

    assert_eq!(part_1("((("), 3);
    assert_eq!(part_1("(()(()("), 3);

    assert_eq!(part_1("))((((("), 3);

    assert_eq!(part_1("())"), -1);
    assert_eq!(part_1("))("), -1);

    assert_eq!(part_1(")))"), -3);
    assert_eq!(part_1(")())())"), -3);
}

#[test]
fn part_2_examples() {
    assert_eq!(part_2(")"), 1);
    assert_eq!(part_2("()())"), 5)
}
