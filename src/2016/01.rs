use std::str::FromStr;

use eyre::{bail, eyre, Report, Result};
use winnow::{
    ascii::dec_uint,
    combinator::{alt, seq},
    error::ContextError,
    prelude::*,
};

use crate::meta::Problem;

pub const NO_TIME_FOR_A_TAXICAB: Problem =
    Problem::partially_solved(&|input| input.parse().map(Instructions::final_distance));

#[derive(Debug)]
struct Instructions(Vec<Instruction>);

impl Instructions {
    fn final_distance(self) -> usize {
        let mut position = Position::default();
        dbg!(&self);

        self.0
            .into_iter()
            .for_each(|instruction| position.follow(instruction));
        position.distance_from_origin()
    }
}

#[derive(Debug, Clone, Copy, Default)]
struct Position {
    x: isize,
    y: isize,
    orientation: Orientation,
}

impl Position {
    fn distance_from_origin(&self) -> usize {
        self.x.unsigned_abs() + self.y.unsigned_abs()
    }

    fn follow(&mut self, Instruction { turn, steps }: Instruction) {
        self.turn(turn);
        self.walk(steps);
    }

    fn turn(&mut self, turn: Turn) {
        self.orientation.turn(turn);
    }

    fn walk(&mut self, steps: u8) {
        let steps = isize::from(steps);

        match self.orientation {
            Orientation::North => self.y += steps,
            Orientation::East => self.x += steps,
            Orientation::South => self.y -= steps,
            Orientation::West => self.x -= steps,
        }
    }
}

#[derive(Debug, Default, Clone, Copy)]
enum Orientation {
    #[default]
    North,
    East,
    South,
    West,
}

impl Orientation {
    #[inline(always)]
    fn turn(&mut self, turn: Turn) {
        *self = match (&self, turn) {
            (Orientation::East, Turn::Left) | (Orientation::West, Turn::Right) => {
                Orientation::North
            }
            (Orientation::North, Turn::Right) | (Orientation::South, Turn::Left) => {
                Orientation::East
            }
            (Orientation::East, Turn::Right) | (Orientation::West, Turn::Left) => {
                Orientation::South
            }
            (Orientation::North, Turn::Left) | (Orientation::South, Turn::Right) => {
                Orientation::West
            }
        };
    }
}

impl FromIterator<Instruction> for Instructions {
    #[inline(always)]
    fn from_iter<T: IntoIterator<Item = Instruction>>(iter: T) -> Self {
        Self(Vec::from_iter(iter))
    }
}

impl FromStr for Instructions {
    type Err = Report;

    fn from_str(s: &str) -> Result<Self> {
        s.trim().split(", ").map(Instruction::from_str).collect()
    }
}

#[derive(Debug, Clone, Copy)]
enum Turn {
    Left,
    Right,
}

impl FromStr for Turn {
    type Err = Report;

    fn from_str(s: &str) -> Result<Self> {
        match s {
            "R" => Ok(Self::Right),
            "L" => Ok(Self::Left),
            _ => bail!("Can only turn left (L) or right (R), not {s}"),
        }
    }
}

#[derive(Debug)]
struct Instruction {
    turn: Turn,
    steps: u8,
}

impl FromStr for Instruction {
    type Err = Report;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        seq! {Instruction {
            turn: alt::<_, _, ContextError, _>(("R", "L")).parse_to(),
            steps: dec_uint
        }}
        .parse(s)
        .map_err(|_| eyre!("Invalid instruction: {s}"))
    }
}
