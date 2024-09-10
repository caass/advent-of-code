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
