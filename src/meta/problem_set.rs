use std::ops::Index;

use crate::meta::{Day, Problem};

#[repr(transparent)]
pub struct ProblemSet([Option<Problem>; 25]);

impl ProblemSet {
    #[inline(always)]
    pub const fn day(&self, day: Day) -> Option<&Problem> {
        self.0[day.as_u8() as usize].as_ref()
    }

    pub(crate) const fn with_day(mut self, day: Day, problem: Problem) -> Self {
        self.0[day.as_u8() as usize] = Some(problem);
        self
    }

    pub(crate) const fn new() -> Self {
        Self([
            None, None, None, None, None, None, None, None, None, None, None, None, None, None,
            None, None, None, None, None, None, None, None, None, None, None,
        ])
    }
}

impl Index<Day> for ProblemSet {
    type Output = Problem;

    fn index(&self, day: Day) -> &Self::Output {
        self.day(day)
            .unwrap_or_else(|| panic!("Haven't solved either part of day {day}"))
    }
}

macro_rules! problems {
    ($($day:literal),+) => {
        ::paste::paste!{
            $(
                #[path = "" $day ".rs"]
                mod [<day $day>];
            )+

            #[allow(clippy::zero_prefixed_literal)]
            pub const PROBLEMS: crate::meta::ProblemSet = const {
                let problems = crate::meta::ProblemSet::new();
                $(
                    let Ok(day_index) = crate::meta::Day::from_u8($day) else {
                        ::std::panic!("Invalid day");
                    };
                    let problems = problems.with_day(day_index, [<day $day>]::PROBLEM);
                )+
                problems
            };
        }
    };
}

pub(crate) use problems;
