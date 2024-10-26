use std::fmt::{self, Display, Formatter, Write};
use std::iter::FusedIterator;
use std::ops::{Add, AddAssign, Sub, SubAssign};
use std::str::FromStr;

use eyre::{bail, eyre, Report, Result};
use rayon::iter::plumbing::{bridge, Producer};
use rayon::prelude::*;

use crate::meta::Problem;

/// <https://adventofcode.com/2015/day/11>
pub const CORPORATE_POLICY: Problem =
    Problem::solved(&|input| input.parse().map(Password::next), &|input| {
        input.parse().map(Password::next).map(Password::next)
    });

const ASCII_LETTER_OFFSET: u8 = b'a';

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
struct Password([Letter; 8]);

impl IntoIterator for Password {
    type Item = Password;

    type IntoIter = PasswordIter;

    fn into_iter(self) -> Self::IntoIter {
        PasswordIter::new(self)
    }
}

impl IntoParallelIterator for Password {
    type Iter = PasswordIter;

    type Item = Password;

    fn into_par_iter(self) -> Self::Iter {
        PasswordIter::new(self)
    }
}

impl ParallelIterator for PasswordIter {
    type Item = Password;

    #[inline]
    fn drive_unindexed<C>(self, consumer: C) -> C::Result
    where
        C: rayon::iter::plumbing::UnindexedConsumer<Self::Item>,
    {
        bridge(self, consumer)
    }
}

impl IndexedParallelIterator for PasswordIter {
    #[inline]
    fn len(&self) -> usize {
        ExactSizeIterator::len(self)
    }

    #[inline]
    fn drive<C: rayon::iter::plumbing::Consumer<Self::Item>>(self, consumer: C) -> C::Result {
        bridge(self, consumer)
    }

    #[inline]
    fn with_producer<CB: rayon::iter::plumbing::ProducerCallback<Self::Item>>(
        self,
        callback: CB,
    ) -> CB::Output {
        callback.callback(self)
    }
}

#[derive(Debug)]
struct PasswordIter {
    /// The password to start iterating from, inclusive
    from: Password,

    /// The password to stop iterating at, exclusive
    to: Password,

    /// Whether or not we're done iterating
    done: bool,
}

impl PasswordIter {
    /// Create a `PasswordIter` that iterates over all possible passwords, starting with `start`.
    fn new(start: Password) -> Self {
        PasswordIter {
            from: start,
            to: start,
            done: false,
        }
    }

    /// Create a `PasswordIter` that iterates over all possible passwords between `from` (inclusive)
    /// and `to` (exclusive).
    #[cfg(test)]
    fn new_range(from: Password, to: Password) -> Self {
        PasswordIter {
            from,
            to,
            done: false,
        }
    }
}

impl Iterator for PasswordIter {
    type Item = Password;

    fn next(&mut self) -> Option<Self::Item> {
        if self.done {
            return None;
        }

        let next = self.from;
        self.from.increment();

        if self.from == self.to {
            self.done = true;
        }

        Some(next)
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        (
            ExactSizeIterator::len(self),
            Some(ExactSizeIterator::len(self)),
        )
    }
}

impl ExactSizeIterator for PasswordIter {
    #[inline]
    fn len(&self) -> usize {
        match self.from.distance(self.to) {
            // We're done iterating, we've hit every password
            0 if self.done => 0,

            // We just started iterating, we need to hit every password
            0 => Password::ZZZZZZZZ.value(),

            // We're in the middle of iterating
            n => n,
        }
    }
}

impl DoubleEndedIterator for PasswordIter {
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.done {
            return None;
        }

        self.to.decrement();
        let next = self.to;

        if self.from == self.to {
            self.done = true;
        }

        Some(next)
    }
}

impl FusedIterator for PasswordIter {}

impl Producer for PasswordIter {
    type Item = Password;

    type IntoIter = PasswordIter;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self
    }

    fn split_at(self, index: usize) -> (Self, Self) {
        let mid = self.from + index;

        (
            PasswordIter {
                from: self.from,
                to: mid,
                done: index == 0,
            },
            PasswordIter {
                from: mid,
                to: self.to,
                done: index == ExactSizeIterator::len(&self),
            },
        )
    }
}

