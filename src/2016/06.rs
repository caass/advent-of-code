use std::fmt::Display;
use std::ops::{AddAssign, Deref};
use std::str;

use eyre::{bail, Result};
use nohash_hasher::IntMap;
use rayon::prelude::*;

use crate::meta::Problem;

pub const SIGNALS_AND_NOISE: Problem = Problem::solved(
    &DecryptedMessage::from_most_common,
    &DecryptedMessage::from_least_common,
);

type FrequencyMap = IntMap<usize, IntMap<u8, u8>>;

struct DecryptedMessage([u8; 8]);

impl DecryptedMessage {
    fn from_most_common(message: &str) -> Result<Self> {
        DecryptedMessage::decrypt(message, |iter| {
            *iter.max_by_key(|(_, freq)| *freq).unwrap().0
        })
    }

    fn from_least_common(message: &str) -> Result<Self> {
        DecryptedMessage::decrypt(message, |iter| {
            *iter.min_by_key(|(_, freq)| *freq).unwrap().0
        })
    }

    fn decrypt<F>(message: &str, decryptor: F) -> Result<Self>
    where
        F: Fn(<&IntMap<u8, u8> as IntoIterator>::IntoIter) -> u8,
    {
        if !message.is_ascii() {
            bail!("Invalid input; need ascii characters");
        }

        let frequencies = message
            .trim()
            .par_lines()
            .flat_map_iter(|line| line.as_bytes().iter().copied().enumerate())
            .fold(FrequencyMap::default, |mut map, (index, ch)| {
                map.entry(index)
                    .or_default()
                    .entry(ch)
                    .or_default()
                    .add_assign(1);
                map
            })
            .reduce(FrequencyMap::default, |a, mut b| {
                for (index, map) in a {
                    for (ch, frequency) in map {
                        b.entry(index)
                            .or_default()
                            .entry(ch)
                            .or_default()
                            .add_assign(frequency);
                    }
                }

                b
            });

        Ok(Self::via(&frequencies, decryptor))
    }

    fn via<F>(map: &FrequencyMap, f: F) -> Self
    where
        F: Fn(<&IntMap<u8, u8> as IntoIterator>::IntoIter) -> u8,
    {
        let inner = std::array::from_fn(|i| {
            let freqs = map.get(&i).unwrap_or_else(|| {
                panic!("expected map to have frequencies for {} positions", i + 1)
            });

            f(freqs.iter())
        });

        Self(inner)
    }
}

impl Deref for DecryptedMessage {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        debug_assert!(self.0.is_ascii());
        unsafe { str::from_utf8_unchecked(&self.0) }
    }
}

impl Display for DecryptedMessage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Display::fmt(&**self, f)
    }
}
