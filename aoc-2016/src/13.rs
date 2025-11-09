use std::collections::VecDeque;
use std::collections::btree_map::{BTreeMap, Entry};
use std::iter::FusedIterator;

use eyre::eyre;

use aoc_common::grid::{Coordinate, Grid};
use aoc_meta::Problem;

const GRID_SIZE: usize = 128;

pub const A_MAZE_OF_TWISTY_LITTLE_CUBICLES: Problem = Problem::solved(
    &|input| {
        let fav = input.parse()?;
        let end = Coordinate { x: 31, y: 39 };

        bfs(fav)
            .find_map(|(coord, steps)| (coord == end).then_some(steps))
            .ok_or_else(|| eyre!("no way to {end:?} on a {GRID_SIZE}x{GRID_SIZE} grid"))
    },
    &|input| {
        input.parse().map(|fav| {
            bfs(fav)
                .take_while(|&(_, this_steps)| this_steps <= 50)
                .count()
        })
    },
);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Location {
    Space,
    Wall,
}

impl Location {
    fn is_space(&self) -> bool {
        matches!(self, Location::Space)
    }
}

#[derive(Debug)]
struct CubicleBfs<const N: usize> {
    grid: Grid<N, Location>,
    queue: VecDeque<(Coordinate, usize)>,
    visited: BTreeMap<Coordinate, usize>,
}

impl<const N: usize> Iterator for CubicleBfs<N> {
    type Item = (Coordinate, usize);

    fn next(&mut self) -> Option<Self::Item> {
        self.queue.pop_front().inspect(|&(coord, steps)| {
            for neighbor in coord.cardinal_neighbors() {
                if self.grid.get(neighbor).is_some_and(Location::is_space)
                    && let Entry::Vacant(vac) = self.visited.entry(neighbor)
                {
                    vac.insert(steps + 1);
                    self.queue.push_back((neighbor, steps + 1));
                }
            }
        })
    }
}

impl<const N: usize> FusedIterator for CubicleBfs<N> {}

fn bfs(favorite_number: usize) -> CubicleBfs<GRID_SIZE> {
    let grid = Grid::from_fn(|Coordinate { x, y }| {
        let sum = x * x + 3 * x + 2 * x * y + y + y * y + favorite_number;
        if sum.count_ones().is_multiple_of(2) {
            Location::Space
        } else {
            Location::Wall
        }
    });

    let start = Coordinate { x: 1, y: 1 };

    let mut queue = VecDeque::default();
    let mut visited = BTreeMap::default();

    queue.push_back((start, 0));
    visited.insert(start, 0);

    CubicleBfs {
        grid,
        queue,
        visited,
    }
}
