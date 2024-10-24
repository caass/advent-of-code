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
    /// The distance from the origin after following all the instructions.
    fn final_distance(self) -> u16 {
        let mut pedestrian = Pose::default();

        for instruction in self {
            pedestrian.follow(instruction);
        }

        pedestrian.position.distance_from_origin()
    }

    /// The distance from the origin at which point a tron character would crash into their own wall.
    fn tron_distance(self) -> u16 {
        todo!()
    }
}

#[derive(Debug, Clone, Copy, Default)]
struct Position {
    x: i16,
    y: i16,
}

impl Position {
    #[inline(always)]
    fn distance_from_origin(&self) -> u16 {
        self.x.unsigned_abs() + self.y.unsigned_abs()
    }

    #[inline(always)]
    fn north(&mut self, steps: i16) {
        self.y += steps;
    }

    #[inline(always)]
    fn east(&mut self, steps: i16) {
        self.x += steps;
    }

    #[inline(always)]
    fn south(&mut self, steps: i16) {
        self.y -= steps;
    }

    #[inline(always)]
    fn west(&mut self, steps: i16) {
        self.x -= steps
    }

    fn walk(&mut self, steps: u8, orientation: Orientation) {
        let steps = i16::from(steps);

        match orientation {
            Orientation::North => self.north(steps),
            Orientation::East => self.east(steps),
            Orientation::South => self.south(steps),
            Orientation::West => self.west(steps),
        }
    }
}

#[derive(Debug, Clone, Copy, Default)]
struct Pose {
    position: Position,
    orientation: Orientation,
}

impl Pose {
    #[inline(always)]
    fn follow(&mut self, Instruction { turn, steps }: Instruction) {
        self.orientation.turn(turn);
        self.position.walk(steps, self.orientation);
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

impl IntoIterator for Instructions {
    type Item = Instruction;

    type IntoIter = <Vec<Instruction> as IntoIterator>::IntoIter;

    #[inline(always)]
    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl FromStr for Instructions {
    type Err = Report;

    #[inline(always)]
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

    #[inline(always)]
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

    #[inline(always)]
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        seq! {Instruction {
            turn: alt::<_, _, ContextError, _>(("R", "L")).parse_to(),
            steps: dec_uint
        }}
        .parse(s)
        .map_err(|_| eyre!("Invalid instruction: {s}"))
    }
}
