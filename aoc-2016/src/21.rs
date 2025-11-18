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

use aoc_meta::Problem;
use winnow::stream::AsChar;
use winnow::token::{any, take_till};

pub const SCRAMBLED_LETTERS_AND_HASH: Problem = Problem::solved(
    &|input| {
        input
            .lines()
            .map(Operation::from_str)
            .try_fold(Password::new(b"abcdefgh"), |pw, op| op?.scramble(pw))
    },
    &|input| {
        input
            .lines()
            .rev()
            .map(Operation::from_str)
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

#[derive(Debug, Clone, Copy)]
enum Operation {
    SwapPosition { idx_a: u8, idx_b: u8 },
    SwapLetter { ch_a: u8, ch_b: u8 },
    RotateLeft { k: u8 },
    RotateRight { k: u8 },
    RotateLetter { ch: u8 },
    Reverse { from: u8, to: u8 },
    Move { from: u8, to: u8 },
}

impl Operation {
    fn scramble(self, mut password: Password) -> Result<Password> {
        match self {
            Operation::SwapPosition { idx_a: a, idx_b: b } => password.swap(a.into(), b.into()),
            Operation::SwapLetter { ch_a, ch_b } => {
                let [a, b] = password
                    .iter()
                    .enumerate()
                    .filter_map(|(idx, &ch)| {
                        if ch == ch_a || ch == ch_b {
                            Some(idx)
                        } else {
                            None
                        }
                    })
                    .collect_array()
                    .ok_or_else(|| {
                        eyre!(
                            "couldn't find characters '{}', '{}' in password \"{password}\"",
                            char::from_u32(ch_a.into()).unwrap_or(char::REPLACEMENT_CHARACTER),
                            char::from_u32(ch_b.into()).unwrap_or(char::REPLACEMENT_CHARACTER),
                        )
                    })?;
                password.swap(a, b);
            }
            Operation::RotateLeft { k } => password.rotate_left(k.into()),
            Operation::RotateRight { k } => password.rotate_right(k.into()),
            Operation::RotateLetter { ch } => {
                let k = password.iter().position(|&c| c == ch).ok_or_else(|| {
                    eyre!(
                        "couldn't find character '{}' in password \"{password}\"",
                        char::from_u32(ch.into()).unwrap_or(char::REPLACEMENT_CHARACTER),
                    )
                })?;
                password.rotate_right(1);
                password.rotate_right(k);
                password.rotate_right((k >= 4) as u8 as usize);
            }
            Operation::Reverse { from, to } => password[from.into()..=to.into()].reverse(),
            Operation::Move { from, to } => {
                let (from, to): (usize, usize) = (from.into(), to.into());
                let (src, dest) = match from.cmp(&to) {
                    Ordering::Less => (from + 1..to + 1, from),
                    Ordering::Equal => return Ok(password),
                    Ordering::Greater => (to..from, to + 1),
                };

                let ch = password[from];
                password.copy_within(src, dest);
                password[to] = ch;
            }
        };

        Ok(password)
    }

    fn unscramble(self, mut password: Password) -> Result<Password> {
        match self {
            Operation::SwapPosition { idx_a: a, idx_b: b } => password.swap(a.into(), b.into()),
            Operation::SwapLetter { ch_a, ch_b } => {
                let [a, b] = password
                    .iter()
                    .enumerate()
                    .filter_map(|(idx, &ch)| {
                        if ch == ch_a || ch == ch_b {
                            Some(idx)
                        } else {
                            None
                        }
                    })
                    .collect_array()
                    .ok_or_else(|| {
                        eyre!(
                            "couldn't find characters '{}', '{}' in password \"{password}\"",
                            char::from_u32(ch_a.into()).unwrap_or(char::REPLACEMENT_CHARACTER),
                            char::from_u32(ch_b.into()).unwrap_or(char::REPLACEMENT_CHARACTER),
                        )
                    })?;
                password.swap(a, b);
            }
            Operation::RotateLeft { k } => password.rotate_right(k.into()),
            Operation::RotateRight { k } => password.rotate_left(k.into()),
            Operation::RotateLetter { ch } => {
                let i = password
                    .iter()
                    .position(|&c| c == ch)
                    .ok_or_eyre("couldn't find character in password")?;

                let k = (0..8)
                    .find(|&j| (j + 1 + j + (j >= 4) as usize) % 8 == i)
                    .ok_or_eyre("no forward rotations matched")?;

                password.rotate_left(1);
                password.rotate_left(k);
                password.rotate_left((k >= 4) as usize);
            }
            Operation::Reverse { from, to } => password[from.into()..=to.into()].reverse(),
            Operation::Move { from, to } => {
                let (to, from): (usize, usize) = (from.into(), to.into());
                let (src, dest) = match from.cmp(&to) {
                    Ordering::Less => (from + 1..to + 1, from),
                    Ordering::Equal => return Ok(password),
                    Ordering::Greater => (to..from, to + 1),
                };

                let ch = password[from];
                password.copy_within(src, dest);
                password[to] = ch;
            }
        };

        Ok(password)
    }
}

impl FromStr for Operation {
    type Err = Report;

    fn from_str(s: &str) -> Result<Self> {
        dispatch! {terminated(take_till(1.., AsChar::is_space), " ");
            "swap" => dispatch!{terminated(take_till(1.., AsChar::is_space), " ");
                "position" => separated_pair(
                    dec_uint::<_, _, ContextError>,
                    " with position ",
                    dec_uint,
                ).map(|(idx_a, idx_b)| Operation::SwapPosition { idx_a, idx_b }),
                "letter" => separated_pair(
                    any,
                    " with letter ",
                    any,
                ).try_map(|(a, b): (char, char)| Ok::<_, <char as TryInto<u8>>::Error>(
                  Operation::SwapLetter { ch_a: a.try_into()?, ch_b: b.try_into()? },
                )),
                _ => fail,
          },
            "rotate" => dispatch!{terminated(take_till(1.., AsChar::is_space), " ");
                "left" => terminated(dec_uint, (" step", opt('s'))).map(|k| Operation::RotateLeft { k }),
                "right" => terminated(dec_uint, (" step", opt('s'))).map(|k| Operation::RotateRight { k }),
                "based" => preceded("on position of letter ", any).try_map(|ch: char| {
                  Ok::<_, <char as TryInto<u8>>::Error>(Operation::RotateLetter { ch: ch.try_into()? })
                }),
                _ => fail,
            },
            "reverse" => preceded("positions ", separated_pair(dec_uint, " through ", dec_uint))
                .map(|(from, to)| Operation::Reverse { from, to }),
            "move" => preceded("position ", separated_pair(dec_uint, " to position ", dec_uint))
                .map(|(from, to)| Operation::Move { from, to }),
            _ => fail,
        }
        .parse(s).map_err(|e| {
          let span = e.char_span();
          eyre!("error parsing input at {}..{}:\n{e}", span.start, span.end)})
    }
}
