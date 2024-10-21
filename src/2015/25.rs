use std::{
    fmt::{self, Display, Formatter},
    num::NonZeroUsize,
    str::FromStr,
};

use eyre::{eyre, Report, Result};

use crate::meta::Problem;

/// https://adventofcode.com/2015/day/25
pub const LET_IT_SNOW: Problem =
    Problem::partially_solved(&|input| input.parse().map(|coord: Coordinate| coord.value()));

#[derive(Debug)]
struct Coordinate {
    row: NonZeroUsize,
    column: NonZeroUsize,
}

impl Display for Coordinate {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "({}, {})", self.column, self.row)
    }
}

impl Coordinate {
    const fn new(row: usize, column: usize) -> Option<Self> {
        let Some(row) = NonZeroUsize::new(row) else {
            return None;
        };
        let Some(column) = NonZeroUsize::new(column) else {
            return None;
        };

        Some(Self { row, column })
    }

    /// Convert the coordinates from (row, column) form to index form given that the grid is filled in according to the
    /// problem description (i.e. diagonally upwards from left to right, beginning at the top left square). The returned
    /// value is 1-indexed (the first value is 1).
    const fn to_index(&self) -> usize {
        // First, notice that the numbers in the first row are the triangular numbers (https://oeis.org/A000217), but
        // with zero omitted -- that is, f(x, 1) = x(x+1)/2 holds for:
        //
        // f(1, 1) = 1
        // f(2, 1) = 3
        // f(3, 1) = 6
        // f(4, 1) = 10
        // f(5, 1) = 15
        // f(6, 1) = 21
        //
        // Second, notice that the values in each successive row look similar to the triangular numbers, but with some
        // expression substituted for x such that the final values are offset by some amount. For example:
        //
        // f(x, 2) = (x)(x + 1)/2 + 2
        // f(x, 3) = (x + 1)(x + 2)/2 + 3
        // f(x, 4) = (x + 2)(x + 3)/2 + 4
        //
        // Backporting this pattern to y = 1, we also get
        // f(x, 1) = (x - 1)(x)/2 + 1, which is equal to x(x+1)/2 when x = 1
        //
        // Therefore, the general form is f(x, y) = (x + y - 2)(x + y - 1)/2 + x
        let x = self.column.get();
        let y = self.row.get();

        (x + y - 2) * (x + y - 1) / 2 + x
    }

    fn value(&self) -> usize {
        Values::new()
            .nth(self.to_index() - 1)
            .expect("`Values` to produce infinite elements")
    }
}

struct Values {
    current: usize,
}

impl Values {
    #[inline(always)]
    const fn new() -> Self {
        Values { current: 20151125 }
    }
}

impl Iterator for Values {
    type Item = usize;

    #[inline(always)]
    fn next(&mut self) -> Option<Self::Item> {
        let current = self.current;
        self.current = (current * 252533) % 33554393;
        Some(current)
    }
}

impl FromStr for Coordinate {
    type Err = Report;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (row, column) = s
            .trim()
            .trim_start_matches(
                "To continue, please consult the code grid in the manual.  Enter the code at row ",
            )
            .trim_end_matches('.')
            .split_once(", column ")
            .ok_or_else(|| eyre!("Failed to parse input: {s}"))?;

        Coordinate::new(row.parse()?, column.parse()?)
            .ok_or_else(|| eyre!("Entries are 1-indexed, encountered zero in ({column}, {row})."))
    }
}

#[cfg(test)]
mod test {
    use super::Coordinate;

    macro_rules! coordinate_to_index {
        (($row:literal, $column:literal), $index:literal) => {
            ::paste::paste!{
                #[test]
                fn [<coordinate_to_index_ $row _ $column >]() {
                    let coord = Coordinate::new($row, $column).unwrap();
                    let actual_index = coord.to_index();
                    assert_eq!(
                        actual_index, $index,
                        "Expected coordinate at {coord} to have index {}, but got {actual_index}.", $index
                    );
                }
            }
        };
    }

    //    | 1   2   3   4   5   6
    // ---+---+---+---+---+---+---+
    // 1  | 1   3   6  10  15  21
    // 2  | 2   5   9  14  20
    // 3  | 4   8  13  19
    // 4  | 7  12  18
    // 5  | 11  17
    // 6  | 16

