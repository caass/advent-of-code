/// Define an advent of code test case using the syntax
/// ```
/// aoc_test!(year/day-part: "answer");
/// ```
macro_rules! test {
    ($year:literal/$day:literal-$part:literal: $answer:literal) => {
        paste::paste! {
            #[test]
            fn [<aoc_ $year _day_ $day _part_ $part>]() -> ::eyre::Result<()> {
                let input_file = {
                    let mut crate_dir: ::std::path::PathBuf = ::std::env!("CARGO_MANIFEST_DIR")
                        .parse()?;

                    crate_dir.push("tests");
                    crate_dir.push("fixtures");
                    crate_dir.push($year.to_string());
                    crate_dir.push($day.to_string());
                    crate_dir
                };

                let input = ::std::fs::read_to_string(input_file)?;

                let year = <::advent_of_code::types::Year as ::std::convert::TryFrom<u16>>::try_from($year)?;
                let day = <::advent_of_code::types::Day as ::std::convert::TryFrom<u8>>::try_from($day)?;
                let part = <::advent_of_code::types::Part as ::std::convert::TryFrom<u8>>::try_from($part)?;

                let problem = ::advent_of_code::AOC[year][day].as_ref()
                    .ok_or_else(|| ::eyre::eyre!("Haven't solved {year} day {day} yet"))?;
                let f = problem[part]
                    .ok_or_else(|| ::eyre::eyre!("Haven't solved part {part} of {year} day {day} yet."))?;

                let output = (f)(input.trim());

                ::pretty_assertions::assert_eq!(output, $answer.to_string());

                Ok(())
            }
        }
    };
}

pub(crate) use test;
