use std::str::Lines;

use eyre::{bail, eyre, OptionExt, Result};

use crate::common::bool_ext::BoolExt;
use crate::meta::Problem;

pub const SQUARES_WITH_THREE_SIDES: Problem = Problem::solved(
    &|input| Dimensions::horizontals(input).count_valid_triangles(),
    &|input| Dimensions::verticals(input).count_valid_triangles(),
);

#[derive(Debug)]
struct Dimensions(u16, u16, u16);

impl Dimensions {
    fn is_valid_triangle(&self) -> bool {
        self.0 + self.1 > self.2 && self.1 + self.2 > self.0 && self.0 + self.2 > self.1
    }
}

impl Dimensions {
    fn verticals(input: &str) -> impl Iterator<Item = Result<Self>> + use<'_> {
        Verticals(input).into_iter()
    }

    fn horizontals(input: &str) -> impl Iterator<Item = Result<Self>> + use<'_> {
        input.lines().map(split_line)
    }
}

struct Verticals<'input>(&'input str);

impl<'input> IntoIterator for Verticals<'input> {
    type Item = Result<Dimensions>;

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
    b: Option<Dimensions>,
    c: Option<Dimensions>,
}

impl<'input> Iterator for VerticalChunks<'input> {
    type Item = Result<Dimensions>;

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

        let Dimensions(a1, b1, c1) = match split_line(first_line) {
            Ok(ok) => ok,
            Err(e) => return Some(Err(e)),
        };
        let Dimensions(a2, b2, c2) = match split_line(second_line) {
            Ok(ok) => ok,
            Err(e) => return Some(Err(e)),
        };
        let Dimensions(a3, b3, c3) = match split_line(third_line) {
            Ok(ok) => ok,
            Err(e) => return Some(Err(e)),
        };

        self.b = Some(Dimensions(b1, b2, b3));
        self.c = Some(Dimensions(c1, c2, c3));

        Some(Ok(Dimensions(a1, a2, a3)))
    }
}

fn split_line(line: &str) -> Result<Dimensions> {
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

    Ok(Dimensions(a, b, c))
}

trait DimensionsList {
    fn count_valid_triangles(self) -> Result<u16>;
}

impl<I> DimensionsList for I
where
    I: Iterator<Item = Result<Dimensions>>,
{
    fn count_valid_triangles(self) -> Result<u16> {
        self.map(|res| res.map(|dims| dims.is_valid_triangle().into_num()))
            .reduce(|a, b| Ok(a? + b?))
            .ok_or_eyre("empty input")?
    }
}
