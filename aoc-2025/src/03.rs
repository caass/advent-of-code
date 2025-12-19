use eyre::Result;
use rayon::prelude::*;

use aoc_meta::Problem;

pub const LOBBY: Problem = Problem::solved(&total_joltage::<2>, &total_joltage::<12>);

fn total_joltage<const N: usize>(input: &str) -> Result<u64> {
    input
        .par_lines()
        .map(maximum_joltage::<N>)
        .try_reduce(|| 0, |a, b| Ok(a + b))
}

fn maximum_joltage<const N: usize>(line: &str) -> Result<u64> {
    if line.len() <= N {
        return Ok(line.parse()?);
    }

    let mut digits = [b'0'; N];
    let mut start = 0;

    for i in 0..N {
        let Some((idx, place)) = line[start..line.len() - (N - i - 1)]
            .bytes()
            .enumerate()
            .reduce(|(i, a), (j, b)| if a >= b { (i, a) } else { (j, b) })
        else {
            return Ok(0);
        };

        digits[i] = place;
        start += idx + 1;
    }

    Ok(str::from_utf8(&digits)?.parse()?)
}

#[test]
fn example() {
    use pretty_assertions::assert_eq;

    let input = "987654321111111
811111111111119
234234234234278
818181911112111";

    assert_eq!(total_joltage::<2>(input).unwrap(), 357);
    assert_eq!(total_joltage::<12>(input).unwrap(), 3121910778619);
}
