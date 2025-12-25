use std::{collections::HashSet, hash::Hash, str::FromStr};

use aoc_meta::Problem;
use either::Either;
use eyre::{OptionExt, Report, Result, eyre};
use itertools::Itertools;
use nohash_hasher::BuildNoHashHasher;
use rayon::prelude::*;

pub const MOVIE_THEATER: Problem = Problem::solved(&largest_rectangle, &largest_green_rectangle);

fn largest_rectangle(input: &str) -> Result<u64> {
    let coords: Vec<_> = input.lines().map(Coordinate::from_str).try_collect()?;

    coords
        .par_iter()
        .copied()
        .enumerate()
        .flat_map(|(i, a)| coords[i + 1..].par_iter().copied().map(move |b| (a, b)))
        .map(|(a, b)| a.area(b))
        .max()
        .ok_or_eyre("no red tiles!")
}

fn largest_green_rectangle(input: &str) -> Result<u64> {
    let reds: Vec<_> = input.lines().map(Coordinate::from_str).try_collect()?;

    let edges = reds
        .iter()
        .copied()
        .circular_tuple_windows()
        .flat_map(|(a, b)| a.edge_to(b))
        .chain(reds.iter().copied())
        .collect::<HashSet<Coordinate, BuildNoHashHasher<Coordinate>>>();

    // TODO: fill edges of the polygon

    todo!()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct Coordinate {
    x: u32,
    y: u32,
}

impl Coordinate {
    fn area(self, other: Coordinate) -> u64 {
        (u64::from(self.x.abs_diff(other.x)) + 1) * (u64::from(self.y.abs_diff(other.y)) + 1)
    }

    fn edge_to(self, other: Coordinate) -> impl Iterator<Item = Coordinate> {
        if self.x == other.x && self.y == other.y {
            Either::Left(Either::Left(std::iter::once(self)))
        } else if self.x != other.x && self.y != other.y {
            Either::Right(std::iter::empty())
        } else {
            let coords = if self.x != other.x {
                let range = if self.x < other.x {
                    self.x..=other.x
                } else {
                    other.x..=self.x
                };

                range.with_y(self.y)
            } else {
                let range = if self.y < other.y {
                    self.y..=other.y
                } else {
                    other.y..=self.y
                };

                range.with_x(self.x)
            };

            Either::Left(Either::Right(coords))
        }
    }
}

impl FromStr for Coordinate {
    type Err = Report;

    fn from_str(s: &str) -> Result<Self> {
        let (x_str, y_str) = s
            .split_once(',')
            .ok_or_else(|| eyre!("failed to parse line \"{s}\""))?;

        let x = x_str.parse()?;
        let y = y_str.parse()?;

        Ok(Self { x, y })
    }
}

impl Hash for Coordinate {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        // Safety: it's safe to interpret two 32-bit integers next to each other as a single 64-bit integer
        state.write_u64(unsafe { std::mem::transmute::<Coordinate, u64>(*self) });
    }
}

impl nohash_hasher::IsEnabled for Coordinate {}

#[derive(Debug, Clone)]
struct CoordRange<I> {
    constant: u32,
    iter_axis: Axis,
    iter: I,
}

impl<I> Iterator for CoordRange<I>
where
    I: Iterator<Item = u32>,
{
    type Item = Coordinate;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().map(|next| {
            if self.iter_axis == Axis::X {
                Coordinate {
                    x: next,
                    y: self.constant,
                }
            } else {
                Coordinate {
                    x: self.constant,
                    y: next,
                }
            }
        })
    }
}

trait IterExt: Sized {
    fn with_x(self, x: u32) -> CoordRange<Self>;
    fn with_y(self, y: u32) -> CoordRange<Self>;
}

impl<I> IterExt for I
where
    I: Iterator<Item = u32>,
{
    fn with_x(self, x: u32) -> CoordRange<Self> {
        CoordRange {
            constant: x,
            iter_axis: Axis::Y,
            iter: self,
        }
    }

    fn with_y(self, y: u32) -> CoordRange<Self> {
        CoordRange {
            constant: y,
            iter_axis: Axis::X,
            iter: self,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Axis {
    X,
    Y,
}

#[test]
fn example() {
    use pretty_assertions::assert_eq;

    let input = "7,1
11,1
11,7
9,7
9,5
2,5
2,3
7,3
";

    assert_eq!(largest_rectangle(input).unwrap(), 50);
}
