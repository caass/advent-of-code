use aoc_meta::{AdventOfCode, Year};

pub const AOC: AdventOfCode = AdventOfCode::new()
    .with_year(Year::Fifteen, aoc_2015::PROBLEMS)
    .with_year(Year::Sixteen, aoc_2016::PROBLEMS)
    .with_year(Year::Seventeen, aoc_2017::PROBLEMS)
    .with_year(Year::Eighteen, aoc_2018::PROBLEMS)
    .with_year(Year::Nineteen, aoc_2019::PROBLEMS)
    .with_year(Year::Twenty, aoc_2020::PROBLEMS)
    .with_year(Year::TwentyOne, aoc_2021::PROBLEMS)
    .with_year(Year::TwentyTwo, aoc_2022::PROBLEMS)
    .with_year(Year::TwentyThree, aoc_2023::PROBLEMS)
    .with_year(Year::TwentyFour, aoc_2024::PROBLEMS);
