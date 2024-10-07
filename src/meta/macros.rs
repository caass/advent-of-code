/// Construct an advent of code problem with the given parts.
macro_rules! problem {
    () => {
        pub const PROBLEM: crate::meta::Problem = crate::meta::Problem::new(None, None);
    };

    ($part1:expr) => {
        pub const PROBLEM: crate::meta::Problem = crate::meta::Problem::new(
            Some(|input| {
                let output = $part1(input)?;
                Ok(output.to_string())
            }),
            None,
        );
    };

    ($part1:expr, $part2:expr) => {
        pub const PROBLEM: crate::meta::Problem = crate::meta::Problem::new(
            Some(|input| {
                let output = $part1(input)?;
                Ok(output.to_string())
            }),
            Some(|input| {
                let output = $part2(input)?;
                Ok(output.to_string())
            }),
        );
    };
}

/// Construct an advent of code problem set with the given days.
macro_rules! problem_set {
    ($($day:literal),+) => {
        ::paste::paste!(
            $(
                #[path = "" $day ".rs"]
                mod [<day $day>];
            )+

            pub const PROBLEMS: crate::meta::ProblemSet = crate::meta::ProblemSet(::phf::phf_map!{
                $([<$day u8>] => [<day $day>]::PROBLEM,)+
            });
        );
    };
}

pub(crate) use problem;
pub(crate) use problem_set;
