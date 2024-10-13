use std::fmt::{self, Debug, Display, Formatter};
use std::hash::{Hash, Hasher};
use std::num::ParseIntError;
use std::str::FromStr;

use enum_iterator::Sequence;
use enum_map::Enum;
use thiserror::Error;

#[derive(Debug, Clone, Copy, Sequence, PartialEq, Eq, Enum)]
#[repr(u8)]
pub enum Day {
    One = 1,
    Two,
    Three,
    Four,
    Five,
    Six,
    Seven,
    Eight,
    Nine,
    Ten,
    Eleven,
    Twelve,
    Thirteen,
    Fourteen,
    Fifteen,
    Sixteen,
    Seventeen,
    Eighteen,
    Nineteen,
    Twenty,
    TwentyOne,
    TwentyTwo,
    TwentyThree,
    TwentyFour,
    TwentyFive,
}

impl Day {
    #[inline(always)]
    pub const fn as_u8(self) -> u8 {
        self as u8
    }

    #[inline(always)]
    pub(crate) const fn from_u8(day: u8) -> Result<Self, FromU8Error> {
        match day {
            0 => Err(FromU8Error::Zero),
            1 => Ok(Day::One),
            2 => Ok(Day::Two),
            3 => Ok(Day::Three),
            4 => Ok(Day::Four),
            5 => Ok(Day::Five),
            6 => Ok(Day::Six),
            7 => Ok(Day::Seven),
            8 => Ok(Day::Eight),
            9 => Ok(Day::Nine),
            10 => Ok(Day::Ten),
            11 => Ok(Day::Eleven),
            12 => Ok(Day::Twelve),
            13 => Ok(Day::Thirteen),
            14 => Ok(Day::Fourteen),
            15 => Ok(Day::Fifteen),
            16 => Ok(Day::Sixteen),
            17 => Ok(Day::Seventeen),
            18 => Ok(Day::Eighteen),
            19 => Ok(Day::Nineteen),
            20 => Ok(Day::Twenty),
            21 => Ok(Day::TwentyOne),
            22 => Ok(Day::TwentyTwo),
            23 => Ok(Day::TwentyThree),
            24 => Ok(Day::TwentyFour),
            25 => Ok(Day::TwentyFive),
            26.. => Err(FromU8Error::TooBig),
        }
    }
}

impl From<Day> for u8 {
    #[inline(always)]
    fn from(value: Day) -> Self {
        value.as_u8()
    }
}

#[derive(Debug, Error)]
pub enum FromU8Error {
    #[error("Days are 1-indexed")]
    Zero,

    #[error("They stop posting problems after the 25th")]
    TooBig,
}

impl TryFrom<u8> for Day {
    type Error = FromU8Error;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        Day::from_u8(value)
    }
}

impl Display for Day {
    #[inline(always)]
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        Display::fmt(&self.as_u8(), f)
    }
}

#[derive(Debug, Error)]
pub enum ParseDayErr {
    #[error(transparent)]
    NaN(#[from] ParseIntError),
    #[error(transparent)]
    OutOfRange(#[from] FromU8Error),
}

impl FromStr for Day {
    type Err = ParseDayErr;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let num: u8 = s.parse()?;
        let day = num.try_into()?;
        Ok(day)
    }
}

impl Hash for Day {
    fn hash<H: Hasher>(&self, state: &mut H) {
        state.write_u8(self.as_u8())
    }
}

impl nohash_hasher::IsEnabled for Day {}
