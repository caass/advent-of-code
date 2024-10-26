use std::fmt::{self, Debug, Display, Formatter, Write};
use std::str::FromStr;

use eyre::{bail, eyre, Report, Result};
use rayon::prelude::*;
use winnow::{
    combinator::{alt, preceded, repeat},
    error::StrContext,
    prelude::*,
    token::{any, take},
};

use crate::meta::Problem;

/// https://adventofcode.com/2015/day/8
pub const MATCHSTICKS: Problem = Problem::solved(
    &|input| {
        input
            .par_lines()
            .map(|s| {
                let line = s.trim().parse::<Line>()?;
                Ok::<_, Report>(line.code_len() - line.data_len())
            })
            .try_reduce(|| 0, |a, b| Ok(a + b))
    },
    &|input| {
        input
            .par_lines()
            .map(|s| {
                let line = s.trim().parse::<Line>()?;

                let code_len = line.code_len();
                let re_encoded_len = line.re_encoded_len();

                Ok::<_, Report>(re_encoded_len - code_len)
            })
            .try_reduce(|| 0, |a, b| Ok(a + b))
            .map(|n| n.to_string())
    },
);

struct Line {
    chars: Vec<Char>,
}

impl Line {
    fn code_len(&self) -> usize {
        self.chars.par_iter().map(Char::code_len).sum::<usize>() + 2 // + 2 for the opening and closing `"`s
    }

    #[inline]
    fn data_len(&self) -> usize {
        self.chars.len()
    }

    fn re_encoded_len(&self) -> usize {
        self.chars
            .par_iter()
            .map(Char::re_encoded_len)
            .sum::<usize>()
            + 6 // + 2 for the opening and closing `"\"`s
    }
}

impl FromStr for Line {
    type Err = Report;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if !(s.starts_with('"') && s.ends_with('"')) {
            bail!("Missing opening and/or closing quotes");
        };

        let unquoted = &s[1..(s.len() - 1)];
        repeat(1.., char)
            .context(StrContext::Label("line"))
            .map(|chars| Line { chars })
            .parse(unquoted)
            .map_err(|e| {
                eyre!(
                    "Error parsing {}: {} at index {} ({})",
                    e.input(),
                    e.inner(),
                    e.offset(),
                    if e.offset() == s.len() {
                        "<EOF>"
                    } else {
                        &s[e.offset()..]
                    }
                )
            })
    }
}

fn char(input: &mut &str) -> PResult<Char> {
    alt((hex, quote, backslash, literal))
        .context(StrContext::Label("char"))
        .parse_next(input)
}

fn literal(input: &mut &str) -> PResult<Char> {
    any.context(StrContext::Label("character literal"))
        .map(|c: char| {
            debug_assert!(c.is_ascii());
            let byte = c as u32 as u8;
            Char::Literal(byte)
        })
        .parse_next(input)
}

fn backslash(input: &mut &str) -> PResult<Char> {
    "\\\\"
        .context(StrContext::Label("escaped backslash"))
        .map(|_| Char::Backslash)
        .parse_next(input)
}

fn quote(input: &mut &str) -> PResult<Char> {
    preceded('\\', '"')
        .context(StrContext::Label("escaped quotation mark"))
        .map(|_| Char::Quote)
        .parse_next(input)
}

fn hex(input: &mut &str) -> PResult<Char> {
    preceded(
        "\\x",
        take(2u8)
            .try_map(|hex| u8::from_str_radix(hex, 16))
            .context(StrContext::Label("hex literal")),
    )
    .context(StrContext::Label("escaped hex character"))
    .map(Char::Hex)
    .parse_next(input)
}

#[derive(PartialEq, Eq)]
enum Char {
    /// Any character literal that's not an escape or an opening/closing quote
    Literal(u8),
    /// An escaped backslash (`\\`) which prints as `\`
    Backslash,
    /// An escaped quotation mark (`\"`) which prints as `"`
    Quote,
    /// An escaped hex code (`\xNN`) which represents an ASCII character
    Hex(u8),
}

impl Debug for Char {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match *self {
            Char::Literal(c) => f.debug_tuple("Literal").field(&(c as char)).finish(),
            Char::Backslash => f.write_str("Backslash"),
            Char::Quote => f.write_str("Quote"),
            Char::Hex(c) => f.debug_tuple("Hex").field(&(c as char)).finish(),
        }
    }
}

impl Display for Char {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match *self {
            Char::Literal(c) => f.write_char(c as char),
            Char::Backslash => f.write_char('\\'),
            Char::Quote => f.write_char('"'),
            Char::Hex(c) => f.write_char(c as char),
        }
    }
}

impl Char {
    #[inline]
    const fn code_len(&self) -> usize {
        match self {
            Char::Literal(_) => 1,
            Char::Backslash => 2,
            Char::Quote => 2,
            Char::Hex(_) => 4,
        }
    }

    #[inline]
    const fn re_encoded_len(&self) -> usize {
        match self {
            Char::Literal(_) => 1,
            Char::Backslash => 4,
            Char::Quote => 4,
            Char::Hex(_) => 5,
        }
    }
}
