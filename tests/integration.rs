util::tests! {
    2015: {
        1: [138, 1771],
        2: [1598415, 3812909],
        3: [2565, 2639],
        4: [346386, 9958218],
        5: [236, 51],
        6: [569999, 17836115],
        7: [16076, 2797],
        8: [1371, 2117],
        9: [251, 898],
        10: [360154, 5103798],
        11: ["vzbxxyzz", "vzcaabcc"],
        12: [111754, 65402],
        13: [709, 668],
        14: [2696, 1084],
        15: [222870, 117936],
        16: [213, 323],
        17: [654, 57],
        18: [821, 886],
        19: [518]
    }
}

mod util {
    //! Utilities for integration testing.

    /// Helper macro for more easily writing advent of code integration tests.
    macro_rules! tests {
    {$($year:literal: {$($day:literal: [$($part:literal),+]),*}),+} => {
        ::paste::paste!{
            $(
                mod [<_ $year>] {
                    $(
                        crate::util::tests_impl!(year: $year, day: $day, parts: [$($part),+]);
                    )*
                }
            )+
        }
    };
}

    /// Implementation detail of the `tests` macro to handle cases where there's either 1 or 2 solved parts of a day.
    #[doc(hidden)]
    macro_rules! tests_impl {
    // Construct a module named `d{$day}` that contains tests for both parts of a given day
    (year: $year:literal, day: $day:literal, parts: [$part1:literal, $part2:literal]) => {
        ::paste::paste!{
            mod [<day $day>] {
                crate::util::tests_impl!(year: $year, day: $day, part: 1, solution: $part1);
                crate::util::tests_impl!(year: $year, day: $day, part: 2, solution: $part2);
            }
        }
    };

    // Construct a module named `d{$day}` that contains a test for part 1 of a given day
    (year: $year:literal, day: $day:literal, parts: [$part1:literal]) => {
        ::paste::paste!{
            mod [<day $day>] {
                crate::util::tests_impl!(year: $year, day: $day, part: 1, solution: $part1);
            }
        }
    };

    // Construct a test named `p{$part}` that checks for the given solution
    (year: $year:literal, day: $day:literal, part: $part:literal, solution: $solution:literal) => {
        ::paste::paste!{
            #[test]
            fn [<part $part>]() {
                let two_digit_day = if $day < 10 {
                    ::std::concat!("0", ::std::stringify!($day))
                } else {
                    ::std::stringify!($day)
                };

                // Read input from ENV in CI, or from disk locally.
                let input = if ::std::env::var("CI").ok().as_deref() == Some("true") {
                    let input_var = ::std::format!("INPUT_{}_{}", $year, two_digit_day);

                    ::std::env::var(&input_var)
                        .unwrap_or_else(|e| ::std::panic!("Error reading {input_var}: {e}"))
                } else {
                    let input_file = {
                        let mut crate_dir: ::std::path::PathBuf =
                            ::std::env!("CARGO_MANIFEST_DIR").parse().unwrap();

                        crate_dir.push("tests");
                        crate_dir.push("fixtures");
                        crate_dir.push(stringify!($year));
                        crate_dir.push(two_digit_day);

                        crate_dir
                    };

                    ::std::fs::read_to_string(input_file).unwrap()
                };

                let year = <::advent_of_code::meta::Year as ::std::convert::TryFrom<u16>>::try_from($year)
                    .unwrap();
                let day =
                    <::advent_of_code::meta::Day as ::std::convert::TryFrom<u8>>::try_from($day).unwrap();
                let part =
                    <::advent_of_code::meta::Part as ::std::convert::TryFrom<u8>>::try_from($part).unwrap();

                let f = ::advent_of_code::AOC
                    .get(year, day, part)
                    .ok_or_else(|| ::eyre::eyre!("Haven't solved part {part} of {year} day {day} yet."))
                    .unwrap();

                let output = (f)(input.trim()).unwrap();

                ::pretty_assertions::assert_eq!(output, $solution.to_string());
            }
        }
    };
}

    pub(crate) use tests;
    pub(crate) use tests_impl;
}
