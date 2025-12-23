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
    _2015 = 2015,
    _2016,
    _2017,
    _2018,
    _2019,
    _2020,
    _2021,
    _2022,
    _2023,
    _2024,
    _2025,
}

impl AsRef<Path> for Year {
    fn as_ref(&self) -> &Path {
        Path::new(match self {
            Year::_2015 => "2015",
            Year::_2016 => "2016",
            Year::_2017 => "2017",
            Year::_2018 => "2018",
            Year::_2019 => "2019",
            Year::_2020 => "2020",
            Year::_2021 => "2021",
            Year::_2022 => "2022",
            Year::_2023 => "2023",
            Year::_2024 => "2024",
            Year::_2025 => "2025",
        })
    }
}

impl Year {
    pub const FIRST: u16 = Year::_2015.as_u16();
    pub const LAST: u16 = Year::_2025.as_u16();

    /// Returns a u16 represenation of `Self` guaranteed to be between [`FIRST_YEAR`] and [`LAST_YEAR`].
    #[inline]
    #[must_use]
    pub const fn as_u16(self) -> u16 {
        self as u16
    }

    pub const fn from_u16(year: u16) -> Result<Self, FromU16Error> {
        match year {
            ..=2014 => Err(FromU16Error::Early(year)),
            2015 => Ok(Year::_2015),
            2016 => Ok(Year::_2016),
            2017 => Ok(Year::_2017),
            2018 => Ok(Year::_2018),
            2019 => Ok(Year::_2019),
            2020 => Ok(Year::_2020),
            2021 => Ok(Year::_2021),
            2022 => Ok(Year::_2022),
            2023 => Ok(Year::_2023),
            2024 => Ok(Year::_2024),
            2025 => Ok(Year::_2025),
            2026.. => Err(FromU16Error::Late(year)),
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
