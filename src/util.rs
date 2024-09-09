/// Helper macro for importing `$year.rs#PROBLEMS` as `PROBLEMS_$year` when
/// `year` may be a number (e.g. `2020`).
macro_rules! mod_years {
    ($($year:literal),+) => {
        ::paste::paste!(
            $(
                #[path = "" $year "/mod.rs"]
                mod [<_ $year>];
                use [<_ $year>]::PROBLEMS as [<PROBLEMS_ $year>];
            )+

            pub const AOC: crate::types::AdventOfCode = crate::types::AdventOfCode([
                $([<PROBLEMS_ $year>],)+
            ]);
        );
    };
}

/// Helper macro for glob-importing from `$day.rs` where `$day` may be a number (e.g. `16`).
macro_rules! mod_days {
    ($($day:literal),+) => {
        ::paste::paste!(
            $(
                #[path = "" $day ".rs"]
                mod [<day $day>];
                use [<day $day>]::*;
            )+
        );
    };
}

pub(crate) use mod_days;
pub(crate) use mod_years;
