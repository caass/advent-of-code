//! Utilities for integration testing.

#[doc(hidden)]
pub use pastey::paste;

/// Helper macro for more easily writing advent of code integration tests.
///
/// # Example
///
/// ```ignore
/// aoc_test::tests!(2015, {
///     1: [138, 1771],
///     2: [1598415, 3812909],
/// });
/// ```
#[macro_export]
macro_rules! tests {
    ($year:literal, {$($day:literal: [$($(#[$attrs:meta])* $part:literal),+]),*$(,)?}) => {
        $($crate::tests_impl!(year: $year, day: $day, parts: [$($(#[$attrs])* $part),+]);)*
    };
}

#[doc(hidden)]
#[macro_export]
/// Implementation detail of the [`tests!`] macro.
macro_rules! tests_impl {
    // Since there's no `.enumerate()` for macro iterations (as far as i can tell), the following is helper code
    // to generate the appropriate test cases for days where either 1 or 2 parts have been solved.

    // Construct a module named `d{$day}` that contains tests for both parts of a given day
    (year: $year:literal, day: $day:literal, parts: [$(#[$attrs1:meta])* $part1:literal, $(#[$attrs2:meta])* $part2:literal]) => {
        $crate::paste!{
            mod [<day $day>] {
                $crate::tests_impl!(year: $year, day: $day, part: 1, solution: $part1, meta: $($attrs1)*);
                $crate::tests_impl!(year: $year, day: $day, part: 2, solution: $part2, meta: $($attrs2)*);
            }
        }
    };

    // Construct a module named `d{$day}` that contains a test for part 1 of a given day
    (year: $year:literal, day: $day:literal, parts: [$(#[$attrs:meta])* $part1:literal]) => {
        $crate::paste!{
            mod [<day $day>] {
                $crate::tests_impl!(year: $year, day: $day, part: 1, solution: $part1, meta: $($attrs)*);
            }
        }
    };

    // Construct a test named `p{$part}` that checks for the given solution
    (year: $year:literal, day: $day:literal, part: $part:literal, solution: $solution:literal, meta: $($attrs:meta)*) => {
        $crate::paste!{
            $(#[$attrs])*
            #[test]
            fn [<part $part>]() {
                let two_digit_day = if $day < 10 {
                    ::std::concat!("0", ::std::stringify!($day))
                } else {
                    ::std::stringify!($day)
                };

                let input = {
                    let input_file = {
                        let mut crate_dir: ::std::path::PathBuf =
                            ::std::env!("CARGO_MANIFEST_DIR").parse().unwrap();

                        crate_dir.pop();
                        crate_dir.push("target");
                        crate_dir.push("inputs");
                        crate_dir.push(stringify!($year));
                        crate_dir.push(two_digit_day);

                        crate_dir
                    };

                    ::std::fs::read_to_string(input_file).unwrap()
                };

                let year = <::aoc_meta::Year as ::std::convert::TryFrom<u16>>::try_from($year)
                    .unwrap();
                let day =
                    <::aoc_meta::Day as ::std::convert::TryFrom<u8>>::try_from($day).unwrap();
                let part =
                    <::aoc_meta::Part as ::std::convert::TryFrom<u8>>::try_from($part).unwrap();

                let output = ::[<aoc_ $year>]::PROBLEMS[day][part].solve(input.trim_end()).unwrap();

                ::pretty_assertions::assert_eq!(output, $solution.to_string());
            }
        }
    };
}
