use std::{iter::once, mem, str::FromStr};

use eyre::{Report, Result, eyre};

use aoc_meta::Problem;
use tinyvec::ArrayVec;

pub const LIKE_A_ROGUE: Problem = Problem::solved(&num_safe_tiles::<40>, &num_safe_tiles::<400000>);

fn num_safe_tiles<const N: usize>(first_row: &str) -> Result<usize> {
    let first = first_row.parse()?;
    let rows = Rows::new(first);
    let n = rows.take(N).map(Row::num_safe_tiles).sum();
    Ok(n)
}

#[derive(Debug, Clone, Copy)]
struct Rows {
    current: Row,
}

impl Rows {
    fn new(first: Row) -> Self {
        Self { current: first }
    }
}

impl Iterator for Rows {
    type Item = Row;

    fn next(&mut self) -> Option<Self::Item> {
        let next = self.current.next_row();
        let current = mem::replace(&mut self.current, next);
        Some(current)
    }
}

#[derive(Debug, Clone, Copy)]
struct Row(ArrayVec<[Tile; 126]>);

impl Row {
    fn next_row(&self) -> Row {
        let mut first = [Tile::Safe; 3];
        first[1..].copy_from_slice(&self.0[..2]);

        let mut last = [Tile::Safe; 3];
        last[..2].copy_from_slice(&self.0[self.0.len() - 2..]);

        once(first.as_slice())
            .chain(self.0.windows(3))
            .chain(once(last.as_slice()))
            .map(Tile::compute_next)
            .collect()
    }

    fn num_safe_tiles(self) -> usize {
        self.0
            .into_iter()
            .filter(|&tile| matches!(tile, Tile::Safe))
            .count()
    }
}

impl FromStr for Row {
    type Err = Report;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        s.chars().map(Tile::try_from).collect()
    }
}

impl FromIterator<Tile> for Row {
    fn from_iter<T: IntoIterator<Item = Tile>>(iter: T) -> Self {
        Self(ArrayVec::from_iter(iter))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default)]
#[repr(u8)]
enum Tile {
    #[default]
    Safe = 0,
    Trap,
}

impl Tile {
    #[inline(always)]
    fn compute_next(prev: &[Tile]) -> Self {
        Tile::new(prev[0] != prev[2])
    }

    #[inline(always)]
    const fn new(is_trap: bool) -> Self {
        // Safety: booleans are represented as 0 or 1, which is always a valid Tile.
        unsafe { std::mem::transmute::<u8, Tile>(is_trap as u8) }
    }
}

impl TryFrom<char> for Tile {
    type Error = Report;

    fn try_from(value: char) -> Result<Self, Self::Error> {
        match value {
            '^' => Ok(Self::Trap),
            '.' => Ok(Self::Safe),
            other => Err(eyre!("unknown char '{other}', expected '.' or '^'")),
        }
    }
}
