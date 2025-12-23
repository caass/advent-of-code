use std::fmt::{self, Display, Formatter, Write};
use std::str::FromStr;

use deranged::OptionRangedU8;
use eyre::{OptionExt, Report, Result, bail};
use itertools::Itertools;
use rayon::prelude::*;

use aoc_meta::Problem as AocProblem;

pub const TRASH_COMPACTOR: AocProblem = AocProblem::solved(
    &|input| input.parse::<Homework>().map(|hw| hw.solve_human()),
    &|input| input.parse::<Homework>().map(|hw| hw.solve_cephalopod()),
);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct Digit(OptionRangedU8<0, 9>);

impl Digit {
    fn get(&self) -> Option<u8> {
        self.0.get_primitive()
    }
}

#[cfg(test)]
impl Digit {
    const ONE: Digit = Digit(OptionRangedU8::Some(deranged::RangedU8::new_static::<1>()));
    const TWO: Digit = Digit(OptionRangedU8::Some(deranged::RangedU8::new_static::<2>()));
    const THREE: Digit = Digit(OptionRangedU8::Some(deranged::RangedU8::new_static::<3>()));
    const FOUR: Digit = Digit(OptionRangedU8::Some(deranged::RangedU8::new_static::<4>()));
    const FIVE: Digit = Digit(OptionRangedU8::Some(deranged::RangedU8::new_static::<5>()));
    const SIX: Digit = Digit(OptionRangedU8::Some(deranged::RangedU8::new_static::<6>()));
    const SEVEN: Digit = Digit(OptionRangedU8::Some(deranged::RangedU8::new_static::<7>()));
    const EIGHT: Digit = Digit(OptionRangedU8::Some(deranged::RangedU8::new_static::<8>()));
    const NINE: Digit = Digit(OptionRangedU8::Some(deranged::RangedU8::new_static::<9>()));
    const SPACE: Digit = Digit(OptionRangedU8::None);
}

impl Display for Digit {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let Some(digit) = self.0.get_primitive() else {
            return f.write_char(' ');
        };

        Display::fmt(&digit, f)
    }
}

impl TryFrom<char> for Digit {
    type Error = Report;

