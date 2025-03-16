use std::ops::{AddAssign, Index, IndexMut};

use eyre::{Report, Result, eyre};
use nohash_hasher::IntMap;
use rayon::prelude::*;

use aoc_meta::Problem;

/// <https://adventofcode.com/2015/day/3>
pub const PERFECTLY_SPHERICAL_HOUSES_IN_A_VACUUM: Problem = Problem::solved(&part_1, &part_2);

fn part_1(input: &str) -> Result<usize> {
    let mut grid = HouseGrid::default();
    let mut sleigh = Sleigh::default();

    let first_stop = std::iter::once(Ok(sleigh.position()));
    let directions = input.bytes().map(|byte| match byte {
        b'^' => Ok((0, 1)),
        b'>' => Ok((1, 0)),
        b'v' => Ok((0, -1)),
        b'<' => Ok((-1, 0)),
        other => Err(eyre!(
            "Unknown direction '{}'",
            char::from_u32(other.into()).unwrap_or(char::REPLACEMENT_CHARACTER)
        )),
    });

    first_stop.chain(directions).try_for_each(|result| {
        let (x, y) = result?;

        sleigh += (x, y);
        grid[sleigh.position()] += 1;

        Ok::<_, Report>(())
    })?;

    Ok(grid
        .into_par_iter()
        .filter(|(_coords, presents)| *presents > 0)
        .count())
}

fn part_2(input: &str) -> Result<usize> {
    let mut grid = HouseGrid::default();
    let mut santa = Sleigh::default();
    let mut robo_santa = Sleigh::default();

    let first_stops = [santa.position(), robo_santa.position()]
        .into_iter()
        .map(Ok);
    let directions = input.bytes().map(|byte| match byte {
        b'^' => Ok((0, 1)),
        b'>' => Ok((1, 0)),
        b'v' => Ok((0, -1)),
        b'<' => Ok((-1, 0)),
        other => Err(eyre!(
            "Unknown direction : {}",
            char::from_u32(other.into()).unwrap_or(char::REPLACEMENT_CHARACTER)
        )),
    });

    first_stops
        .chain(directions)
        .enumerate()
        .try_for_each(|(i, result)| {
            let (x, y) = result?;
            let sleigh = if i % 2 == 0 {
                &mut santa
            } else {
                &mut robo_santa
            };

            *sleigh += (x, y);
            grid[sleigh.position()] += 1;

            Ok::<_, Report>(())
        })?;

    Ok(grid
        .into_par_iter()
        .filter(|(_coords, presents)| *presents > 0)
        .count())
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
        self.0.entry(a).or_default().entry(b).or_default()
    }
}

impl HouseGrid {
    fn into_par_iter(self) -> impl ParallelIterator<Item = ((i32, i32), usize)> {
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
        let (a, b) = &mut self.position;

        *a += x;
        *b += y;
    }
}