impl Password {
    #[cfg(test)]
    const AAAAAAAA: Password = Password([Letter::A; 8]);
    const ZZZZZZZZ: Password = Password([Letter::Z; 8]);

    /// Returns an iterator over the `Letter`s in this password.
    #[inline]
    fn iter(&self) -> std::slice::Iter<Letter> {
        self.0.iter()
    }

    /// Returns an iterator over mutable references to the `Letter`s in this password.
    #[inline]
    fn iter_mut(&mut self) -> std::slice::IterMut<Letter> {
        self.0.iter_mut()
    }

    /// Returns the slice of `Letter`s that make up this password.
    #[inline]
    fn letters(&self) -> &[Letter] {
        &self.0
    }

    /// Increment this `Password`, wrapping at `ZZZZZZZZ`.
    fn increment(&mut self) {
        for letter in self.iter_mut().rev() {
            let wrapped = letter.increment();

            if !wrapped {
                return;
            }
        }
    }

    /// Decrement this `Password`, wrapping at `AAAAAAAA`.
    fn decrement(&mut self) {
        for letter in self.iter_mut().rev() {
            let wrapped = letter.decrement();

            if !wrapped {
                return;
            }
        }
    }

    /// Returns the next password that passes validation, if one exists.
    fn next(mut self) -> Password {
        self.increment();
        self.into_par_iter()
            .by_exponential_blocks()
            .find_first(|pwd| pwd.is_valid())
            .expect("to find a valid password")
    }

    fn is_valid(self) -> bool {
        let has_increasing_straight = self.letters().windows(3).any(|window| {
            window[0] <= Letter::X && window[0] + 1 == window[1] && window[1] + 1 == window[2]
        });
        let has_forbidden_letter = self
            .iter()
            .any(|&letter| letter == Letter::I || letter == Letter::O || letter == Letter::L);
        let has_nonoverlapping_pairs = (0..5).any(|i| {
            self.letters()[i] == self.letters()[i + 1]
                && ((i + 2)..7).any(|j| self.letters()[j] == self.letters()[j + 1])
        });

        has_increasing_straight && !has_forbidden_letter && has_nonoverlapping_pairs
    }

    /// Returns the numerical `value` of this password, where `AAAAAAAA.value() == 0`.
    const fn value(self) -> usize {
        let mut exp = 7;
        let mut i = 0;
        let mut out = 0;

        loop {
            let val = self.0[i].value() as usize;
            out += val * 26usize.pow(exp);

            if exp == 0 {
                break out;
            }

            i += 1;
            exp -= 1;
        }
    }

    /// Returns the numerical `distance` between this password and `other`, representing the number of passwords
    /// needed to increment from `self` to `other`.
    fn distance(self, other: Password) -> usize {
        const BASE: usize = Password::ZZZZZZZZ.value();

        let lhs = other.value();
        let rhs = self.value();

        (lhs + BASE - rhs) % BASE
    }

    /// Construct a password from its `value`
    ///
    /// # Safety
    ///
    /// The given value must not be greater than `Password::ZZZZZZZZ.value()`.
    unsafe fn from_value_unchecked(mut value: usize) -> Password {
        Password(std::array::from_fn(|i| {
            // Safety: `i` is between 0-7, so `7 - i` is between `7-0`, i.e. it fits in a `u32`.
            let exp = unsafe { (7 - i).try_into().unwrap_unchecked() };

            let this_place_value = {
                let this_place_value_usize = value / 26usize.pow(exp);
                let this_place_value_maybe = this_place_value_usize.try_into();

                debug_assert!(this_place_value_maybe.is_ok());
                // Safety: we mod-assign value by 26 ^ exp every iteration, so dividing by 26 ^ exp
                // will always yield a number between 0-25 as long as the caller upholds their contract.
                unsafe { this_place_value_maybe.unwrap_unchecked() }
            };
            value %= 26usize.pow(exp);

            // Safety: same as above
            unsafe { Letter::from_value_unchecked(this_place_value) }
        }))
    }
}

impl Add<usize> for Password {
    type Output = Password;

