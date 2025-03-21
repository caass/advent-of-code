use itertools::Itertools;

use aoc_meta::Problem;

/// <https://adventofcode.com/2015/day/10>
pub const ELVES_LOOK_ELVES_SAY: Problem = Problem::solved(&look_and_say::<40>, &look_and_say::<50>);

fn look_and_say<const N: usize>(n: &str) -> usize {
    let mut n = n.to_string();
    for _ in 0..N {
        n = look_and_say_once(&n);
    }

    n.len()
}

fn look_and_say_once(n: &str) -> String {
    let mut out = String::new();
    for (digit, group) in &n.to_string().chars().chunk_by(|ch| *ch) {
        let num_repetitions = group.count().to_string();
        out.push_str(&num_repetitions);
        out.push(digit);
    }

    out
}
