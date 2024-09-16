use eyre::Result;
use itertools::Itertools;

use crate::types::{problem, Problem};

pub const ELVES_LOOK_ELVES_SAY: Problem = problem!(part1, part2);

fn part1(input: &str) -> Result<usize> {
    let mut n = input.to_string();
    dbg!(&n);
    for _ in 0..40 {
        n = look_and_say(&n);
        dbg!(&n);
    }

    Ok(n.len())
}

fn part2(input: &str) -> Result<usize> {
    let mut n = input.to_string();
    for _ in 0..50 {
        n = look_and_say(&n);
    }

    Ok(n.len())
}

fn look_and_say(n: &str) -> String {
    let mut out = String::new();
    for (digit, group) in &n.to_string().chars().chunk_by(|ch| *ch) {
        let num_repetitions = group.count().to_string();
        out.push_str(&num_repetitions);
        out.push(digit);
    }

    out
}
