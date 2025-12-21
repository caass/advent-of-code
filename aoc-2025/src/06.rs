use std::str::FromStr;

use eyre::{Report, Result, bail};
use rayon::prelude::*;

use aoc_meta::Problem;

pub const TRASH_COMPACTOR: Problem =
    Problem::partially_solved(&|input| input.parse().map(Homework::solve_human));

#[derive(Debug)]
struct HomeworkProblem {
    numbers: Vec<u64>,
    operation: fn(u64, u64) -> u64,
}

impl HomeworkProblem {
    fn solve_human(self) -> u64 {
        self.numbers
            .into_iter()
            .reduce(self.operation)
            .unwrap_or_default()
    }
}

#[derive(Debug)]
struct Homework {
    problems: Vec<HomeworkProblem>,
}

impl Homework {
    fn solve_human(self) -> u64 {
        self.solve(HomeworkProblem::solve_human)
    }

    fn solve<F: Fn(HomeworkProblem) -> u64 + Send + Sync>(self, f: F) -> u64 {
        self.problems.into_par_iter().map(f).sum()
    }
}

impl FromStr for Homework {
    type Err = Report;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut problems = Vec::new();
        let mut lines = s.lines();

        let Some(operations) = lines.next_back() else {
            bail!("empty homework")
        };

        for op in operations.split_ascii_whitespace() {
            let operation = match op {
                "*" => |a, b| a * b,
                "+" => |a, b| a + b,
                other => bail!("unknown operation \"{other}\""),
            };

            problems.push(HomeworkProblem {
                numbers: vec![],
                operation,
            });
        }

        for line in lines {
            for (i, number) in line.split_ascii_whitespace().enumerate() {
                problems[i].numbers.push(number.parse()?);
            }
        }

        Ok(Homework { problems })
    }
}
