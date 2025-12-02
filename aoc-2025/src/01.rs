use std::str::FromStr;

use aoc_meta::Problem;
use eyre::{Report, Result, bail};

pub const SECRET_ENTRANCE: Problem =
    Problem::solved(&|input| zero_count(input, Dial::apply_v1), &|input| {
        zero_count(input, Dial::apply_v2)
    });

fn zero_count<F: FnMut(&mut Dial, Rotation)>(input: &str, mut f: F) -> Result<usize> {
    let mut dial = Dial::new();

    for line in input.lines() {
        let rotation = line.parse()?;
        (f)(&mut dial, rotation);
    }

    Ok(dial.zero_count)
}

#[derive(Debug, Clone, Copy)]
struct Dial {
    pointing_to: i16,
    zero_count: usize,
}

impl Dial {
    const fn new() -> Dial {
        Dial {
            pointing_to: 50,
            zero_count: 0,
        }
    }

    fn apply_v1(&mut self, Rotation(rotation): Rotation) {
        self.pointing_to = (self.pointing_to + rotation).rem_euclid(100);
        self.zero_count += self.is_zero() as usize;
    }

    fn apply_v2(&mut self, Rotation(_rotation): Rotation) {
        todo!()
    }

    const fn is_zero(&self) -> bool {
        self.pointing_to == 0
    }
}

impl Default for Dial {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Copy)]
struct Rotation(i16);

impl FromStr for Rotation {
    type Err = Report;

    fn from_str(s: &str) -> Result<Self> {
        let Some((direction, magnitude)) = s.split_at_checked(1) else {
            bail!("couldn't split line {s} into direction and magnitude");
        };

        let direction_multiplier = match direction {
            "L" => -1i16,
            "R" => 1,
            _ => bail!("couldn't parse a direction from {direction}"),
        };

        Ok(Self(magnitude.parse::<i16>()? * direction_multiplier))
    }
}

#[test]
fn example() {
    use pretty_assertions::assert_eq;

    static INPUT: &str = "L68
L30
R48
L5
R60
L55
L1
L99
R14
L82";

    assert_eq!(zero_count(INPUT, Dial::apply_v1).unwrap(), 3);
    assert_eq!(zero_count(INPUT, Dial::apply_v2).unwrap(), 6);
}
