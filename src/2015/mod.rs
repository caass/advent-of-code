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

mod_days!(1);

pub const PROBLEMS: ProblemSet = ProblemSet([
    Some(NOT_QUITE_LISP),
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
    None,
]);
