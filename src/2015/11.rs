use std::{
    fmt::{self, Display, Formatter, Write},
    ops::{Index, IndexMut},
    str::FromStr,
};

use eyre::{bail, Report, Result};
use rayon::{
    iter::plumbing::{self, bridge, Producer, Reducer},
    prelude::*,
};
use winnow::Parser;

use crate::types::{problem, Problem};

pub const CORPORATE_POLICY: Problem = problem!(part1);

fn part1(input: &str) -> Result<Password> {
    let password = input.parse::<Password>()?;
    todo!()
}

struct PasswordIter {
    from: Password,
    to: Password,
}

impl PasswordIter {
    /// Get the number of passwords that will be produced by this iterator
    fn len(&self) -> usize {
        self.from.distance(&self.to)
    }
}

impl ParallelIterator for PasswordIter {
    type Item = Password;

    fn drive_unindexed<C>(self, consumer: C) -> C::Result
    where
        C: plumbing::UnindexedConsumer<Self::Item>,
    {
        self.drive(consumer)
    }
}

impl IndexedParallelIterator for PasswordIter {
    fn len(&self) -> usize {
        self.len()
    }

    fn drive<C: plumbing::Consumer<Self::Item>>(self, consumer: C) -> C::Result {
        let len = self.len();

        if len <= 64 {
            bridge(self, consumer)
        } else {
            let (left_iter, right_iter) = self.split_at(len / 2);
            let (left_consumer, right_consumer, reducer) = consumer.split_at(len / 2);

            let left = bridge(left_iter, left_consumer);
            let right = bridge(right_iter, right_consumer);

            reducer.reduce(left, right)
        }
    }

    fn with_producer<CB: plumbing::ProducerCallback<Self::Item>>(self, callback: CB) -> CB::Output {
        callback.callback(self)
    }
}

impl Iterator for PasswordIter {
    type Item = Password;

    fn next(&mut self) -> Option<Self::Item> {
        if self.from == self.to {
            None
        } else {
            let out = self.from;
            self.from.increment();
            Some(out)
        }
    }
}

impl DoubleEndedIterator for PasswordIter {
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.from == self.to {
            None
        } else {
            let out = self.to;
            self.to.decrement();
            Some(out)
        }
    }
}

impl ExactSizeIterator for PasswordIter {
    fn len(&self) -> usize {
        self.from.distance(&self.to)
    }
}

impl Producer for PasswordIter {
    type Item = Password;

    type IntoIter = PasswordIter;

    fn into_iter(self) -> Self::IntoIter {
        self
    }

    fn split_at(self, index: usize) -> (Self, Self) {
        let halfway_base10 = self.from.as_base10() + index;
    }
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct Password([Letter; 8]);

impl IntoParallelIterator for Password {
    type Iter = PasswordIter;

    type Item = Password;

    fn into_par_iter(self) -> Self::Iter {
        let from = self;
        let mut to = self;
        to.decrement();

        PasswordIter { from, to }
    }
}

impl Password {
    const AAAAAAAA: Password = Password([Letter::A; 8]);
    const ZZZZZZZZ: Password = Password([Letter::Z; 8]);

    #[inline]
    fn iter(&self) -> std::slice::Iter<'_, Letter> {
        self.0.iter()
    }

    #[inline]
    fn iter_mut(&mut self) -> std::slice::IterMut<'_, Letter> {
        self.0.iter_mut()
    }

    fn increment(&mut self) {
        for letter in self.iter_mut().rev() {
            if let Some(next_letter) = letter.next() {
                *letter = next_letter;
                return;
            } else {
                // Overflow from `Z` to `A` and keep going
                *letter = Letter::A;
            }
        }
    }

    fn decrement(&mut self) {
        for letter in self.iter_mut().rev() {
            if let Some(prev_letter) = letter.next() {
                *letter = prev_letter;
                return;
            } else {
                // Underflow from `A` to `Z` and keep going
                *letter = Letter::Z;
            }
        }
    }

    fn as_base10(&self) -> usize {
        self.iter()
            .rev()
            .enumerate()
            .map(|(i, letter)| {
                let n = letter.as_base10() as usize;
                let exp = i as u32;
                let base = 26usize;
                n * base.pow(exp)
            })
            .sum()
    }

    fn from_base10(num: usize) -> Self {
        let mut buf = itoa::Buffer::new();
        let bytes = buf.format(num).as_bytes();

        let inner = std::array::from_fn(|i| {
            const ASCII_NUMBER_OFFSET: u8 = b'0';
            let exp = (7 - i) as u32;

            let digit = (bytes[i] - ASCII_NUMBER_OFFSET) as usize;
            digit * 26usize.pow(exp)
        });

        Self(inner)
        s.as_bytes().iter().rev().map(|ascii| ascii - ASCII_NUMBER_OFFSET).enumerate();
    }

    /// Returns the "distance" between two passwords, i.e. how many increments are required to go from
    /// `self` to `to`.
    fn distance(&self, to: &Password) -> usize {
        const MAX: usize = const {
            let mut max = 0;
            let mut exp = 0;

            loop {
                if exp == 8 {
                    break;
                }

                max += 25 * 26usize.pow(exp);
                exp += 1;
            }

            max
        };

        let this = self.as_base10();
        let other = to.as_base10();

        if this < other {
            other - this
        } else {
            MAX - this + other
        }
    }
}

impl Display for Password {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        self.0.iter().try_for_each(|letter| Display::fmt(letter, f))
    }
}

impl FromStr for Password {
    type Err = Report;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        if !(s.len() == 8 && s.is_ascii()) {
            bail!("password must be 8 ascii characters");
        }

