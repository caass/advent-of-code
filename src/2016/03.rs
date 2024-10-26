use std::str::Lines;

use eyre::{bail, eyre, OptionExt, Result};

use crate::common::BoolExt;
use crate::meta::Problem;

pub const SQUARES_WITH_THREE_SIDES: Problem = Problem::solved(
    &|input| Triangle::horizontals(input).count_valid(),
    &|input| Triangle::verticals(input).count_valid(),
);

#[derive(Debug)]
struct Triangle(u16, u16, u16);

impl Triangle {
    fn is_valid(&self) -> bool {
        self.0 + self.1 > self.2 && self.1 + self.2 > self.0 && self.0 + self.2 > self.1
    }
}

impl Triangle {
    fn verticals(input: &str) -> impl Iterator<Item = Result<Self>> + use<'_> {
        Verticals(input).into_iter()
    }

    fn horizontals(input: &str) -> impl Iterator<Item = Result<Self>> + use<'_> {
        input.lines().map(split_line)
    }
}

struct Verticals<'input>(&'input str);

impl<'input> IntoIterator for Verticals<'input> {
    type Item = Result<Triangle>;

    type IntoIter = VerticalChunks<'input>;

    fn into_iter(self) -> Self::IntoIter {
        VerticalChunks {
            source: self.0.trim().lines(),
            b: None,
            c: None,
        }
    }
}

struct VerticalChunks<'input> {
    source: Lines<'input>,
    b: Option<Triangle>,
    c: Option<Triangle>,
}

impl Iterator for VerticalChunks<'_> {
    type Item = Result<Triangle>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(next) = self.b.take().or_else(|| self.c.take()) {
            return Some(Ok(next));
        }

        let first_line = self.source.next()?;
        let Some(second_line) = self.source.next() else {
            return Some(Err(eyre!(
                "input didn't have enough lines (needed two more)"
            )));
        };
        let Some(third_line) = self.source.next() else {
            return Some(Err(eyre!(
                "input didn't have enough lines (needed one more)"
            )));
        };

        let Triangle(a1, b1, c1) = match split_line(first_line) {
            Ok(ok) => ok,
            Err(e) => return Some(Err(e)),
        };
        let Triangle(a2, b2, c2) = match split_line(second_line) {
            Ok(ok) => ok,
            Err(e) => return Some(Err(e)),
        };
        let Triangle(a3, b3, c3) = match split_line(third_line) {
            Ok(ok) => ok,
            Err(e) => return Some(Err(e)),
        };

        self.b = Some(Triangle(b1, b2, b3));
        self.c = Some(Triangle(c1, c2, c3));

        Some(Ok(Triangle(a1, a2, a3)))
    }
}

fn split_line(line: &str) -> Result<Triangle> {
    let mut parts = line.split_whitespace();

    let a = parts.next().ok_or_eyre("empty input")?.parse()?;
    let b = parts
        .next()
        .ok_or_eyre("unexpected end of input (needed two more numbers)")?
        .parse()?;
    let c = parts
        .next()
        .ok_or_eyre("unexpected end of input (needed one more number)")?
        .parse()?;

    if let Some(rest) = parts.next() {
        bail!("Unexpected token in input: \"{rest}\"");
    }

    Ok(Triangle(a, b, c))
}

trait DimensionsList {
    fn count_valid(self) -> Result<u16>;
}

impl<I> DimensionsList for I
where
    I: Iterator<Item = Result<Triangle>>,
{
    fn count_valid(self) -> Result<u16> {
        self.map(|res| res.map(|dims| dims.is_valid().into_num()))
            .reduce(|a, b| Ok(a? + b?))
            .ok_or_eyre("empty input")?
    }
}
