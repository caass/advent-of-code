use crate::types::ProblemSet;

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

mod_days!(1, 2, 3);

pub const PROBLEMS: ProblemSet = ProblemSet([
    Some(NOT_QUITE_LISP),
    Some(I_WAS_TOLD_THERE_WOULD_BE_NO_MATH),
    Some(PERFECTLY_SPHERICAL_HOUSES_IN_A_VACUUM),
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
]);
