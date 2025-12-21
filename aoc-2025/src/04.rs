use std::str::FromStr;

use eyre::{Report, Result, bail, eyre};
use rayon::prelude::*;

use aoc_meta::Problem;

pub const PRINTING_DEPARTMENT: Problem = Problem::solved(
    &|input| input.parse::<Grid>().map(|g| g.accessible().count()),
    &|input| -> Result<usize> {
        let mut grid = input.parse::<Grid>()?;
        let mut removed = 0;

        let mut acc = grid.accessible().collect_vec_list();
        while !acc.is_empty() {
            for i in acc.into_iter().flatten() {
                grid.rows[i] = Space::Empty;
                removed += 1;
            }

            acc = grid.accessible().collect_vec_list();
        }

        Ok(removed)
    },
);

#[derive(Debug, Default)]
struct Grid {
    rows: Vec<Space>,
    row_len: usize,
}

impl Grid {
    fn num_rows(&self) -> usize {
        self.rows.len() / self.row_len
    }

    fn accessible(&self) -> impl ParallelIterator<Item = usize> {
        self.rows
            .par_iter()
            .copied()
            .enumerate()
            .filter_map(|(i, space)| {
                if matches!(space, Space::Roll) && {
                    let x = i % self.row_len;
                    let y = i / self.row_len;

                    let neighbor_rolls = self
                        .neighbors(x, y)
                        .filter(|&idx| matches!(self.rows[idx], Space::Roll))
                        .count();

                    neighbor_rolls < 4
                } {
                    Some(i)
                } else {
                    None
                }
            })
    }

    fn neighbors(&self, x: usize, y: usize) -> impl Iterator<Item = usize> + '_ {
        const OFFSETS: [(isize, isize); 8] = [
            (-1, -1),
            (0, -1),
            (1, -1),
            (-1, 0),
            (1, 0),
            (-1, 1),
            (0, 1),
            (1, 1),
        ];

        OFFSETS.into_iter().filter_map(move |(dx, dy)| {
            let nx = x.checked_add_signed(dx)?;
            let ny = y.checked_add_signed(dy)?;

            if nx < self.row_len && ny < self.num_rows() {
                Some(ny * self.row_len + nx)
            } else {
                None
            }
        })
    }
}

impl FromStr for Grid {
    type Err = Report;

    fn from_str(s: &str) -> Result<Self> {
        s.lines()
            .enumerate()
            .map(|(i, line)| {
                (
                    i,
                    line.len(),
                    line.bytes().map(|ch| match ch {
                        b'@' => Ok(Space::Roll),
                        b'.' => Ok(Space::Empty),
                        other => Err(eyre!("unexpected char {other} (expected '@' or '.')")),
                    }),
                )
            })
            .try_fold(
                Grid {
                    rows: Vec::with_capacity(s.len()),
                    row_len: 0,
                },
                |mut grid, (i, row_len, row)| {
                    if grid.row_len == 0 {
                        grid.row_len = row_len
                    }

                    if grid.row_len != row_len {
                        bail!("row {i} had length {row_len} (expected {})", grid.row_len)
                    }

                    for res in row {
                        grid.rows.push(res?);
                    }

                    Ok(grid)
                },
            )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default)]
enum Space {
    #[default]
    Empty,
    Roll,
}
