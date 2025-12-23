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
    _1 = 1,
    _2,
    _3,
    _4,
    _5,
    _6,
    _7,
    _8,
    _9,
    _10,
    _11,
    _12,
    _13,
    _14,
    _15,
    _16,
    _17,
    _18,
    _19,
    _20,
    _21,
    _22,
    _23,
    _24,
    _25,
}

impl AsRef<Path> for Day {
    fn as_ref(&self) -> &Path {
        Path::new(match self {
            Day::_1 => "01",
            Day::_2 => "02",
            Day::_3 => "03",
            Day::_4 => "04",
            Day::_5 => "05",
            Day::_6 => "06",
            Day::_7 => "07",
            Day::_8 => "08",
            Day::_9 => "09",
            Day::_10 => "10",
            Day::_11 => "11",
            Day::_12 => "12",
            Day::_13 => "13",
            Day::_14 => "14",
            Day::_15 => "15",
            Day::_16 => "16",
            Day::_17 => "17",
            Day::_18 => "18",
            Day::_19 => "19",
            Day::_20 => "20",
            Day::_21 => "21",
            Day::_22 => "22",
            Day::_23 => "23",
            Day::_24 => "24",
            Day::_25 => "25",
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
            1 => Ok(Day::_1),
            2 => Ok(Day::_2),
            3 => Ok(Day::_3),
            4 => Ok(Day::_4),
            5 => Ok(Day::_5),
            6 => Ok(Day::_6),
            7 => Ok(Day::_7),
            8 => Ok(Day::_8),
            9 => Ok(Day::_9),
            10 => Ok(Day::_10),
            11 => Ok(Day::_11),
            12 => Ok(Day::_12),
            13 => Ok(Day::_13),
            14 => Ok(Day::_14),
            15 => Ok(Day::_15),
            16 => Ok(Day::_16),
            17 => Ok(Day::_17),
            18 => Ok(Day::_18),
            19 => Ok(Day::_19),
            20 => Ok(Day::_20),
            21 => Ok(Day::_21),
            22 => Ok(Day::_22),
            23 => Ok(Day::_23),
            24 => Ok(Day::_24),
            25 => Ok(Day::_25),
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
