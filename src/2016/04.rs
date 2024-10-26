use std::collections::HashMap;
use std::fmt::{self, Display, Formatter, Write};
use std::hash::Hash;
use std::iter::repeat;
use std::mem::transmute;
use std::ops::{AddAssign, Deref};
use std::ptr;
use std::str::{self, FromStr};

use eyre::{bail, eyre, OptionExt, Report, Result};
use itertools::Itertools;
use nohash_hasher::BuildNoHashHasher;
use rayon::prelude::*;

use crate::meta::Problem;

pub const SECURITY_THROUGH_OBSCURITY: Problem = Problem::solved(
    &|input| {
        input
            .trim()
            .par_lines()
            .map(|line| {
                line.parse().map(|room: Room| {
                    room.is_real()
                        .then_some(room.sector.into())
                        .unwrap_or_default()
                })
            })
            .try_reduce(|| 0usize, |a, b| Ok(a + b))
    },
    &|input| {
        input
            .trim()
            .par_lines()
            .map(|line| {
                line.parse().map(|room: Room| {
                    let sector = room.sector;
                    if room.is_real() && room.decrypt() == ["northpole", "object", "storage"] {
                        Some(sector)
                    } else {
                        None
                    }
                })
            })
            .reduce(
                || Ok(None),
                |a, b| match (a, b) {
                    (Err(e), _) | (_, Err(e)) => Err(e),
                    (Ok(Some(_)), Ok(Some(_))) => bail!("found two rooms with northpole objects"),
                    (Ok(Some(sector)), _) | (_, Ok(Some(sector))) => Ok(Some(sector)),
                    (Ok(None), Ok(None)) => Ok(None),
                },
            )?
            .ok_or_eyre("no rooms with northpole objects")
    },
);

#[derive(Debug)]
struct Room {
    name: EncryptedName,
    sector: u16,
    checksum: Checksum,
}

impl Room {
    fn is_real(&self) -> bool {
        let mut frequencies: HashMap<Letter, u8, BuildNoHashHasher<Letter>> = HashMap::default();

        self.name
            .letters()
            .for_each(|letter| frequencies.entry(letter).or_default().add_assign(1));

        let expected_checksum = frequencies
            .into_iter()
            .map(|(letter, count)| LetterFrequency { letter, count })
            .sorted()
            .rev()
            .map(|LetterFrequency { letter, .. }| letter)
            .take(5)
            .collect::<Checksum>();

        expected_checksum == self.checksum
    }

    fn decrypt(self) -> DecryptedName {
        self.name.decrypt(self.sector)
    }
}

#[derive(Debug, PartialEq, Eq)]
struct LetterFrequency {
    letter: Letter,
    count: u8,
}

impl PartialOrd for LetterFrequency {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for LetterFrequency {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.count
            .cmp(&other.count)
            .then_with(|| other.letter.cmp(&self.letter))
    }
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

impl EncryptedName {
    fn letters(&self) -> impl Iterator<Item = Letter> + use<'_> {
        self.0.iter().flat_map(NamePart::letters)
    }

    fn decrypt(mut self, shift: u16) -> DecryptedName {
        self.0.iter_mut().for_each(|part| part.decrypt(shift));
        DecryptedName(self.0)
    }
}

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
        Itertools::intersperse(self.0.iter().map(NamePart::deref), "-")
            .try_for_each(|data| f.write_str(data))
    }
}

struct DecryptedName(Vec<NamePart>);

impl<'a, T: AsRef<[&'a str]>> PartialEq<T> for DecryptedName {
    fn eq(&self, other: &T) -> bool {
        let other = other.as_ref();
        self.0.len() == other.len()
            && self
                .0
                .iter()
                .map(NamePart::deref)
                .zip(other)
                .all(|(a, &b)| a == b)
    }
}

impl Display for DecryptedName {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        self.0
            .iter()
            .map(NamePart::deref)
            .interleave_shortest(repeat(" "))
            .try_for_each(|s| f.write_str(s))
    }
}

#[derive(Debug)]
struct NamePart(Vec<Letter>);

impl NamePart {
    fn letters(&self) -> impl Iterator<Item = Letter> + use<'_> {
        self.0.iter().copied()
    }

    fn decrypt(&mut self, shift: u16) {
        self.0
            .iter_mut()
            .for_each(|letter| *letter = letter.rotate(shift));
    }
}

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

impl Deref for NamePart {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        let bytes_ptr = ptr::from_ref::<[Letter]>(&self.0) as *const [u8];
        let bytes = unsafe { &*bytes_ptr };
        unsafe { str::from_utf8_unchecked(bytes) }
    }
}

impl Display for NamePart {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str(self)
    }
}

#[allow(dead_code, reason = "not actually dead")]
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
    /// Rotate a letter by a given amount
    fn rotate(self, by: u16) -> Self {
        let ascii_byte = self as u8;
        let zeroed_byte = ascii_byte - b'a';

        let shifted = u16::from(zeroed_byte) + by;
        let rotated_byte = (shifted % 26) as u8;

        let shifted_ascii_byte = rotated_byte + b'a';

        debug_assert!(shifted_ascii_byte.is_ascii());
        unsafe { std::mem::transmute::<u8, Letter>(shifted_ascii_byte) }
    }
}

impl Hash for Letter {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        state.write_u8(*self as u8);
    }
}

impl nohash_hasher::IsEnabled for Letter {}

impl Display for Letter {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_char(*self as u8 as char)
    }
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

#[derive(Debug, PartialEq, Eq)]
struct Checksum([Letter; 5]);

impl FromIterator<Letter> for Checksum {
    fn from_iter<T: IntoIterator<Item = Letter>>(iter: T) -> Self {
        let mut it = iter.into_iter();
        Self(std::array::from_fn(|_| {
            it.next().expect("iterator to contain 5 letters")
        }))
    }
}

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

impl Deref for Checksum {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        let bytes_ptr = ptr::from_ref::<[Letter]>(&self.0) as *const [u8];
        let bytes = unsafe { &*bytes_ptr };
        unsafe { str::from_utf8_unchecked(bytes) }
    }
}

impl Display for Checksum {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str(self)
    }
}
