use std::ops::{Bound, RangeBounds};
use std::str::FromStr;

use eyre::{eyre, Report, Result};
use rayon::prelude::*;
use winnow::{
    ascii::digit1,
    combinator::{alt, separated_pair},
    prelude::*,
};

use crate::common::grid::{Coordinate, Grid};
use crate::meta::Problem;

/// <https://adventofcode.com/2015/day/6>
pub const PROBABLY_A_FIRE_HAZARD: Problem = Problem::solved(&part_1, &part_2);

const SIDE_LENGTH: usize = 1000;

fn part_1(input: &str) -> Result<usize> {
    let mut grid = Grid::<SIDE_LENGTH, Light>::default();
    for line in input.lines() {
        let Instruction { action, range } = line.parse()?;
        grid.par_range_mut(range)
            .for_each(|(_, light)| light.act(action));
    }

    Ok(grid.into_par_iter().filter(|light| light.on).count())
}

fn part_2(input: &str) -> Result<usize> {
    let mut grid = Grid::<SIDE_LENGTH, AdjustableLight>::default();
    for line in input.lines() {
        let Instruction { action, range } = line.parse()?;
        grid.par_range_mut(range)
            .for_each(|(_, light)| light.act(action));
    }

    Ok(grid.into_par_iter().map(|light| light.brightness).sum())
}

#[derive(Debug)]
struct Instruction {
    action: Action,
    range: InstructionRange,
}

impl FromStr for Instruction {
    type Err = Report;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        separated_pair(Action::parse, ' ', InstructionRange::parse)
            .map(|(action, range)| Instruction { action, range })
            .parse(s)
            .map_err(|e| eyre!("{e}"))
    }
}

#[derive(Debug)]
struct InstructionRange {
    from: Coordinate,
    to: Coordinate,
}

impl RangeBounds<Coordinate> for InstructionRange {
    fn start_bound(&self) -> Bound<&Coordinate> {
        Bound::Included(&self.from)
    }

    fn end_bound(&self) -> Bound<&Coordinate> {
        Bound::Included(&self.to)
    }
}

impl InstructionRange {
    fn parse(input: &mut &str) -> ModalResult<Self> {
        fn parse_coordinate(input: &mut &str) -> ModalResult<Coordinate> {
            separated_pair(digit1.parse_to(), ',', digit1.parse_to())
                .map(|(x, y)| Coordinate { x, y })
                .parse_next(input)
        }

        separated_pair(parse_coordinate, " through ", parse_coordinate)
            .map(|(from, to)| InstructionRange { from, to })
            .parse_next(input)
    }
}

#[derive(Debug, Clone, Copy)]
enum Action {
    TurnOn,
    TurnOff,
    Toggle,
}

impl Action {
    fn parse(input: &mut &str) -> ModalResult<Self> {
        alt((
            "turn off".map(|_| Action::TurnOff),
            "turn on".map(|_| Action::TurnOn),
            "toggle".map(|_| Action::Toggle),
        ))
        .parse_next(input)
    }
}

#[derive(Debug, Default, Clone, Copy)]
struct Light {
    on: bool,
}

impl Light {
    #[inline]
    fn act(&mut self, action: Action) {
        match action {
            Action::TurnOn => self.on = true,
            Action::TurnOff => self.on = false,
            Action::Toggle => self.on = !self.on,
        }
    }
}

#[derive(Debug, Default, Clone, Copy)]
struct AdjustableLight {
    brightness: usize,
}

impl AdjustableLight {
    #[inline]
    fn act(&mut self, action: Action) {
        match action {
            Action::TurnOn => self.brightness += 1,
            Action::TurnOff => self.brightness = self.brightness.saturating_sub(1),
            Action::Toggle => self.brightness += 2,
        }
    }
}
