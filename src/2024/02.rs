use itertools::Itertools;
use rayon::prelude::*;

use crate::meta::Problem;

pub const RED_NOSED_REPORTS: Problem = Problem::partially_solved(&count_safe_levels);

fn count_safe_levels(input: &str) -> usize {
    input
        .par_lines()
        .map(Report::parse)
        .filter_map(|report| report.is_safe().then_some(()))
        .count()
}

#[derive(Debug)]
struct Report<I> {
    levels: I,
}

impl Report<()> {
    fn parse(line: &str) -> Report<impl Iterator<Item = u8> + '_> {
        Report {
            levels: line
                .split_ascii_whitespace()
                .map(|report_str| report_str.parse::<u8>().unwrap()),
        }
    }
}

impl<I: Iterator<Item = u8>> Report<I> {
    fn is_safe(self) -> bool {
        let mut decreases_globally = Option::None;

        self.tuple_windows().all(|(level_1, level_2)| {
            let decreases_locally = level_1 > level_2;
            let slope = level_1.abs_diff(level_2);

            let has_matching_gradient =
                *decreases_globally.get_or_insert(decreases_locally) == decreases_locally;
            let has_safe_gradient = (1..=3).contains(&slope);

            has_matching_gradient && has_safe_gradient
        })
    }
}

impl<I: Iterator<Item = u8>> Iterator for Report<I> {
    type Item = I::Item;

    fn next(&mut self) -> Option<Self::Item> {
        self.levels.next()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.levels.size_hint()
    }
}
