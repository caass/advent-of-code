use std::{
    fmt::{self, Display, Formatter},
    marker::PhantomData,
    ops::{Bound, Index, IndexMut, RangeBounds},
};

use either::Either;
use itertools::Itertools;
use rayon::{iter::Flatten, prelude::*};

/// A 2-dimensional square grid of `T` with side length `N`.
#[derive(Debug)]
pub struct Grid<const N: usize, T> {
    // N.B.: I think this actually should say "columns"
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
    /// Returns a parallel iterator of `(Coordinate, &mut T)` (where the first item in the tuple is the grid coordinates
    /// of the second item in the tuple) over all the elements in the given range.
    pub fn par_range_mut<R: RangeBounds<Coordinate>>(
        &mut self,
        range: R,
    ) -> impl ParallelIterator<Item = (Coordinate, &mut T)> {
        let range = CoordinateRange::new(range);

        self.rows[range.x]
            .par_iter_mut()
            .enumerate()
            .flat_map(move |(x, row)| {
                row[range.y]
                    .par_iter_mut()
                    .enumerate()
                    .map(move |(y, item)| (Coordinate { x, y }, item))
            })
    }
}

impl<const N: usize, T: Sync> Grid<N, T> {
    /// Retruns a parallel iterator of `(Coordinate, &T)` (where the first item in the tuple is the grid coordinates
    /// of the second item in the tuple) over all the elements in the given range.
    pub fn par_range<R: RangeBounds<Coordinate>>(
        &self,
        range: R,
    ) -> impl ParallelIterator<Item = (Coordinate, &T)> {
        let range = CoordinateRange::new(range);

        self.rows[range.x]
            .par_iter()
            .enumerate()
            .flat_map(move |(x, row)| {
                row[range.y]
                    .par_iter()
                    .enumerate()
                    .map(move |(y, item)| (Coordinate { x, y }, item))
            })
    }
}

impl<const N: usize, T> Grid<N, T> {
    pub fn from_fn<F: FnMut(Coordinate) -> T>(mut f: F) -> Self {
        let rows: [Vec<T>; N] = std::array::from_fn(|x| {
            let row: [T; N] = std::array::from_fn(|y| f(Coordinate { x, y }));
            Vec::from(row)
        });

        Self {
            rows: Vec::from(rows),
            _ghost: PhantomData,
        }
    }

    pub fn get(&self, Coordinate { x, y }: Coordinate) -> Option<&T> {
        self.rows.get(x).and_then(|row| row.get(y))
    }

    pub fn get_mut(&mut self, Coordinate { x, y }: Coordinate) -> Option<&mut T> {
        self.rows.get_mut(x).and_then(|row| row.get_mut(y))
    }

    pub fn neighbors(&self, coordinate: Coordinate) -> impl Iterator<Item = &T> {
        coordinate.neighbors().flat_map(|coord| self.get(coord))
    }
}

impl<const N: usize, T> FromIterator<T> for Grid<N, T> {
    fn from_iter<I: IntoIterator<Item = T>>(into_iter: I) -> Self {
        let mut iter = into_iter.into_iter();
        let grid = Grid::from_fn(|coord| {
            iter.next()
                .unwrap_or_else(|| panic!("Ran out of elements before getting to {coord}"))
        });

        if let Some(_unexpected) = iter.next() {
            panic!("Iterator contained more items than fit in a {N}x{N} grid.");
        }

        grid
    }
}

impl<const N: usize, T: Send> IntoParallelIterator for Grid<N, T> {
    type Iter = Flatten<<Vec<Vec<T>> as IntoParallelIterator>::Iter>;

    type Item = T;

    fn into_par_iter(self) -> Self::Iter {
        self.rows.into_par_iter().flatten()
    }
}

impl<'data, const N: usize, T: Send> IntoParallelIterator for &'data mut Grid<N, T> {
    type Iter = Flatten<<Vec<Vec<T>> as IntoParallelRefMutIterator<'data>>::Iter>;

    type Item = &'data mut T;

    fn into_par_iter(self) -> Self::Iter {
        self.rows.par_iter_mut().flatten()
    }
}

impl<'data, const N: usize, T: Sync> IntoParallelIterator for &'data Grid<N, T> {
    type Iter = Flatten<<Vec<Vec<T>> as IntoParallelRefIterator<'data>>::Iter>;

    type Item = &'data T;

    fn into_par_iter(self) -> Self::Iter {
        self.rows.par_iter().flatten()
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

#[derive(Debug, Clone, Copy)]
pub struct Coordinate {
    pub x: usize,
    pub y: usize,
}

struct CoordinateRange<X: RangeBounds<usize>, Y: RangeBounds<usize>> {
    x: X,
    y: Y,
}

impl CoordinateRange<(Bound<usize>, Bound<usize>), (Bound<usize>, Bound<usize>)> {
    fn new<R: RangeBounds<Coordinate>>(range: R) -> Self {
        Self::from(range)
    }
}

impl<R: RangeBounds<Coordinate>> From<R>
    for CoordinateRange<(Bound<usize>, Bound<usize>), (Bound<usize>, Bound<usize>)>
{
    fn from(range: R) -> Self {
        let x1 = range.start_bound().map(|&Coordinate { x, .. }| x);
        let x2 = range.end_bound().map(|&Coordinate { x, .. }| x);

        let y1 = range.start_bound().map(|&Coordinate { y, .. }| y);
        let y2 = range.end_bound().map(|&Coordinate { y, .. }| y);

        Self {
            x: (x1, x2),
            y: (y1, y2),
        }
    }
}

impl Coordinate {
    fn neighbors(self) -> impl Iterator<Item = Coordinate> {
        let Coordinate { x, y } = self;
        let left_x = x.checked_sub(1);
        let right_x = x.checked_add(1);
        let top_y = y.checked_sub(1);
        let bottom_y = y.checked_add(1);

        let top_left = top_y.and_then(|y| left_x.map(move |x| Coordinate { x, y }));
        let top = top_y.map(|y| Coordinate { x, y });
        let top_right = top_y.and_then(|y| right_x.map(move |x| Coordinate { x, y }));

        let left = left_x.map(|x| Coordinate { x, y });
        let right = right_x.map(|x| Coordinate { x, y });

        let bottom_left = bottom_y.and_then(|y| left_x.map(move |x| Coordinate { x, y }));
        let bottom = bottom_y.map(|y| Coordinate { x, y });
        let bottom_right = bottom_y.and_then(|y| right_x.map(move |x| Coordinate { x, y }));

        [
            top_left,
            top,
            top_right,
            left,
            right,
            bottom_left,
            bottom,
            bottom_right,
        ]
        .into_iter()
        .flatten()
    }
}

impl From<(usize, usize)> for Coordinate {
    fn from((x, y): (usize, usize)) -> Self {
        Self { x, y }
    }
}

impl Display for Coordinate {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "({}, {})", self.x, self.y)
    }
}

impl<const N: usize, T> Index<Coordinate> for Grid<N, T> {
    type Output = T;

    #[inline(always)]
    fn index(&self, index: Coordinate) -> &Self::Output {
        self.get(index).unwrap()
    }
}

impl<const N: usize, T> IndexMut<Coordinate> for Grid<N, T> {
    #[inline(always)]
    fn index_mut(&mut self, index: Coordinate) -> &mut Self::Output {
        self.get_mut(index).unwrap()
    }
}
