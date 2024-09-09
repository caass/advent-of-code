use std::{
    collections::hash_map::Entry,
    ops::{AddAssign, Index, IndexMut},
};

use nohash_hasher::IntMap;
use rayon::prelude::*;

use crate::types::Problem;

pub const PERFECTLY_SPHERICAL_HOUSES_IN_A_VACUUM: Problem = Problem {
    part1: Some(|input| part_1(input).to_string()),
    part2: Some(|input| part_2(input).to_string()),
};

fn part_1(input: &str) -> usize {
    let mut grid = HouseGrid::default();
    let mut sleigh = Sleigh::default();

    let first_stop = std::iter::once(sleigh.position());
    let directions = input.bytes().map(|byte| match byte {
        b'^' => (0, 1),
        b'>' => (1, 0),
        b'v' => (0, -1),
        b'<' => (-1, 0),
        _ => unreachable!("invalid direction for Santa"),
    });

    first_stop.chain(directions).for_each(|(x, y)| {
        sleigh += (x, y);
        grid[sleigh.position()] += 1;
    });

    grid._into_par_iter()
        .filter(|(_coords, presents)| *presents > 0)
        .count()
}

fn part_2(input: &str) -> usize {
    let mut grid = HouseGrid::default();
    let mut santa = Sleigh::default();
    let mut robo_santa = Sleigh::default();

    let first_stops = [santa.position(), robo_santa.position()].into_iter();
    let directions = input.bytes().map(|byte| match byte {
        b'^' => (0, 1),
        b'>' => (1, 0),
        b'v' => (0, -1),
        b'<' => (-1, 0),
        _ => unreachable!("invalid direction for Santa"),
    });

    first_stops
        .chain(directions)
        .enumerate()
        .for_each(|(i, (x, y))| {
            let sleigh = if i % 2 == 0 {
                &mut santa
            } else {
                &mut robo_santa
            };

            *sleigh += (x, y);
            grid[sleigh.position()] += 1;
        });

    grid._into_par_iter()
        .filter(|(_coords, presents)| *presents > 0)
        .count()
}

#[derive(Default)]
struct HouseGrid(IntMap<i32, IntMap<i32, usize>>);

impl Index<(i32, i32)> for HouseGrid {
    type Output = usize;

    fn index(&self, (a, b): (i32, i32)) -> &Self::Output {
        self.0.get(&a).and_then(|inner| inner.get(&b)).unwrap_or(&0)
    }
}

impl IndexMut<(i32, i32)> for HouseGrid {
    fn index_mut(&mut self, (a, b): (i32, i32)) -> &mut Self::Output {
        let inner = match self.0.entry(a) {
            Entry::Occupied(occ) => occ.into_mut(),
            Entry::Vacant(vac) => vac.insert(IntMap::default()),
        };

        match inner.entry(b) {
            Entry::Occupied(occ) => occ.into_mut(),
            Entry::Vacant(vac) => vac.insert(0),
        }
    }
}

impl HouseGrid {
    fn _into_par_iter(self) -> impl ParallelIterator<Item = ((i32, i32), usize)> {
        self.0
            .into_par_iter()
            .flat_map(|(x, inner)| inner.into_par_iter().map(move |(y, count)| ((x, y), count)))
    }
}

#[derive(Default)]
struct Sleigh {
    position: (i32, i32),
}

impl Sleigh {
    fn position(&self) -> (i32, i32) {
        self.position
    }
}

impl AddAssign<(i32, i32)> for Sleigh {
    fn add_assign(&mut self, (x, y): (i32, i32)) {
        let (ref mut a, ref mut b) = &mut self.position;

        *a += x;
        *b += y;
    }
}
