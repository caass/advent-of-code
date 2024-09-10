macro_rules! aoc {
    ($year:literal/$day:literal-$part:literal: $answer:literal) => {{
        let input_file = {
            let mut crate_dir: ::std::path::PathBuf = ::std::env!("CARGO_MANIFEST_DIR").parse()?;

            crate_dir.push("tests");
            crate_dir.push("fixtures");
            crate_dir.push($year.to_string());
            crate_dir.push($day.to_string());
            crate_dir
        };

        let input = ::std::fs::read_to_string(input_file)?;

        let year =
            <::advent_of_code::types::Year as ::std::convert::TryFrom<u16>>::try_from($year)?;
        let day = <::advent_of_code::types::Day as ::std::convert::TryFrom<u8>>::try_from($day)?;
        let part = <::advent_of_code::types::Part as ::std::convert::TryFrom<u8>>::try_from($part)?;

        let f = ::advent_of_code::AOC
            .get(year, day, part)
            .ok_or_else(|| ::eyre::eyre!("Haven't solved part {part} of {year} day {day} yet."))?;

        let output = (f)(input.trim());

        ::pretty_assertions::assert_eq!(output, $answer.to_string());

        Ok(())
    }};
}

pub(crate) use aoc;
