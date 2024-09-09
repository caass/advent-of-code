use std::fmt::{self, Debug, Formatter};
use std::ops::Index;

mod day;
mod part;
mod year;

pub use day::{Day, ParseDayErr};
pub use part::{ParsePartError, Part};
pub use year::{ParseYearError, Year};

pub struct AdventOfCode(pub(crate) [ProblemSet; (Year::LAST - Year::FIRST) as usize + 1]);

impl Debug for AdventOfCode {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_map()
            .entries(Year::iter().map(|year| (year.as_u16(), self.index(year))))
            .finish()
    }
}

impl Index<Year> for AdventOfCode {
    type Output = ProblemSet;

    fn index(&self, index: Year) -> &Self::Output {
        let array_index: usize = (index.as_u16() - Year::FIRST).into();

        // Safety: `array_index` is guaranteed to be between `0` and `Year::LAST - Year::FIRST`
        unsafe { self.0.get_unchecked(array_index) }
    }
}

#[derive(Debug)]
pub struct ProblemSet(pub(crate) [Option<Problem>; 25]);

impl ProblemSet {
    pub(crate) const fn unsolved() -> Self {
        ProblemSet([
            None, None, None, None, None, None, None, None, None, None, None, None, None, None,
            None, None, None, None, None, None, None, None, None, None, None,
        ])
    }
}

impl Index<Day> for ProblemSet {
    type Output = Option<Problem>;

    fn index(&self, index: Day) -> &Self::Output {
        let array_index: usize = (index.as_u8() - 1).into();

        // Safety: `array_index` is guaranteed to be between `0` and `24`
        unsafe { self.0.get_unchecked(array_index) }
    }
}

#[derive(Debug)]
pub struct Problem {
    pub(crate) part_1: Option<ProblemPart>,
    pub(crate) part_2: Option<ProblemPart>,
}

impl Index<Part> for Problem {
    type Output = Option<ProblemPart>;

    fn index(&self, index: Part) -> &Self::Output {
        match index {
            Part::ONE => &self.part_1,
            Part::TWO => &self.part_2,
        }
    }
}

pub type ProblemPart = fn(&str) -> String;
