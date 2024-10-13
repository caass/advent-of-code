use std::{
    fmt::{self, Debug, Display, Formatter},
    hash::{Hash, Hasher},
    num::ParseIntError,
    str::FromStr,
};

use enum_map::Enum;
use thiserror::Error;

#[derive(PartialEq, Eq, Clone, Copy, Enum)]
pub struct Part(bool);

impl Debug for Part {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_tuple("Part")
            .field(match *self {
                Part::ONE => &1u8,
                Part::TWO => &2u8,
            })
            .finish()
    }
}

impl Display for Part {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match *self {
            Part::ONE => Display::fmt(&1u8, f),
            Part::TWO => Display::fmt(&2u8, f),
        }
    }
}

impl Part {
    pub const ONE: Part = Part(true);
    pub const TWO: Part = Part(false);

    pub(crate) const fn as_u8(&self) -> u8 {
        match *self {
            Part::ONE => 1,
            Part::TWO => 2,
        }
    }
}

#[derive(Debug, Error)]
pub enum FromU8Error {
    #[error("Parts are 1-indexed")]
    Zero,

    #[error("Invalid part {0}: problems only have two parts.")]
    TooHigh(u8),
}

impl TryFrom<u8> for Part {
    type Error = FromU8Error;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Err(FromU8Error::Zero),
            1 => Ok(Part::ONE),
            2 => Ok(Part::TWO),
            3.. => Err(FromU8Error::TooHigh(value)),
        }
    }
}

#[derive(Debug, Error)]
pub enum ParsePartError {
    #[error(transparent)]
    NaN(#[from] ParseIntError),
    #[error(transparent)]
    OutOfRange(#[from] FromU8Error),
}

impl FromStr for Part {
    type Err = ParsePartError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let num: u8 = s.parse()?;
        let part = num.try_into()?;

        Ok(part)
    }
}

impl Hash for Part {
    fn hash<H: Hasher>(&self, state: &mut H) {
        state.write_u8(self.as_u8());
    }
}

impl nohash_hasher::IsEnabled for Part {}
