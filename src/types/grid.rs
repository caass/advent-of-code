use std::{
    fmt::{self, Display, Formatter},
    marker::PhantomData,
    ops::RangeBounds,
};

use either::Either;
use itertools::Itertools;
use rayon::{iter::Flatten, prelude::*};

/// A 2-dimensional square grid of `T` with side length `N`.
#[derive(Debug)]
pub struct Grid<const N: usize, T> {
    rows: Vec<Vec<T>>,
    _ghost: PhantomData<[(); N]>,
}

impl<const N: usize, T: Default + Clone> Default for Grid<N, T> {
    fn default() -> Self {
        Self {
            rows: vec![vec![T::default(); N]; N],
            _ghost: PhantomData,
        }
    }
}

impl<const N: usize, T: Send> Grid<N, T> {
    pub fn range_mut<R: RangeBounds<Coordinate>>(
        &mut self,
        range: R,
    ) -> impl ParallelIterator<Item = &mut T> {
        let x1 = range.start_bound().map(|coord| coord.x);
        let x2 = range.end_bound().map(|coord| coord.x);

        let y1 = range.start_bound().map(|coord| coord.y);
        let y2 = range.end_bound().map(|coord| coord.y);

        self.rows[(x1, x2)]
            .par_iter_mut()
            .flat_map(move |row| &mut row[(y1, y2)])
    }
}

impl<const N: usize, T: Sync> Grid<N, T> {
    pub fn range<R: RangeBounds<Coordinate>>(&self, range: R) -> impl ParallelIterator<Item = &T> {
        let x1 = range.start_bound().map(|coord| coord.x);
        let x2 = range.end_bound().map(|coord| coord.x);

        let y1 = range.start_bound().map(|coord| coord.y);
        let y2 = range.end_bound().map(|coord| coord.y);

        self.rows[(x1, x2)]
            .par_iter()
            .flat_map(move |row| &row[(y1, y2)])
    }
}

impl<const N: usize, T: Send> IntoParallelIterator for Grid<N, T> {
    type Iter = Flatten<<Vec<Vec<T>> as IntoParallelIterator>::Iter>;

    type Item = T;

    fn into_par_iter(self) -> Self::Iter {
        self.rows.into_par_iter().flatten()
    }
}

impl<const N: usize, T: Display> Display for Grid<N, T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        self.rows.iter().try_for_each(|row| {
            for item in Itertools::intersperse(row.iter().map(Either::Left), Either::Right(&' ')) {
                either::for_both!(item, t_or_space => Display::fmt(t_or_space, f))?;
            }

            writeln!(f)
        })
    }
}

#[derive(Debug)]
pub struct Coordinate {
    pub x: usize,
    pub y: usize,
}

impl From<(usize, usize)> for Coordinate {
    fn from((x, y): (usize, usize)) -> Self {
        Self { x, y }
    }
}
