use std::{
    fmt::{self, Display, Formatter, Write},
    ops::{Add, AddAssign, Sub, SubAssign},
    str::FromStr,
};

use eyre::{bail, eyre, Report, Result};

use crate::types::{problem, Problem};

pub const CORPORATE_POLICY: Problem = problem!(part1);
const ASCII_LETTER_OFFSET: u8 = b'a';

fn part1(input: &str) -> Result<Password> {
    let current_password = input.parse::<Password>()?;

    todo!()
}

#[derive(Debug)]
struct Password([Letter; 8]);

impl Password {
    /// Returns an iterator over the `Letter`s in this password.
    #[inline(always)]
    fn iter(&self) -> std::slice::Iter<Letter> {
        self.0.iter()
    }

    /// Returns the slice of `Letter`s that make up this password.
    #[inline(always)]
    fn letters(&self) -> &[Letter] {
        &self.0
    }

    fn next<V: FnMut(&Password) -> bool>(&self, validator: V) -> Password {
        todo!()
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
    #[inline(always)]
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
    #[inline(always)]
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
    #[inline(always)]
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
    #[inline(always)]
    const unsafe fn from_value_unchecked(value: u8) -> Self {
        let byte = value + ASCII_LETTER_OFFSET;
        // Safety: the caller guarantees the value is between 0-25
        unsafe { std::mem::transmute::<u8, Letter>(byte) }
    }

    /// Returns the [`char`] representation of this `Letter`
    #[inline(always)]
    const fn char(&self) -> char {
        *self as u8 as char
    }

    /// Returns the base 10 value of this `Letter`, where `Letter::A.value() == 0` and `Letter::Z.value() == 25`.
    #[inline(always)]
    const fn value(&self) -> u8 {
        *self as u8 - ASCII_LETTER_OFFSET
    }

    /// Increments this `Letter` by 1 wrapping at `Z`.
    ///
    /// Returns `true` if the operation wrapped.
    #[inline(always)]
    fn increment(&mut self) -> bool {
        *self += 1u8;

        *self == Letter::A
    }

    /// Decrements this `Letter` by 1, wrapping at `A`.
    ///
    /// Returns `true` if the operation wrapped.
    #[inline(always)]
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
    #[inline(always)]
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
    #[inline(always)]
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
            #[inline(always)]
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
            #[inline(always)]
            fn sub_assign(&mut self, rhs: $ty) {
                *self = *self - rhs;
            }
        }

        impl TryFrom<$ty> for Letter {
            type Error = ::eyre::Report;

            #[inline(always)]
            fn try_from(value: $ty) -> Result<Self, Self::Error> {
                let value = value.try_into()?;
                Self::from_value(value).map_err(Report::msg)
            }
        }

        impl PartialEq<$ty> for Letter {
            #[inline(always)]
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
    use super::*;

    mod letter_math {
        use super::*;
        use pretty_assertions::assert_eq;

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
}