        let mut letters = [Letter::A; 8];
        s.char_indices().try_for_each(|(i, char)| {
            char.try_into().map(|letter| {
                letters[i] = letter;
            })
        })?;

        Ok(Password(letters))
    }
}

impl Index<usize> for Password {
    type Output = Letter;

    #[inline]
    fn index(&self, index: usize) -> &Self::Output {
        self.0.index(index)
    }
}

impl IndexMut<usize> for Password {
    #[inline]
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        self.0.index_mut(index)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u8)]
enum Letter {
    A = b'a',
    B,
    C,
    D,
    E,
    F,
    G,
    H,
    I,
    J,
    K,
    L,
    M,
    N,
    O,
    P,
    Q,
    R,
    S,
    T,
    U,
    V,
    W,
    X,
    Y,
    Z,
}

impl TryFrom<char> for Letter {
    type Error = Report;

    fn try_from(value: char) -> std::result::Result<Self, Self::Error> {
        match value {
            'a' => Ok(Letter::A),
            'b' => Ok(Letter::B),
            'c' => Ok(Letter::C),
            'd' => Ok(Letter::D),
            'e' => Ok(Letter::E),
            'f' => Ok(Letter::F),
            'g' => Ok(Letter::G),
            'h' => Ok(Letter::H),
            'i' => Ok(Letter::I),
            'j' => Ok(Letter::J),
            'k' => Ok(Letter::K),
            'l' => Ok(Letter::L),
            'm' => Ok(Letter::M),
            'n' => Ok(Letter::N),
            'o' => Ok(Letter::O),
            'p' => Ok(Letter::P),
            'q' => Ok(Letter::Q),
            'r' => Ok(Letter::R),
            's' => Ok(Letter::S),
            't' => Ok(Letter::T),
            'u' => Ok(Letter::U),
            'v' => Ok(Letter::V),
            'w' => Ok(Letter::W),
            'x' => Ok(Letter::X),
            'y' => Ok(Letter::Y),
            'z' => Ok(Letter::Z),
            other => bail!("invalid letter {other}, must be ascii lowercase"),
        }
    }
}

impl Letter {
    /// Returns the [`char`] representation of this `Letter`
    #[inline]
    fn as_char(&self) -> char {
        *self as u8 as char
    }

    /// Returns the base 10 representation of this `Letter`
    #[inline]
    fn as_base10(&self) -> u8 {
        const ASCII_LETTER_OFFSET: u8 = b'a';
        *self as u8 - ASCII_LETTER_OFFSET
    }

    /// Returns the next `Letter` after this one.
    ///
    /// Returns `None` if there's no next letter, i.e. when overflowing from `Z` to `A`.
    #[inline]
    fn next(&self) -> Option<Letter> {
        match *self {
            Letter::A => Some(Letter::B),
            Letter::B => Some(Letter::C),
            Letter::C => Some(Letter::D),
            Letter::D => Some(Letter::E),
            Letter::E => Some(Letter::F),
            Letter::F => Some(Letter::G),
            Letter::G => Some(Letter::H),
            Letter::H => Some(Letter::I),
            Letter::I => Some(Letter::J),
            Letter::J => Some(Letter::K),
            Letter::K => Some(Letter::L),
            Letter::L => Some(Letter::M),
            Letter::M => Some(Letter::N),
            Letter::N => Some(Letter::O),
            Letter::O => Some(Letter::P),
            Letter::P => Some(Letter::Q),
            Letter::Q => Some(Letter::R),
            Letter::R => Some(Letter::S),
            Letter::S => Some(Letter::T),
            Letter::T => Some(Letter::U),
            Letter::U => Some(Letter::V),
            Letter::V => Some(Letter::W),
            Letter::W => Some(Letter::X),
            Letter::X => Some(Letter::Y),
            Letter::Y => Some(Letter::Z),
            Letter::Z => None,
        }
    }

    #[inline]
    fn wrapping_next(&self) -> Letter {
        self.next().unwrap_or(Letter::A)
    }

    /// Returns the previous `Letter` before this one.
    ///
    /// Returns `None` if there's no previous letter, i.e. when underflowing from `A` to `Z`.
    fn previous(&self) -> Option<Letter> {
        match *self {
            Letter::A => None,
            Letter::B => Some(Letter::A),
            Letter::C => Some(Letter::B),
            Letter::D => Some(Letter::C),
            Letter::E => Some(Letter::D),
            Letter::F => Some(Letter::E),
            Letter::G => Some(Letter::F),
            Letter::H => Some(Letter::G),
            Letter::I => Some(Letter::H),
            Letter::J => Some(Letter::I),
            Letter::K => Some(Letter::J),
            Letter::L => Some(Letter::K),
            Letter::M => Some(Letter::L),
            Letter::N => Some(Letter::M),
            Letter::O => Some(Letter::N),
            Letter::P => Some(Letter::O),
            Letter::Q => Some(Letter::P),
            Letter::R => Some(Letter::Q),
            Letter::S => Some(Letter::R),
            Letter::T => Some(Letter::S),
            Letter::U => Some(Letter::T),
            Letter::V => Some(Letter::U),
            Letter::W => Some(Letter::V),
            Letter::X => Some(Letter::W),
            Letter::Y => Some(Letter::X),
            Letter::Z => Some(Letter::Y),
        }
    }
}

impl Display for Letter {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_char(self.as_char())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn letter_base10() {
        assert_eq!(Letter::A.as_base10(), 0);
        assert_eq!(Letter::Z.as_base10(), 25);
    }

    #[test]
    fn password_base10() {
        assert_eq!(Password::AAAAAAAA.as_base10(), 0);
        assert_eq!(
            Password::ZZZZZZZZ.as_base10(),
            (0..8).map(|exp| 25 * 26usize.pow(exp)).sum()
        )
    }
}
