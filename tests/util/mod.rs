/// `proc-macro` to reduce boilerplate in writing tests.
///
/// if the `CI` environment variable is set to `true`, test input will be read from the environment
/// variable `INPUT_$year_$day`. Otherwise, input will be read from `fixtures/$year/$day`.
macro_rules! aoc {
    ($year:literal/$day:literal-$part:literal: $answer:literal) => {
        let two_digit_day = if $day < 10 {
            concat!("0", stringify!($day))
        } else {
            stringify!($day)
        };

        // Read input from ENV in CI, or from disk locally.
        let input = if ::std::env::var("CI").ok().as_deref() == Some("true") {
            let input_var = format!("INPUT_{}_{}", $year, two_digit_day);

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

        ::pretty_assertions::assert_eq!(output, $answer.to_string());
    };
}

pub(crate) use aoc;
