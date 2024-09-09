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

mod_days!(1, 2);

pub const PROBLEMS: ProblemSet = ProblemSet([
    Some(NOT_QUITE_LISP),
    Some(I_WAS_TOLD_THERE_WOULD_BE_NO_MATH),
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
    None,
]);
