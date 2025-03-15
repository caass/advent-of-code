//! Utilities for integration testing.

/// Helper macro for more easily writing advent of code integration tests.
macro_rules! tests {
    {$($year:literal: {$($day:literal: [$($(#[$attrs:meta])* $part:literal),+]),*}),+} => {
        ::paste::paste!{
            $(
                mod [<_ $year>] {
                    $(
                        crate::util::tests_impl!(year: $year, day: $day, parts: [$($(#[$attrs])* $part),+]);
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
    (year: $year:literal, day: $day:literal, parts: [$(#[$attrs1:meta])* $part1:literal, $(#[$attrs2:meta])* $part2:literal]) => {
        ::paste::paste!{
            mod [<day $day>] {
                crate::util::tests_impl!(year: $year, day: $day, part: 1, solution: $part1, meta: $($attrs1)*);
                crate::util::tests_impl!(year: $year, day: $day, part: 2, solution: $part2, meta: $($attrs2)*);
            }
        }
    };

    // Construct a module named `d{$day}` that contains a test for part 1 of a given day
    (year: $year:literal, day: $day:literal, parts: [$(#[$attrs:meta])* $part1:literal]) => {
        ::paste::paste!{
            mod [<day $day>] {
                crate::util::tests_impl!(year: $year, day: $day, part: 1, solution: $part1, meta: $($attrs)*);
            }
        }
    };

    // Construct a test named `p{$part}` that checks for the given solution
    (year: $year:literal, day: $day:literal, part: $part:literal, solution: $solution:literal, meta: $($attrs:meta)*) => {
        ::paste::paste!{
            $(#[$attrs])*
            #[test]
            fn [<part $part>]() {
                let two_digit_day = if $day < 10 {
                    ::std::concat!("0", ::std::stringify!($day))
                } else {
                    ::std::stringify!($day)
                };

                // Read input from ENV in CI, or from disk locally.
                let input = {
                    let input_file = {
                        let mut crate_dir: ::std::path::PathBuf =
                            ::std::env!("CARGO_MANIFEST_DIR").parse().unwrap();

                        crate_dir.push("tests");
                        crate_dir.push("inputs");
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

                let output = ::advent_of_code::AOC[year][day][part].solve(input.trim()).unwrap();

                ::pretty_assertions::assert_eq!(output, $solution.to_string());
            }
        }
    };
}

pub(crate) use tests;
#[doc(hidden)]
pub(crate) use tests_impl;
