#[cfg(test)]
use itertools::Itertools;
use memchr::memchr;

use eyre::{eyre, Report, Result};
use rayon::prelude::*;

#[cfg(test)]
use crate::common::TryParse;
use crate::common::{BoolExt, TryFromStr};
use crate::meta::Problem;

pub const INTERNET_PROTOCOL_VERSION_7: Problem = Problem::solved(
    &|input| count(input, |address| address.supports_tls()),
    &|input| count(input, |address| address.supports_ssl()),
);

#[inline]
fn count<F: Sync + Send + Fn(Address) -> bool>(input: &str, f: F) -> Result<usize> {
    input
        .par_lines()
        .map(Address::try_from_str)
        .try_fold(usize::default, |a, result| {
            result.map(|address| f(address).into_num::<usize>() + a)
        })
        .try_reduce(usize::default, |a, b| Ok(a + b))
}

struct Address<'a>(&'a [u8]);

impl<'a> TryFromStr<'a> for Address<'a> {
    type Err = Report;

    fn try_from_str(s: &'a str) -> Result<Self, Self::Err> {
        s.par_chars()
            .all(|ch| ch.is_ascii_lowercase() || ['[', ']'].contains(&ch))
            .then_some(Self(s.as_bytes()))
            .ok_or_else(|| eyre!("Invalid IPv7 address: \"{s}\""))
    }
}

impl<'a> Address<'a> {
    #[inline]
    fn segments(&self) -> Segments<'a> {
        Segments::of_address(self)
    }

    #[inline]
    fn supports_tls(&self) -> bool {
        let mut has_supernet_abba = false;

        for segment in self.segments() {
            if segment.has_abba() {
                if segment.is_hypernet() {
                    return false;
                };

                has_supernet_abba = true;
            }
        }

        has_supernet_abba
    }

    #[inline]
    fn supports_ssl(&self) -> bool {
        let supernets = self.segments().filter(Segment::is_supernet);

        for (a, b) in supernets.flat_map(Segment::abas) {
            if self
                .segments()
                .filter(Segment::is_hypernet)
                .any(|segment| segment.has_bab(a, b))
            {
                return true;
            }
        }

        false
    }
}

impl<'a> Segments<'a> {
    #[inline]
    fn of_address(address: &Address<'a>) -> Self {
        Self {
            source: address.0,
            // We always start _outside_ of an hypernet sequence
            hypernet: false,
        }
    }
}

struct Segments<'a> {
    source: &'a [u8],
    hypernet: bool,
}

impl<'a> Iterator for Segments<'a> {
    type Item = Segment<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.source.is_empty() {
            return None;
        };

        let delimiter = if self.hypernet { b']' } else { b'[' };
        let idx = memchr(delimiter, self.source).unwrap_or(self.source.len());

        // Safety: this index is guaranteed to be valid, since it was found via `memchr` / is `str.len`
        let (segment_body, rest) = unsafe { self.source.split_at_unchecked(idx) };

        let segment = Segment::new(self.hypernet, segment_body);

        self.hypernet = !self.hypernet;
        self.source = rest.get(1..).unwrap_or_default();

        Some(segment)
    }
}

#[derive(Debug, PartialEq, Eq)]
enum Segment<'a> {
    Hypernet { body: &'a [u8] },
    Supernet { body: &'a [u8] },
}

impl<'a> Segment<'a> {
    fn new(hypernet: bool, body: &'a [u8]) -> Self {
        if hypernet {
            Self::Hypernet { body }
        } else {
            Self::Supernet { body }
        }
    }

    #[inline]
    fn body(&self) -> &'a [u8] {
        match self {
            Segment::Hypernet { body } | Segment::Supernet { body } => body,
        }
    }

    #[inline]
    fn abas(self) -> impl Iterator<Item = (u8, u8)> + 'a {
        self.body().windows(3).filter_map(|slice| {
            if slice[0] == slice[2] && slice[0] != slice[1] {
                Some((slice[0], slice[1]))
            } else {
                None
            }
        })
    }
}

impl Segment<'_> {
    #[inline]
    fn has_abba(&self) -> bool {
        self.body()
            .windows(4)
            .any(|slice| slice[0] == slice[3] && slice[1] == slice[2] && slice[0] != slice[1])
    }

    #[inline]
    fn has_bab(&self, a: u8, b: u8) -> bool {
        self.body()
            .windows(3)
            .any(|slice| slice[0] == slice[2] && slice[0] == b && slice[1] == a)
    }

    #[inline]
    fn is_hypernet(&self) -> bool {
        matches!(self, Segment::Hypernet { .. })
    }

    #[inline]
    fn is_supernet(&self) -> bool {
        matches!(self, Segment::Supernet { .. })
    }
}

#[test]
fn example() {
    let abba = "abba[mnop]qrst".try_parse::<Address>().unwrap();
    let segments: (_, _, _) = abba.segments().collect_tuple().unwrap();
    assert_eq!(
        segments,
        (
            Segment::Supernet { body: b"abba" },
            Segment::Hypernet { body: b"mnop" },
            Segment::Supernet { body: b"qrst" }
        )
    );
    assert!(abba.supports_tls());

    assert!(!"abcd[bddb]xyyx"
        .try_parse::<Address>()
        .unwrap()
        .supports_tls());

    assert!(!"aaaa[qwer]tyui"
        .try_parse::<Address>()
        .unwrap()
        .supports_tls());

    assert!("ioxxoj[asdfgh]zxcvbn"
        .try_parse::<Address>()
        .unwrap()
        .supports_tls());
}
