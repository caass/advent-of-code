use std::ops::Index;

mod indices;
mod problem;
mod problem_set;
mod solution;

pub use indices::{Day, Part, Year};
pub(crate) use problem::Problem;
pub(crate) use problem_set::{ProblemSet, PROBLEMS};
use solution::Solution;

#[repr(transparent)]
pub struct AdventOfCode([Option<ProblemSet>; const { (Year::LAST - Year::FIRST + 1) as usize }]);

impl AdventOfCode {
    #[inline]
    #[must_use]
    pub const fn year(&self, year: Year) -> Option<&ProblemSet> {
        let idx = (year.as_u16() - Year::FIRST) as usize;
        self.0[idx].as_ref()
    }

    pub fn years(&self) -> impl Iterator<Item = (Year, &ProblemSet)> {
        Year::iter().filter_map(|year| self.year(year).map(|set| (year, set)))
    }

    pub(crate) const fn with_year(mut self, year: Year, problems: ProblemSet) -> Self {
        let idx = (year.as_u16() - Year::FIRST) as usize;
        self.0[idx] = Some(problems);
        self
    }

    pub(crate) const fn new() -> Self {
        Self([None, None, None, None, None, None, None, None, None, None])
    }
}

impl Index<Year> for AdventOfCode {
    type Output = ProblemSet;

    #[inline]
    fn index(&self, year: Year) -> &Self::Output {
        self.year(year)
            .unwrap_or_else(|| panic!("Haven't solved any problems from {year}"))
    }
}

/// Helper macro to assemble an [`AdventOfCode`] from the given [`Year`]s.
macro_rules! AOC {
    ([$($year:literal),+]) => {
        ::paste::paste!{
            $(
                #[path = "" $year "/mod.rs"]
                mod [<year $year>];
            )+

            pub const AOC: crate::meta::AdventOfCode = const {
                let aoc = crate::meta::AdventOfCode::new();

                $(
                    let Ok(year_index) = crate::meta::Year::from_u16($year) else {
                        ::std::panic!("Invalid year");
                    };
                    let aoc = aoc.with_year(year_index, self::[<year $year>]::PROBLEMS);
                )+

                aoc
            };
        }
    };
}

pub(crate) use AOC;
