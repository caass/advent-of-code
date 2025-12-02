use std::fmt::{self, Debug, Display, Formatter};
use std::hash::{Hash, Hasher};
use std::num::ParseIntError;
use std::path::Path;
use std::str::FromStr;

use enum_iterator::{Sequence, all};
use enum_map::Enum;
use thiserror::Error;

#[derive(Debug, Clone, Copy, Sequence, PartialEq, Eq, Enum)]
#[repr(u16)]
pub enum Year {
    Fifteen = 2015,
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

impl AsRef<Path> for Year {
    fn as_ref(&self) -> &Path {
        Path::new(match self {
            Year::Fifteen => "2015",
            Year::Sixteen => "2016",
            Year::Seventeen => "2017",
            Year::Eighteen => "2018",
            Year::Nineteen => "2019",
            Year::Twenty => "2020",
            Year::TwentyOne => "2021",
            Year::TwentyTwo => "2022",
            Year::TwentyThree => "2023",
            Year::TwentyFour => "2024",
            Year::TwentyFive => "2025",
        })
    }
}

impl Year {
    pub const FIRST: u16 = Year::Fifteen.as_u16();
    pub const LAST: u16 = Year::TwentyFive.as_u16();

    /// Returns a u16 represenation of `Self` guaranteed to be between [`FIRST_YEAR`] and [`LAST_YEAR`].
    #[inline]
    #[must_use]
    pub const fn as_u16(self) -> u16 {
        self as u16
    }

    pub const fn from_u16(year: u16) -> Result<Self, FromU16Error> {
        match year {
            ..=2014 => Err(FromU16Error::Early(year)),
            2015 => Ok(Year::Fifteen),
            2016 => Ok(Year::Sixteen),
            2017 => Ok(Year::Seventeen),
            2018 => Ok(Year::Eighteen),
            2019 => Ok(Year::Nineteen),
            2020 => Ok(Year::Twenty),
            2021 => Ok(Year::TwentyOne),
            2022 => Ok(Year::TwentyTwo),
            2023 => Ok(Year::TwentyThree),
            2024 => Ok(Year::TwentyFour),
            2025.. => Err(FromU16Error::Late(year)),
        }
    }

    /// Returns an iterator over all values of `Self`
    #[inline]
    pub fn iter() -> impl Iterator<Item = Self> {
        all::<Self>()
    }
}

impl From<Year> for u16 {
    #[inline]
    fn from(value: Year) -> Self {
        value.as_u16()
    }
}

#[derive(Debug, Error)]
pub enum FromU16Error {
    #[error("{0} is too early, Advent of Code started in 2015")]
    Early(u16),
    #[error("They haven't posted problems from {0} yet.")]
    Late(u16),
}

impl TryFrom<u16> for Year {
    type Error = FromU16Error;

    fn try_from(year: u16) -> Result<Self, Self::Error> {
        Self::from_u16(year)
    }
}

impl Display for Year {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        Display::fmt(&self.as_u16(), f)
    }
}

#[derive(Debug, Error)]
pub enum ParseYearError {
    #[error(transparent)]
    OutOfRange(#[from] FromU16Error),
    #[error(transparent)]
    NaN(#[from] ParseIntError),
}

impl FromStr for Year {
    type Err = ParseYearError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let num: u16 = s.parse()?;
        let year = num.try_into()?;
        Ok(year)
    }
}

impl Hash for Year {
    fn hash<H: Hasher>(&self, state: &mut H) {
        state.write_u16(self.as_u16());
    }
}

impl nohash_hasher::IsEnabled for Year {}
