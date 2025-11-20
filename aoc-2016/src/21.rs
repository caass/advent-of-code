use std::cmp::Ordering;
use std::fmt::Display;
use std::ops;
use std::str::FromStr;

use eyre::{OptionExt, Report, Result, eyre};
use itertools::Itertools;
use winnow::ascii::dec_uint;
use winnow::combinator::{dispatch, fail, opt, preceded, separated_pair, terminated};
use winnow::error::ContextError;
use winnow::prelude::*;
use winnow::stream::AsChar;
use winnow::token::{any, take_till};

use aoc_meta::Problem;

pub const SCRAMBLED_LETTERS_AND_HASH: Problem = Problem::solved(
    &|input| {
        input
            .lines()
            .map(Op::from_str)
            .try_fold(Password::new(b"abcdefgh"), |pw, op| op?.scramble(pw))
    },
    &|input| {
        input
            .lines()
            .rev()
            .map(Op::from_str)
            .try_fold(Password::new(b"fbgdceah"), |pw, op| op?.unscramble(pw))
    },
);

#[derive(Debug, Clone, Copy)]
struct Password([u8; 8]);

impl Password {
    const fn new(password: &[u8; 8]) -> Self {
        Self(*password)
    }
}

impl Display for Password {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match str::from_utf8(&self.0) {
            Ok(s) => f.write_str(s),
            Err(e) => Display::fmt(&e, f),
        }
    }
}

impl ops::Deref for Password {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl ops::DerefMut for Password {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

trait Operation {
    fn scramble(self, password: Password) -> Result<Password>;

    fn unscramble(self, password: Password) -> Result<Password>;
}

#[derive(Debug, Clone, Copy)]
struct SwapPosition {
    a: u8,
    b: u8,
}

impl Operation for SwapPosition {
    fn scramble(self, mut password: Password) -> Result<Password> {
        password.swap(self.a.into(), self.b.into());
        Ok(password)
    }

    fn unscramble(self, password: Password) -> Result<Password> {
        self.scramble(password)
    }
}

#[derive(Debug, Clone, Copy)]
struct SwapLetter {
    ch_a: u8,
    ch_b: u8,
}

impl Operation for SwapLetter {
    fn scramble(self, mut password: Password) -> Result<Password> {
        let [a, b] = password
            .iter()
            .enumerate()
            .filter_map(|(idx, &ch)| {
                if ch == self.ch_a || ch == self.ch_b {
                    Some(idx)
                } else {
                    None
                }
            })
            .collect_array()
            .ok_or_else(|| {
                eyre!(
                    "couldn't find characters '{}', '{}' in password \"{password}\"",
                    char::from_u32(self.ch_a.into()).unwrap_or(char::REPLACEMENT_CHARACTER),
                    char::from_u32(self.ch_b.into()).unwrap_or(char::REPLACEMENT_CHARACTER),
                )
            })?;

        password.swap(a, b);
        Ok(password)
    }

    fn unscramble(self, password: Password) -> Result<Password> {
        self.scramble(password)
    }
}

#[derive(Debug, Clone, Copy)]
struct RotateLeft {
    k: u8,
}

impl Operation for RotateLeft {
    fn scramble(self, mut password: Password) -> Result<Password> {
        password.rotate_left(self.k.into());
        Ok(password)
    }

    fn unscramble(self, mut password: Password) -> Result<Password> {
        password.rotate_right(self.k.into());
        Ok(password)
    }
}

#[derive(Debug, Clone, Copy)]
struct RotateRight {
    k: u8,
}

impl Operation for RotateRight {
    fn scramble(self, mut password: Password) -> Result<Password> {
        password.rotate_right(self.k.into());
        Ok(password)
    }

    fn unscramble(self, mut password: Password) -> Result<Password> {
        password.rotate_left(self.k.into());
        Ok(password)
    }
}

#[derive(Debug, Clone, Copy)]
struct RotateLetter {
    ch: u8,
}

impl Operation for RotateLetter {
    fn scramble(self, mut password: Password) -> Result<Password> {
        let k = password.iter().position(|&c| c == self.ch).ok_or_else(|| {
            eyre!(
                "couldn't find character '{}' in password \"{password}\"",
                char::from_u32(self.ch.into()).unwrap_or(char::REPLACEMENT_CHARACTER),
            )
        })?;
        password.rotate_right(1);
        password.rotate_right(k);
        password.rotate_right((k >= 4) as u8 as usize);

        Ok(password)
    }

