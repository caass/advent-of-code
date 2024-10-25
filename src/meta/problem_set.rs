use std::ops::Index;

use crate::meta::{Day, Problem};

#[repr(transparent)]
pub struct ProblemSet([Option<Problem>; 25]);

impl ProblemSet {
    #[inline]
    pub const fn day(&self, day: Day) -> Option<&Problem> {
        let idx = day.as_u8() as usize - 1;
        self.0[idx].as_ref()
    }

    pub fn days(&self) -> impl Iterator<Item = (Day, &Problem)> {
        Day::iter().flat_map(|day| self.day(day).map(|problem| (day, problem)))
    }

    pub(crate) const fn with_day(mut self, day: Day, problem: Problem) -> Self {
        let idx = day.as_u8() as usize - 1;
        self.0[idx] = Some(problem);
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

    #[inline]
    fn index(&self, day: Day) -> &Self::Output {
        self.day(day)
            .unwrap_or_else(|| panic!("Haven't solved either part of day {day}"))
    }
}

/// Helper macro to create a [`ProblemSet`] from a series of [`Problems`].
/// ```compile_fail
/// // Equivalent to:
/// // PROBLEMS! {
/// //     01 => SOME_PROBLEM,
/// //     02 => ANOTHER_PROBLEM,
/// //     // ...
/// //     25 => THE_FINAL_PROBLEM
/// // }
///
/// #[path = "01.rs"]
/// mod day1;
///
/// #[path = "02.rs"]
/// mod day2;
///
/// // ...
///
/// #[path = "25.rs"]
/// mod day25;
///
/// pub const PROBLEMS: ProblemSet = ProblemSet::new()
///     .with_day(Day::One, day1::SOME_PROBLEM)
///     .with_day(Day::Two, day2::ANOTHER_PROBLEM)
///     // ...
///     .with_day(Day::TwentyFive, day25::THE_FINAL_PROBLEM);
/// ```
macro_rules! PROBLEMS {
    {$($day:literal => $problem:ident),+} => {
        ::paste::paste!{
            $(
                #[path = "" $day ".rs"]
                mod [<day $day>];
            )+

            pub const PROBLEMS: crate::meta::ProblemSet = const {
                let problems = crate::meta::ProblemSet::new();

                $(
                    #[allow(clippy::zero_prefixed_literal)]
                    let Ok(day_index) = crate::meta::Day::from_u8($day) else {
                        ::std::panic!("Invalid day");
                    };
                    let problems = problems.with_day(day_index, self::[<day $day>]::$problem);
                )+

                problems
            };
        }
    };
}

pub(crate) use PROBLEMS;
