use std::ops::Index;

mod indices;
mod problem;
mod problem_set;
mod solution;

pub use indices::{Day, Part, Year};
pub use problem::Problem;
pub use problem_set::ProblemSet;
pub use solution::Solution;

#[doc(hidden)]
pub use pastey::paste;

#[repr(transparent)]
pub struct AdventOfCode([Option<ProblemSet>; (Year::LAST - Year::FIRST + 1) as usize]);

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

    pub const fn with_year(mut self, year: Year, problems: ProblemSet) -> Self {
        let idx = (year.as_u16() - Year::FIRST) as usize;
        self.0[idx] = Some(problems);
        self
    }

    pub const fn new() -> Self {
        Self([None; (Year::LAST - Year::FIRST + 1) as usize])
    }
}

impl Default for AdventOfCode {
    fn default() -> Self {
        Self::new()
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
