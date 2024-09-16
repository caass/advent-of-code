use eyre::Result;
use itertools::Itertools;
use void::Void;

use crate::types::{problem, Problem};

pub const ELVES_LOOK_ELVES_SAY: Problem = problem!(look_and_say_n::<40>, look_and_say_n::<50>);

fn look_and_say_n<const N: usize>(n: &str) -> Result<usize, Void> {
    let mut n = n.to_string();
    for _ in 0..N {
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