    #[inline]
    fn add(self, rhs: usize) -> Self::Output {
        let lhs = self.value();
        let value = (lhs + rhs) % const { Password::ZZZZZZZZ.value() + 1 };

        // Safety: value is between 0 and the maximum allowed by virtue of the % operator.
        unsafe { Password::from_value_unchecked(value) }
    }
}

impl AddAssign<usize> for Password {
    #[inline]
    fn add_assign(&mut self, rhs: usize) {
        *self = *self + rhs;
    }
}

impl Display for Password {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        self.iter().try_for_each(|letter| Display::fmt(letter, f))
    }
}

impl FromStr for Password {
    type Err = Report;

    fn from_str(s: &str) -> Result<Self> {
        if !s.is_ascii() {
            bail!("passwords must be ascii");
        }

        if s.len() != 8 {
            bail!("passwords must be 8 ascii characters");
        }

        let mut letters = [Letter::A; 8];

        for (i, ch) in s.chars().enumerate() {
            letters[i] = ch.try_into()?;
        }

        Ok(Password(letters))
    }
}

#[allow(dead_code, reason = "incorrectly detecting variants as unused")]
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

impl Letter {
    /// Create a new `Letter` from its `char` representation.
    ///
    /// # Errors
    ///
    /// Returns an error if the given `char` isn't an ASCII lowercase letter.
    #[inline]
    const fn from_char(letter: char) -> Result<Self, &'static str> {
        if letter.is_ascii_lowercase() {
            // Safety: letter is ascii lowercase
            Ok(unsafe { Letter::from_char_unchecked(letter) })
        } else {
            Err("letters must be ascii lowercase")
        }
    }

    /// Create a new `Letter` from its `char` representation.
    ///
    /// # Safety
    ///
    /// The caller must guarantee the given `char` is an ASCII lowercase letter.
    #[inline]
    const unsafe fn from_char_unchecked(letter: char) -> Self {
        let byte = letter as u8;
        // Safety: the caller guarantees the letter is ascii lowercase
        unsafe { std::mem::transmute::<u8, Letter>(byte) }
    }

    /// Create a new `Letter` from its `value` representation.
    ///
    /// # Errors
    ///
    /// Returns an error if the given value isn't between 0-25.
    #[inline]
    const fn from_value(value: u8) -> Result<Self, &'static str> {
        if value < 26 {
            // Safety: the letter is between 0-25
            Ok(unsafe { Letter::from_value_unchecked(value) })
        } else {
            Err("letter values are between 0-25")
        }
    }

    /// Create a new `Letter` from its `value` representation
    ///
    /// # Safety
    ///
    /// The caller must guarantee the value is between 0-25.
    #[inline]
    const unsafe fn from_value_unchecked(value: u8) -> Self {
        let byte = value + ASCII_LETTER_OFFSET;
        // Safety: the caller guarantees the value is between 0-25
        unsafe { std::mem::transmute::<u8, Letter>(byte) }
    }

    /// Returns the [`char`] representation of this `Letter`
    #[inline]
    const fn char(self) -> char {
        self as u8 as char
    }

    /// Returns the base 10 value of this `Letter`, where `Letter::A.value() == 0` and `Letter::Z.value() == 25`.
    #[inline]
    const fn value(self) -> u8 {
        self as u8 - ASCII_LETTER_OFFSET
    }

    /// Increments this `Letter` by 1 wrapping at `Z`.
    ///
    /// Returns `true` if the operation wrapped.
    #[inline]
    fn increment(&mut self) -> bool {
        *self += 1u8;

        *self == Letter::A
    }

    /// Decrements this `Letter` by 1, wrapping at `A`.
    ///
    /// Returns `true` if the operation wrapped.
    #[inline]
    fn decrement(&mut self) -> bool {
        *self -= 1u8;

        *self == Letter::Z
    }
}

impl TryFrom<char> for Letter {
    type Error = Report;

    #[inline]
    fn try_from(value: char) -> std::result::Result<Self, Self::Error> {
        Letter::from_char(value).map_err(Report::msg)
    }
}

impl PartialEq<char> for Letter {
    #[inline]
    fn eq(&self, other: &char) -> bool {
        self.char().eq(other)
    }
}

impl FromStr for Letter {
    type Err = Report;

