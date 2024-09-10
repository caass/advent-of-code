use rayon::{iter::Flatten, prelude::*};
use winnow::{
    ascii::digit1,
    combinator::{alt, separated_pair},
    prelude::*,
};

use crate::types::{problem, Problem};

pub const PROBABLY_A_FIRE_HAZARD: Problem = problem!(part_1, part_2);

fn part_1(input: &str) -> usize {
    let mut grid = Grid::<Light>::default();
    input
        .lines()
        .map(Instruction::parse)
        .for_each(|Instruction { action, range }| {
            grid.range_mut(range).for_each(|light| light.act(action));
        });

    grid.into_par_iter().filter(|light| light.on).count()
}

fn part_2(input: &str) -> usize {
    let mut grid = Grid::<AdjustableLight>::default();
    input
        .lines()
        .map(Instruction::parse)
        .for_each(|Instruction { action, range }| {
            grid.range_mut(range).for_each(|light| light.act(action));
        });

    grid.into_par_iter().map(|light| light.brightness).sum()
}

struct Grid<T> {
    rows: Vec<Vec<T>>,
}

impl<T: Default + Clone> Default for Grid<T> {
    fn default() -> Self {
        Self {
            rows: vec![vec![T::default(); 1000]; 1000],
        }
    }
}

impl<T: Send> Grid<T> {
    fn range_mut(&mut self, range: InstructionRange) -> impl ParallelIterator<Item = &mut T> {
        self.rows[range.from.x..=range.to.x]
            .par_iter_mut()
            .flat_map(move |row| &mut row[range.from.y..=range.to.y])
    }
}

impl<T: Send> IntoParallelIterator for Grid<T> {
    type Iter = Flatten<<Vec<Vec<T>> as IntoParallelIterator>::Iter>;

    type Item = T;

    fn into_par_iter(self) -> Self::Iter {
        self.rows.into_par_iter().flatten()
    }
}

#[derive(Debug)]
struct Instruction {
    action: Action,
    range: InstructionRange,
}

impl Instruction {
    fn parse(line: &str) -> Self {
        separated_pair(Action::parse, ' ', InstructionRange::parse)
            .map(|(action, range)| Instruction { action, range })
            .parse(line)
            .unwrap()
    }
}

#[derive(Debug)]
struct InstructionRange {
    from: Coordinates,
    to: Coordinates,
}

impl InstructionRange {
    fn parse(input: &mut &str) -> PResult<Self> {
        separated_pair(Coordinates::parse, " through ", Coordinates::parse)
            .map(|(from, to)| InstructionRange { from, to })
            .parse_next(input)
    }
}

#[derive(Debug)]
struct Coordinates {
    x: usize,
    y: usize,
}

impl Coordinates {
    fn parse(input: &mut &str) -> PResult<Self> {
        separated_pair(digit1.parse_to(), ',', digit1.parse_to())
            .map(|(x, y)| Coordinates { x, y })
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
    fn parse(input: &mut &str) -> PResult<Self> {
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
    #[inline(always)]
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
    #[inline(always)]
    fn act(&mut self, action: Action) {
        match action {
            Action::TurnOn => self.brightness += 1,
            Action::TurnOff => self.brightness = self.brightness.saturating_sub(1),
            Action::Toggle => self.brightness += 2,
        }
    }
}
