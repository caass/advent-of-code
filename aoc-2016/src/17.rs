use std::{collections::VecDeque, iter::FusedIterator, ops};

use eyre::{OptionExt, Result};
use md5::{Digest, Md5};

use aoc_common::grid::Coordinate;
use aoc_meta::Problem;

pub const TWO_STEPS_FORWARD: Problem = Problem::solved(&shortest_path, &longest_path_length);

fn shortest_path(passcode: &str) -> Result<String> {
    bfs(passcode).next().ok_or_eyre("no path to exit!")
}

fn longest_path_length(passcode: &str) -> Result<usize> {
    bfs(passcode)
        .map(|s| s.len())
        .max()
        .ok_or_eyre("no path to exit")
}

fn bfs(passcode: &str) -> Bfs {
    Bfs::new(passcode)
}

struct Bfs {
    hasher: Md5,
    queue: VecDeque<(Coordinate, String)>,
}

impl Bfs {
    const TARGET: Coordinate = Coordinate { x: 3, y: 3 };

    fn new(passcode: &str) -> Self {
        let hasher = Md5::new_with_prefix(passcode);
        let mut queue = VecDeque::new();
        queue.push_back((Coordinate { x: 0, y: 0 }, String::new()));

        Self { hasher, queue }
    }
}

impl Iterator for Bfs {
    type Item = String;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some((coord, steps)) = self.queue.pop_front() {
            if coord == Self::TARGET {
                return Some(steps);
            }

            let hash = self.hasher.clone().chain_update(&steps).finalize();
            let up_open = hash[0] >> 4 > 0xA;
            let down_open = hash[0] & 0xF > 0xA;
            let left_open = hash[1] >> 4 > 0xA;
            let right_open = hash[1] & 0xF > 0xA;

            let next = [
                (up_open && coord.y > 0).then_some(Move::Up),
                (down_open && coord.y < 3).then_some(Move::Down),
                (left_open && coord.x > 0).then_some(Move::Left),
                (right_open && coord.x < 3).then_some(Move::Right),
            ]
            .into_iter()
            .flatten()
            .map(|mov| (coord + mov, steps.clone() + mov));

            self.queue.extend(next);
        }

        None
    }
}

impl FusedIterator for Bfs {}

#[derive(Debug, Clone, Copy)]
enum Move {
    Up,
    Down,
    Left,
    Right,
}

impl From<Move> for char {
    fn from(value: Move) -> Self {
        match value {
            Move::Up => 'U',
            Move::Down => 'D',
            Move::Left => 'L',
            Move::Right => 'R',
        }
    }
}

impl ops::Add<Move> for Coordinate {
    type Output = Coordinate;

    fn add(mut self, rhs: Move) -> Self::Output {
        match rhs {
            Move::Up => self.y -= 1,
            Move::Down => self.y += 1,
            Move::Left => self.x -= 1,
            Move::Right => self.x += 1,
        }

        self
    }
}

impl ops::Add<Move> for String {
    type Output = String;

    fn add(mut self, rhs: Move) -> Self::Output {
        self.push(rhs.into());
        self
    }
}