    fn from_str(s: &str) -> Result<Self> {
        if s.len() > 1 {
            bail!("Can't parse a letter from a multibyte string");
        }

        let ch = s
            .chars()
            .next()
            .ok_or_else(|| eyre!("Can't parse a letter from an empty string"))?;
        ch.try_into()
    }
}

impl Display for Letter {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_char(self.char())
    }
}

macro_rules! impl_letter_math {
    ($($ty:ty),+) => {$(
        impl Add<$ty> for Letter {
            type Output = Letter;

            fn add(self, rhs: $ty) -> Letter {
                // By adding 26 to the left-hand side and taking the mod 26 of the right hand side,
                // we wrap around 26 and ensure the value is always positive.

                // Safety: `0-25` fits in every integer type
                let lhs: $ty = unsafe { self.value().try_into().unwrap_unchecked() };
                let rhs = rhs % 26;
                let sum = (lhs + 26 + rhs) % 26;

                // Safety: `sum` is guaranteed to be between 0-25 because it's mod 26.
                unsafe {
                    let value = sum.try_into().unwrap_unchecked();
                    Letter::from_value_unchecked(value)
                }
            }
        }

        impl AddAssign<$ty> for Letter {
            #[inline]
            fn add_assign(&mut self, rhs: $ty) {
                *self = *self + rhs
            }
        }

        impl Sub<$ty> for Letter {
            type Output = Letter;

            fn sub(self, rhs: $ty) -> Letter {
                // By adding 26 to the left-hand side and taking the mod 26 of the right hand side,
                // we wrap around 26 and ensure the value is always positive.

                // Safety: `0-25` fits in every integer type
                let lhs: $ty = unsafe { self.value().try_into().unwrap_unchecked() };
                let rhs = rhs % 26;
                let difference = (lhs + 26 - rhs) % 26;

                // Safety: `sum` is guaranteed to be between 0-25 because it's mod 26.
                unsafe {
                    let value = difference.try_into().unwrap_unchecked();
                    Letter::from_value_unchecked(value)
                }
            }
        }

        impl SubAssign<$ty> for Letter {
            #[inline]
            fn sub_assign(&mut self, rhs: $ty) {
                *self = *self - rhs;
            }
        }

        impl TryFrom<$ty> for Letter {
            type Error = ::eyre::Report;

            #[inline]
            fn try_from(value: $ty) -> Result<Self, Self::Error> {
                let value = value.try_into()?;
                Self::from_value(value).map_err(Report::msg)
            }
        }

        impl PartialEq<$ty> for Letter {
            #[inline]
            fn eq(&self, other: &$ty) -> bool {
                // Safety: 0-25 fits in every integer representation
                let this: $ty = unsafe { self.value().try_into().unwrap_unchecked() };
                this.eq(other)
            }
        }
    )+};
}

impl_letter_math!(u8, u16, u32, u64, usize, u128, i8, i16, i32, i64, isize, i128);

#[cfg(test)]
mod test {
    use pretty_assertions::assert_eq;

    use super::*;

    mod letter_math {
        use pretty_assertions::assert_eq;

        use super::*;

        #[test]
        fn add() {
            assert_eq!(Letter::A + 0, Letter::A);
            assert_eq!(Letter::B + 5, Letter::G);
        }

        #[test]
        fn add_wrapping() {
            assert_eq!(Letter::Z + 1, Letter::A);
            assert_eq!(Letter::A + 26, Letter::A);
        }

        #[test]
        fn add_negative() {
            assert_eq!(Letter::A + -1, Letter::Z);
            assert_eq!(Letter::Z + -20, Letter::F);
        }

        #[test]
        fn sub() {
            assert_eq!(Letter::J - 1, Letter::I);
            assert_eq!(Letter::M - 0, Letter::M);
        }

        #[test]
        fn sub_wrapping() {
            assert_eq!(Letter::C - 3, Letter::Z);
            assert_eq!(Letter::G - 26, Letter::G);
        }
    }