    fn unscramble(self, mut password: Password) -> Result<Password> {
        let i = password
            .iter()
            .position(|&c| c == self.ch)
            .ok_or_eyre("couldn't find character in password")?;

        let k = (0..8)
            .find(|&j| (j + 1 + j + (j >= 4) as usize) % 8 == i)
            .ok_or_eyre("no forward rotations matched")?;

        password.rotate_left(1);
        password.rotate_left(k);
        password.rotate_left((k >= 4) as usize);

        Ok(password)
    }
}

#[derive(Debug, Clone, Copy)]
struct Reverse {
    from: u8,
    to: u8,
}

impl Operation for Reverse {
    fn scramble(self, mut password: Password) -> Result<Password> {
        password[self.from.into()..=self.to.into()].reverse();
        Ok(password)
    }

    fn unscramble(self, password: Password) -> Result<Password> {
        self.scramble(password)
    }
}

#[derive(Debug, Clone, Copy)]
struct Move {
    from: u8,
    to: u8,
}

impl Operation for Move {
    fn scramble(self, mut password: Password) -> Result<Password> {
        let (from, to): (usize, usize) = (self.from.into(), self.to.into());
        let (src, dest) = match from.cmp(&to) {
            Ordering::Less => (from + 1..to + 1, from),
            Ordering::Equal => return Ok(password),
            Ordering::Greater => (to..from, to + 1),
        };

        let ch = password[from];
        password.copy_within(src, dest);
        password[to] = ch;

        Ok(password)
    }

    fn unscramble(self, mut password: Password) -> Result<Password> {
        let (to, from): (usize, usize) = (self.from.into(), self.to.into());
        let (src, dest) = match from.cmp(&to) {
            Ordering::Less => (from + 1..to + 1, from),
            Ordering::Equal => return Ok(password),
            Ordering::Greater => (to..from, to + 1),
        };

        let ch = password[from];
        password.copy_within(src, dest);
        password[to] = ch;

        Ok(password)
    }
}

#[derive(Debug, Clone, Copy)]
enum Op {
    SwapPosition(SwapPosition),
    SwapLetter(SwapLetter),
    RotateLeft(RotateLeft),
    RotateRight(RotateRight),
    RotateLetter(RotateLetter),
    Reverse(Reverse),
    Move(Move),
}

impl Operation for Op {
    fn scramble(self, password: Password) -> Result<Password> {
        match self {
            Op::SwapPosition(op) => op.scramble(password),
            Op::SwapLetter(op) => op.scramble(password),
            Op::RotateLeft(op) => op.scramble(password),
            Op::RotateRight(op) => op.scramble(password),
            Op::RotateLetter(op) => op.scramble(password),
            Op::Reverse(op) => op.scramble(password),
            Op::Move(op) => op.scramble(password),
        }
    }

    fn unscramble(self, password: Password) -> Result<Password> {
        match self {
            Op::SwapPosition(op) => op.unscramble(password),
            Op::SwapLetter(op) => op.unscramble(password),
            Op::RotateLeft(op) => op.unscramble(password),
            Op::RotateRight(op) => op.unscramble(password),
            Op::RotateLetter(op) => op.unscramble(password),
            Op::Reverse(op) => op.unscramble(password),
            Op::Move(op) => op.unscramble(password),
        }
    }
}

impl FromStr for Op {
    type Err = Report;

    fn from_str(s: &str) -> Result<Self> {
        dispatch! {terminated(take_till(1.., AsChar::is_space), " ");
            "swap" => dispatch!{terminated(take_till(1.., AsChar::is_space), " ");
                "position" => separated_pair(
                    dec_uint::<_, _, ContextError>,
                    " with position ",
                    dec_uint,
                ).map(|(a, b)| Op::SwapPosition(SwapPosition { a, b })),
                "letter" => separated_pair(
                    any,
                    " with letter ",
                    any,
                ).try_map(|(a, b): (char, char)| Ok::<_, <char as TryInto<u8>>::Error>(
                  Op::SwapLetter(SwapLetter { ch_a: a.try_into()?, ch_b: b.try_into()? }),
                )),
                _ => fail,
          },
            "rotate" => dispatch!{terminated(take_till(1.., AsChar::is_space), " ");
                "left" => terminated(dec_uint, (" step", opt('s'))).map(|k| Op::RotateLeft(RotateLeft { k })),
                "right" => terminated(dec_uint, (" step", opt('s'))).map(|k| Op::RotateRight(RotateRight { k })),
                "based" => preceded("on position of letter ", any).try_map(|c: char| {
                    c.try_into().map(|ch| Op::RotateLetter(RotateLetter { ch }))
                }),
                _ => fail,
            },
            "reverse" => preceded("positions ", separated_pair(dec_uint, " through ", dec_uint))
                .map(|(from, to)| Op::Reverse(Reverse { from, to })),
            "move" => preceded("position ", separated_pair(dec_uint, " to position ", dec_uint))
                .map(|(from, to)| Op::Move(Move { from, to })),
            _ => fail,
        }
        .parse(s).map_err(|e| {
          let span = e.char_span();
          eyre!("error parsing input at {}..{}:\n{e}", span.start, span.end)})
    }
}
