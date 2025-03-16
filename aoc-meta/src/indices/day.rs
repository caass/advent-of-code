use std::fmt::{self, Debug, Display, Formatter};
use std::hash::{Hash, Hasher};
use std::num::ParseIntError;
use std::path::Path;
use std::str::FromStr;

use enum_iterator::{Sequence, all};
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

impl AsRef<Path> for Day {
    fn as_ref(&self) -> &Path {
        Path::new(match self {
            Day::One => "01",
            Day::Two => "02",
            Day::Three => "03",
            Day::Four => "04",
            Day::Five => "05",
            Day::Six => "06",
            Day::Seven => "07",
            Day::Eight => "08",
            Day::Nine => "09",
            Day::Ten => "10",
            Day::Eleven => "11",
            Day::Twelve => "12",
            Day::Thirteen => "13",
            Day::Fourteen => "14",
            Day::Fifteen => "15",
            Day::Sixteen => "16",
            Day::Seventeen => "17",
            Day::Eighteen => "18",
            Day::Nineteen => "19",
            Day::Twenty => "20",
            Day::TwentyOne => "21",
            Day::TwentyTwo => "22",
            Day::TwentyThree => "23",
            Day::TwentyFour => "24",
            Day::TwentyFive => "25",
        })
    }
}

impl Day {
    #[inline]
    #[must_use]
    pub const fn as_u8(self) -> u8 {
        self as u8
    }

    #[inline]
    pub const fn from_u8(day: u8) -> Result<Self, FromU8Error> {
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

    /// Returns an iterator over all values of `Self`
    #[inline]
    pub fn iter() -> impl Iterator<Item = Self> {
        all::<Self>()
    }
}

impl From<Day> for u8 {
    #[inline]
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
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{:0>2}", self.as_u8())
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
        state.write_u8(self.as_u8());
    }
}

impl nohash_hasher::IsEnabled for Day {}