    #[allow(clippy::similar_names, reason = "they're similar passwords")]
    #[test]
    fn password_distance() {
        let aaaaaaaa: Password = "aaaaaaaa".parse().unwrap();
        let aaaaaaaa2: Password = "aaaaaaaa".parse().unwrap();

        assert_eq!(aaaaaaaa.distance(aaaaaaaa2), 0);
        assert_eq!(aaaaaaaa2.distance(aaaaaaaa), 0);

        let aaaaaaab: Password = "aaaaaaab".parse().unwrap();
        assert_eq!(aaaaaaaa.distance(aaaaaaab), 1);
        assert_eq!(aaaaaaab.distance(aaaaaaaa), Password::ZZZZZZZZ.value() - 1);

        let aaaaaaaz: Password = "aaaaaaaz".parse().unwrap();
        assert_eq!(aaaaaaaa.distance(aaaaaaaz), 25);
        assert_eq!(aaaaaaaz.distance(aaaaaaaa), Password::ZZZZZZZZ.value() - 25);

        let aaaaaaba: Password = "aaaaaaba".parse().unwrap();
        assert_eq!(aaaaaaaa.distance(aaaaaaba), 26);
        assert_eq!(aaaaaaba.distance(aaaaaaaa), Password::ZZZZZZZZ.value() - 26);
    }

    #[test]
    fn password_iter() {
        let mut iter = PasswordIter {
            from: "aaaaaaaa".parse().unwrap(),
            to: "aaaaaaba".parse().unwrap(),
            done: false,
        };

        assert_eq!(ExactSizeIterator::len(&iter), 26);

        for i in 0..26 {
            let mut expected = [Letter::A; 8];
            expected[7] += i;

            let actual = iter
                .next()
                .unwrap_or_else(|| panic!("No {i}th item in iterator"));
            assert_eq!(actual.letters(), expected);
            assert_eq!(
                ExactSizeIterator::len(&iter),
                25 - i,
                "wrong len at i = {i}"
            );
        }

        let next = iter.next();
        assert_eq!(None, next);
    }

    #[test]
    fn password_iter_rev() {
        let mut iter = Iterator::rev(PasswordIter {
            from: "aaaaaaaa".parse().unwrap(),
            to: "aaaaaaba".parse().unwrap(),
            done: false,
        });

        assert_eq!(iter.len(), 26);

        for i in 0..26 {
            let mut expected = [Letter::A; 8];
            expected[7] += 25 - i;

            let actual = iter
                .next()
                .unwrap_or_else(|| panic!("No {i}th item in iterator"));
            assert_eq!(actual.letters(), expected);
            assert_eq!(iter.len(), 25 - i, "wrong len at i = {i}");
        }

        let next = iter.next();
        assert_eq!(None, next);
    }

    #[test]
    fn password_math() {
        assert_eq!(Password::AAAAAAAA + 1, "aaaaaaab".parse().unwrap());
        assert_eq!(Password::AAAAAAAA + 2 * 26, "aaaaaaca".parse().unwrap());
        assert_eq!(
            Password::AAAAAAAA + 25 * 26usize.pow(3),
            "aaaazaaa".parse().unwrap()
        );
    }

    #[test]
    fn password_math_wrap() {
        assert_eq!(Password::ZZZZZZZZ + 1, Password::AAAAAAAA);
        assert_eq!(Password::ZZZZZZZZ + 2 * 26, "aaaaaabz".parse().unwrap());
    }

    #[test]
    fn password_producer() {
        let producer = PasswordIter::new_range(Password::AAAAAAAA, Password::ZZZZZZZZ);
        let (left, right) = producer.split_at(Password::ZZZZZZZZ.value() / 2);
        let mid = "mzzzzzzz".parse().unwrap();

        assert_eq!(left.from, Password::AAAAAAAA);
        assert_eq!(left.to, mid);

        assert_eq!(right.from, mid);
        assert_eq!(right.to, Password::ZZZZZZZZ);
    }

    #[test]
    fn password_producer_wrap() {
        let to = "aaaaaaaz".parse().unwrap();
        let producer = PasswordIter::new_range(Password::ZZZZZZZZ, to);

        let (left, right) = producer.split_at(13);
        let mid = "aaaaaaam".parse().unwrap();

        assert_eq!(left.from, Password::ZZZZZZZZ);
        assert_eq!(left.to, mid);

        assert_eq!(right.from, mid);
        assert_eq!(right.to, to);
    }
}
