use std::{
    fmt::{self, Display, Formatter},
    mem::transmute,
    str::{self, FromStr},
};

use eyre::{bail, eyre, OptionExt, Report, Result};
use itertools::Itertools;
use rayon::prelude::*;

use crate::meta::Problem;

pub const SECURITY_THROUGH_OBSCURITY: Problem = Problem::partially_solved(&|input| todo!());

#[derive(Debug)]
struct Room {
    name: EncryptedName,
    sector: u16,
    checksum: Checksum,
}

impl FromStr for Room {
    type Err = Report;

    fn from_str(s: &str) -> Result<Self> {
        if !s.ends_with(']') {
            bail!("Missing checksum closing delimiter");
        };

        let s = &s[..(s.len() - 1)];
        let (s, checksum) = s
            .split_once('[')
            .ok_or_eyre("Missing checksum opening delimiter")?;

        let (name, sector) = s.rsplit_once('-').ok_or_eyre("Missing sector delimiter")?;

        Ok(Self {
            name: name.parse()?,
            sector: sector.parse()?,
            checksum: checksum.parse()?,
        })
    }
}

impl Display for Room {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}-{}[{}]", self.name, self.sector, self.checksum)
    }
}

#[derive(Debug)]
struct EncryptedName(Vec<NamePart>);

impl FromIterator<NamePart> for EncryptedName {
    fn from_iter<T: IntoIterator<Item = NamePart>>(iter: T) -> Self {
        Self(Vec::from_iter(iter))
    }
}

impl FromStr for EncryptedName {
    type Err = Report;

    fn from_str(s: &str) -> Result<Self> {
        s.split('-').map(NamePart::from_str).collect()
    }
}

impl Display for EncryptedName {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        Itertools::intersperse(self.0.iter().map(NamePart::as_ref), "-")
            .try_for_each(|data| f.write_str(data))
    }
}

#[derive(Debug)]
struct NamePart(Vec<Letter>);

impl FromIterator<Letter> for NamePart {
    fn from_iter<T: IntoIterator<Item = Letter>>(iter: T) -> Self {
        Self(Vec::from_iter(iter))
    }
}

impl FromStr for NamePart {
    type Err = Report;

    fn from_str(s: &str) -> Result<Self> {
        s.chars().map(Letter::try_from).collect()
    }
}

impl AsRef<str> for NamePart {
    fn as_ref(&self) -> &str {
        let bytes = unsafe { transmute::<&[Letter], &[u8]>(&self.0) };
        unsafe { str::from_utf8_unchecked(bytes) }
    }
}

impl Display for NamePart {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_ref())
    }
}

#[derive(Debug, Clone, Copy)]
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

    fn try_from(value: char) -> Result<Self, Self::Error> {
        if value.is_ascii_lowercase() {
            Ok(unsafe { transmute::<u8, Letter>(value as u8) })
        } else {
            Err(eyre!(
                "Invalid letter \"{}\", need ascii lowercase",
                value.escape_debug()
            ))
        }
    }
}

#[derive(Debug)]
struct Checksum([Letter; 5]);

impl FromStr for Checksum {
    type Err = Report;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.len() != 5 {
            bail!("Checksums are 5 letters long");
        };

        let mut letters = [Letter::A; 5];
        s.chars()
            .enumerate()
            .try_for_each(|(i, ch)| ch.try_into().map(|letter| letters[i] = letter))?;

        Ok(Self(letters))
    }
}

impl AsRef<str> for Checksum {
    fn as_ref(&self) -> &str {
        let bytes = unsafe { transmute::<&[Letter; 5], &[u8; 5]>(&self.0) };
        unsafe { str::from_utf8_unchecked(bytes) }
    }
}

impl Display for Checksum {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_ref())
    }
}