    coordinate_to_index!((1, 1), 1);
    coordinate_to_index!((2, 1), 2);
    coordinate_to_index!((1, 2), 3);
    coordinate_to_index!((3, 1), 4);
    coordinate_to_index!((2, 2), 5);
    coordinate_to_index!((1, 3), 6);
    coordinate_to_index!((4, 1), 7);
    coordinate_to_index!((3, 2), 8);
    coordinate_to_index!((2, 3), 9);
    coordinate_to_index!((1, 4), 10);
    coordinate_to_index!((5, 1), 11);
    coordinate_to_index!((4, 2), 12);
    coordinate_to_index!((3, 3), 13);
    coordinate_to_index!((2, 4), 14);
    coordinate_to_index!((1, 5), 15);
    coordinate_to_index!((6, 1), 16);
    coordinate_to_index!((5, 2), 17);
    coordinate_to_index!((4, 3), 18);
    coordinate_to_index!((3, 4), 19);
    coordinate_to_index!((2, 5), 20);
    coordinate_to_index!((1, 6), 21);

    macro_rules! coordinate_to_value {
        (($row:literal, $column:literal), $value:literal) => {
            ::paste::paste!{
                #[test]
                fn [<coordinate_to_value_ $row _ $column >]() {
                    let coord = Coordinate::new($row, $column).unwrap();
                    let actual_value = coord.value();
                    assert_eq!(
                        actual_value, $value,
                        "Expected coordinate at {coord} to have value {}, but got {actual_value}.", $value
                    );
                }
            }
        };
    }

    //    |    1         2         3         4         5         6
    // ---+---------+---------+---------+---------+---------+---------+
    //  1 |20151125  18749137  17289845  30943339  10071777  33511524
    //  2 |31916031  21629792  16929656   7726640  15514188   4041754
    //  3 |16080970   8057251   1601130   7981243  11661866  16474243
    //  4 |24592653  32451966  21345942   9380097  10600672  31527494
    //  5 |   77061  17552253  28094349   6899651   9250759  31663883
    //  6 |33071741   6796745  25397450  24659492   1534922  27995004

    coordinate_to_value!((1, 1), 20151125);
    coordinate_to_value!((2, 1), 31916031);
    coordinate_to_value!((3, 1), 16080970);
    coordinate_to_value!((4, 1), 24592653);
    coordinate_to_value!((5, 1), 77061);
    coordinate_to_value!((6, 1), 33071741);

    coordinate_to_value!((1, 2), 18749137);
    coordinate_to_value!((2, 2), 21629792);
    coordinate_to_value!((3, 2), 8057251);
    coordinate_to_value!((4, 2), 32451966);
    coordinate_to_value!((5, 2), 17552253);
    coordinate_to_value!((6, 2), 6796745);

    coordinate_to_value!((1, 3), 17289845);
    coordinate_to_value!((2, 3), 16929656);
    coordinate_to_value!((3, 3), 1601130);
    coordinate_to_value!((4, 3), 21345942);
    coordinate_to_value!((5, 3), 28094349);
    coordinate_to_value!((6, 3), 25397450);

    coordinate_to_value!((1, 4), 30943339);
    coordinate_to_value!((2, 4), 7726640);
    coordinate_to_value!((3, 4), 7981243);
    coordinate_to_value!((4, 4), 9380097);
    coordinate_to_value!((5, 4), 6899651);
    coordinate_to_value!((6, 4), 24659492);

    coordinate_to_value!((1, 5), 10071777);
    coordinate_to_value!((2, 5), 15514188);
    coordinate_to_value!((3, 5), 11661866);
    coordinate_to_value!((4, 5), 10600672);
    coordinate_to_value!((5, 5), 9250759);
    coordinate_to_value!((6, 5), 1534922);

    coordinate_to_value!((1, 6), 33511524);
    coordinate_to_value!((2, 6), 4041754);
    coordinate_to_value!((3, 6), 16474243);
    coordinate_to_value!((4, 6), 31527494);
    coordinate_to_value!((5, 6), 31663883);
    coordinate_to_value!((6, 6), 27995004);
}
