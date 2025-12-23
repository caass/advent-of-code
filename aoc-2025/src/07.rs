use std::{
    fmt::{self, Display, Formatter, Write},
    hash::BuildHasherDefault,
    str::FromStr,
};

use dashmap::DashMap;
use eyre::{Report, Result, eyre};

use aoc_meta::Problem;
use itertools::Itertools;
use seahash::SeaHasher;

pub const LABORATORIES: Problem = Problem::solved(
    &|input| input.parse().map(Manifold::count_splits),
    &|input| input.parse().map(Manifold::count_timelines),
);

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
enum Space {
    #[default]
    Empty,
    Emitter,
    Splitter,
    Beam,
}

impl TryFrom<char> for Space {
    type Error = Report;

    fn try_from(ch: char) -> Result<Space> {
        match ch {
            'S' => Ok(Space::Emitter),
            '.' => Ok(Space::Empty),
            '^' => Ok(Space::Splitter),
            '|' => Ok(Space::Beam),
            _ => Err(eyre!("unknown char '{ch}'")),
        }
    }
}

impl Display for Space {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_char(match *self {
            Space::Empty => '.',
            Space::Emitter => 'S',
            Space::Splitter => '^',
            Space::Beam => '|',
        })
    }
}

#[derive(Debug, Clone)]
struct Manifold(Vec<Vec<Space>>);

impl Manifold {
    fn count_splits(self) -> usize {
        let mut iter = self.0.into_iter().peekable();
        let mut splits = 0;

        while let (Some(prev), Some(next)) = (iter.next(), iter.peek_mut()) {
            for i in 0..prev.len() {
                if matches!(prev[i], Space::Emitter | Space::Beam) {
                    match next[i] {
                        Space::Splitter => {
                            splits += 1;

                            if let Some(sp @ Space::Empty) =
                                i.checked_sub(1).and_then(|j| next.get_mut(j))
                            {
                                *sp = Space::Beam;
                            }

                            if let Some(sp @ Space::Empty) =
                                i.checked_add(1).and_then(|j| next.get_mut(j))
                            {
                                *sp = Space::Beam;
                            }
                        }
                        ref mut other => *other = Space::Beam,
                    }
                }
            }
        }

        splits
    }

    fn count_timelines(self) -> u64 {
        let Some(emitter_position) = self.0[0].iter().position(|&s| s == Space::Emitter) else {
            return 0;
        };

        self._count_timelines(1, emitter_position, &DashMap::default())
    }

    fn _count_timelines(
        &self,
        row: usize,
        col: usize,
        cache: &DashMap<(usize, usize), u64, BuildHasherDefault<SeaHasher>>,
    ) -> u64 {
        if let Some(n) = cache.get(&(row, col)).as_deref().copied() {
            return n;
        }

        let mut current_row = row;

        while matches!(
            self.0.get(current_row).and_then(|row| row.get(col)),
            Some(Space::Empty)
        ) {
            current_row += 1;
        }

        let n = if current_row >= self.0.len() {
            1
        } else {
            let (a, b) = rayon::join(
                || {
                    if col > 0 {
                        self._count_timelines(current_row + 1, col - 1, cache)
                    } else {
                        0
                    }
                },
                || {
                    if col + 1 < self.0[0].len() {
                        self._count_timelines(current_row + 1, col + 1, cache)
                    } else {
                        0
                    }
                },
            );

            a + b
        };

        cache.insert((row, col), n);
        n
    }
}

impl FromStr for Manifold {
    type Err = Report;

    fn from_str(s: &str) -> Result<Self> {
        s.lines()
            .map(|line| line.chars().map(Space::try_from).try_collect())
            .try_collect()
            .map(Manifold)
    }
}

#[test]
fn example() {
    use pretty_assertions::assert_eq;

    let input = ".......S.......
...............
.......^.......
...............
......^.^......
...............
.....^.^.^.....
...............
....^.^...^....
...............
...^.^...^.^...
...............
..^...^.....^..
...............
.^.^.^.^.^...^.
...............";

    let m = input.parse::<Manifold>().unwrap();
    assert_eq!(m.clone().count_splits(), 21);
    assert_eq!(m.count_timelines(), 40);
}
