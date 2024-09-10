mod day;
mod part;
mod year;

pub use day::{Day, ParseDayErr};
pub use part::{ParsePartError, Part};
pub use year::{ParseYearError, Year};

pub struct AdventOfCode(pub(crate) phf::Map<u16, ProblemSet>);

impl AdventOfCode {
    #[inline(always)]
    pub fn year(&self, year: Year) -> Option<&ProblemSet> {
        self.0.get(&year.as_u16())
    }

    #[inline(always)]
    pub fn get(&self, year: Year, day: Day, part: Part) -> Option<ProblemPart> {
        self.year(year)
            .and_then(|set| set.day(day))
            .and_then(|problem| problem.part(part))
    }
}

pub struct ProblemSet(pub(crate) phf::Map<u8, Problem>);

impl ProblemSet {
    #[inline(always)]
    pub fn day(&self, day: Day) -> Option<&Problem> {
        self.0.get(&day.as_u8())
    }
}

pub struct Problem(Option<ProblemPart>, Option<ProblemPart>);

impl Problem {
    #[inline(always)]
    pub fn part(&self, part: Part) -> Option<fn(&str) -> String> {
        match part {
            Part::ONE => self.0,
            Part::TWO => self.1,
        }
    }

    pub(crate) const fn new(part1: Option<ProblemPart>, part2: Option<ProblemPart>) -> Self {
        Self(part1, part2)
    }
}

pub type ProblemPart = fn(&str) -> String;

macro_rules! problem {
    ($part1:ident) => {
        Problem::new(Some(|input| $part1(input).to_string()), None)
    };

    ($part1:ident, $part2:ident) => {
        Problem::new(
            Some(|input| $part1(input).to_string()),
            Some(|input| $part2(input).to_string()),
        )
    };
}

pub(crate) use problem;
