use std::ops::Index;

mod indices;
mod problem;
mod problem_set;
mod solution;

pub use indices::{Day, Part, Year};
pub(crate) use problem::Problem;
pub(crate) use problem_set::{problems, ProblemSet};
use solution::Solution;

#[repr(transparent)]
pub struct AdventOfCode([Option<ProblemSet>; const { (Year::LAST - Year::FIRST) as usize }]);

impl AdventOfCode {
    pub const fn get(&self, year: Year, day: Day, part: Part) -> Option<&dyn Solution> {
        let Some(set) = self.year(year) else {
            return None;
        };

        let Some(day) = set.day(day) else { return None };

        day.part(part)
    }

    #[inline(always)]
    pub const fn year(&self, year: Year) -> Option<&ProblemSet> {
        self.0[year.as_u16() as usize].as_ref()
    }

    pub(crate) const fn with_year(mut self, year: Year, problems: ProblemSet) -> Self {
        self.0[year.as_u16() as usize - Year::FIRST as usize] = Some(problems);
        self
    }

    pub(crate) const fn new() -> Self {
        Self([None, None, None, None, None, None, None, None])
    }
}

impl Index<Year> for AdventOfCode {
    type Output = ProblemSet;

    fn index(&self, year: Year) -> &Self::Output {
        self.year(year)
            .unwrap_or_else(|| panic!("Haven't solved any problems from {year}"))
    }
}