    fn try_from(ch: char) -> Result<Self> {
        if ch == ' ' {
            Ok(Digit(OptionRangedU8::None))
        } else {
            let digit = ch.to_digit(10).ok_or_eyre("not a digit")?;
            let u8_digit = u8::try_from(digit)?.try_into()?;

            Ok(Digit(OptionRangedU8::Some(u8_digit)))
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Operator {
    Add,
    Mul,
}

impl Operator {
    fn apply(self, a: u64, b: u64) -> u64 {
        match self {
            Operator::Add => a + b,
            Operator::Mul => a * b,
        }
    }
}

impl Display for Operator {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_char(match *self {
            Operator::Add => '+',
            Operator::Mul => '*',
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Problem {
    grid: Vec<Vec<Digit>>,
    op: Operator,
}

impl Problem {
    fn solve_human(&self) -> u64 {
        self.grid
            .iter()
            .map(|row| {
                row.iter()
                    .filter_map(Digit::get)
                    .fold(0u64, |acc, digit| acc * 10 + digit as u64)
            })
            .reduce(|a, b| self.op.apply(a, b))
            .unwrap_or_default()
    }

    fn solve_cephalopod(&self) -> u64 {
        let Some(len) = self.grid.first().map(Vec::len) else {
            return 0;
        };

        (0..len)
            .rev()
            .map(|x| {
                self.grid
                    .iter()
                    .map(|row| &row[x])
                    .filter_map(Digit::get)
                    .fold(0u64, |acc, digit| acc * 10 + digit as u64)
            })
            .reduce(|a, b| self.op.apply(a, b))
            .unwrap_or_default()
    }
}

impl Display for Problem {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        for line in &self.grid {
            for digit in line {
                Display::fmt(digit, f)?;
            }

            f.write_char('\n')?;
        }

        writeln!(f, "{}\n", &self.op)
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
struct Homework {
    problems: Vec<Problem>,
}

impl Homework {
    fn solve<F: Fn(&Problem) -> u64 + Send + Sync>(&self, f: F) -> u64 {
        self.problems.par_iter().map(f).sum()
    }

    fn solve_human(&self) -> u64 {
        self.solve(Problem::solve_human)
    }

    fn solve_cephalopod(&self) -> u64 {
        self.solve(Problem::solve_cephalopod)
    }
}

impl FromStr for Homework {
    type Err = Report;

    fn from_str(s: &str) -> Result<Self> {
        if !s.is_ascii() {
            bail!("expected string to be ascii");
        }

        let mut lines = s.lines();

        let Some(op_line) = lines.next_back() else {
            return Ok(Homework::default());
        };

        // Collect all operators with their positions
        let operators = op_line
            .char_indices()
            .filter_map(|(i, ch)| {
                let op = match ch {
                    '+' => Operator::Add,
                    '*' => Operator::Mul,
                    _ => return None,
                };
                Some((i, op))
            })
            .collect_vec();

        // Build problems with ranges (use usize::MAX for last; clamped to line.len() below)
        let mut problems = Vec::with_capacity(operators.len());
        let mut problem_indices = Vec::with_capacity(operators.len());

        for (idx, &(pos, op)) in operators.iter().enumerate() {
            let end = operators
                .get(idx + 1)
                .map(|(next_pos, _)| next_pos - 1) // Exclude separator column
                .unwrap_or(usize::MAX);

            problem_indices.push(pos..end);
            problems.push(Problem { grid: vec![], op });
        }

        for line in lines {
            for (i, idx) in problem_indices.iter().cloned().enumerate() {
                // Handle lines shorter than the range
                let start = idx.start.min(line.len());
                let end = idx.end.min(line.len());
                let digits = line[start..end]
                    .chars()
                    .map(Digit::try_from)
                    .try_collect()?;
                problems[i].grid.push(digits);
            }
        }

        Ok(Homework { problems })
    }
}

#[test]
fn example() {
    use pretty_assertions::assert_eq;

    let input = "123 328  51 64 
 45 64  387 23 
  6 98  215 314
*   +   *   +  ";

    let hw = input.parse::<Homework>().unwrap();
    assert_eq!(
        hw,
        Homework {
            problems: vec![
                Problem {
                    grid: vec![
                        vec![Digit::ONE, Digit::TWO, Digit::THREE],
                        vec![Digit::SPACE, Digit::FOUR, Digit::FIVE],
                        vec![Digit::SPACE, Digit::SPACE, Digit::SIX],
                    ],
                    op: Operator::Mul,
                },
                Problem {
                    grid: vec![
                        vec![Digit::THREE, Digit::TWO, Digit::EIGHT],
                        vec![Digit::SIX, Digit::FOUR, Digit::SPACE],
                        vec![Digit::NINE, Digit::EIGHT, Digit::SPACE],
                    ],
                    op: Operator::Add,
                },
                Problem {
                    grid: vec![
                        vec![Digit::SPACE, Digit::FIVE, Digit::ONE],
                        vec![Digit::THREE, Digit::EIGHT, Digit::SEVEN],
                        vec![Digit::TWO, Digit::ONE, Digit::FIVE],
                    ],
                    op: Operator::Mul,
                },
                Problem {
                    grid: vec![
                        vec![Digit::SIX, Digit::FOUR, Digit::SPACE],
                        vec![Digit::TWO, Digit::THREE, Digit::SPACE],
                        vec![Digit::THREE, Digit::ONE, Digit::FOUR],
                    ],
                    op: Operator::Add,
                },
            ]
        }
    );

    assert_eq!(hw.solve_human(), 4277556);
    assert_eq!(hw.solve_cephalopod(), 3263827);
}
