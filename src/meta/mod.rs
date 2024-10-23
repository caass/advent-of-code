use std::ops::Index;

use eyre::Result;

mod indices;
mod problem;
mod problem_set;
mod solution;

pub use self::indices::{Day, Part, Year};
pub(crate) use self::problem::Problem;
pub(crate) use self::problem_set::{ProblemSet, PROBLEMS};
use self::solution::Solution;

#[repr(transparent)]
pub struct AdventOfCode([Option<ProblemSet>; const { (Year::LAST - Year::FIRST + 1) as usize }]);

impl AdventOfCode {
    #[inline(always)]
    pub(crate) fn solve(
        &self,
        year: Year,
        day: Day,
        part: Part,
        input: impl AsRef<str>,
    ) -> Option<Result<String>> {
        Some(self.year(year)?.day(day)?.part(part)?.solve(input.as_ref()))
    }

    #[inline(always)]
    pub const fn year(&self, year: Year) -> Option<&ProblemSet> {
        let idx = (year.as_u16() - Year::FIRST) as usize;
        self.0[idx].as_ref()
    }

    pub fn years(&self) -> impl Iterator<Item = (Year, &ProblemSet)> {
        Year::iter().flat_map(|year| self.year(year).map(|set| (year, set)))
    }

    pub(crate) const fn with_year(mut self, year: Year, problems: ProblemSet) -> Self {
        let idx = (year.as_u16() - Year::FIRST) as usize;
        self.0[idx] = Some(problems);
        self
    }

    pub(crate) const fn new() -> Self {
        Self([None, None, None, None, None, None, None, None, None])
    }
}

impl Index<Year> for AdventOfCode {
    type Output = ProblemSet;

    #[inline(always)]
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

            #[cfg_attr(feature = "wasm", wasm_bindgen::prelude::wasm_bindgen(js_name = "aoc"))]
            pub fn wasm_bindgen_aoc(year: u16, day: u8, part: u8, input: String) -> Result<Option<String>, String> {
                let year = crate::meta::Year::try_from(year).map_err(|e| e.to_string())?;
                let day = crate::meta::Day::try_from(day).map_err(|e| e.to_string())?;
                let part = crate::meta::Part::try_from(part).map_err(|e| e.to_string())?;
                AOC.solve(year, day, part, input).transpose().map_err(|e| e.to_string())
            }
        }
    };
}

pub(crate) use AOC;
