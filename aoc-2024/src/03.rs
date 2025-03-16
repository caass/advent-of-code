use eyre::Result;
#[cfg(test)]
use pretty_assertions::assert_eq;
use regex::Regex;
use void::Void;

use aoc_common::{TryFromStr, TryParse};
use aoc_meta::Problem;

pub const MULL_IT_OVER: Problem = Problem::solved(
    &|input| input.try_parse::<CorruptedMemory>()?.sum_of_products(),
    &|input| {
        input
            .try_parse::<CorruptedMemory>()?
            .conditional_sum_of_products()
    },
);

struct CorruptedMemory<'i>(&'i str);

impl<'i> TryFromStr<'i> for CorruptedMemory<'i> {
    type Err = Void;

    fn try_from_str(s: &'i str) -> Result<Self, Self::Err> {
        Ok(Self(s))
    }
}

thread_local! {
    static MUL_RE: Regex = Regex::new(r"mul\((\d+),(\d+)\)").unwrap();
}

impl CorruptedMemory<'_> {
    fn sum_of_products(&self) -> Result<usize> {
        sum_products_in_str(self.0)
    }

    fn conditional_sum_of_products(&self) -> Result<usize> {
        self.0
            .split("do()")
            .map(|chunk| {
                chunk
                    .split_once("don't()")
                    .map_or(chunk, |(inner_chunk, ..)| inner_chunk)
            })
            .map(sum_products_in_str)
            .try_fold(0, |a, res| res.map(|b| a + b))
    }
}

fn sum_products_in_str(s: &str) -> Result<usize> {
    MUL_RE.with(|re| {
        re.captures_iter(s)
            .map(|cap| -> Result<usize> {
                let (_, [a_str, b_str]) = cap.extract();

                let a = a_str.parse::<usize>()?;
                let b = b_str.parse::<usize>()?;
                Ok(a * b)
            })
            .try_fold(0usize, |a, res| res.map(|b| a + b))
    })
}

#[test]
fn part_1() {
    static INPUT: &str =
        r"xmul(2,4)&mul[3,7]!^don't()_mul(5,5)+mul(32,64](mul(11,8)undo()?mul(8,5))";

    let memory: CorruptedMemory = INPUT.try_parse().unwrap();
    assert_eq!(memory.sum_of_products().unwrap(), 161);
}

#[test]
fn part_2() {
    static INPUT: &str =
        r"xmul(2,4)&mul[3,7]!^don't()_mul(5,5)+mul(32,64](mul(11,8)undo()?mul(8,5))";

    let memory: CorruptedMemory = INPUT.try_parse().unwrap();
    assert_eq!(memory.conditional_sum_of_products().unwrap(), 48);
}
