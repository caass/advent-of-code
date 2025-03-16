use std::collections::hash_map;
use std::fmt::Display;
use std::ops::{AddAssign, Deref};
use std::str;

use eyre::{Result, bail};
use nohash_hasher::IntMap;
use rayon::prelude::*;

use aoc_meta::Problem;

pub const SIGNALS_AND_NOISE: Problem = Problem::solved(
    &|message| DecryptedMessage::decrypt(message, |iter| iter.max_by_key(|(_, freq)| *freq)),
    &|message| DecryptedMessage::decrypt(message, |iter| iter.min_by_key(|(_, freq)| *freq)),
);

type FrequencyMap = IntMap<usize, IntMap<u8, u8>>;
type Frequencies = hash_map::IntoIter<u8, u8>;

struct DecryptedMessage([u8; 8]);

impl DecryptedMessage {
    fn decrypt<F>(message: &str, decryptor: F) -> Result<Self>
    where
        F: Fn(Frequencies) -> Option<(u8, u8)>,
    {
        if !message.is_ascii() {
            bail!("Invalid input; need ascii characters");
        }

        let index_frequencies = message
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

        let mut inner = [0; 8];
        for (idx, freqs) in index_frequencies {
            let Some((ch, _)) = decryptor(freqs.into_iter()) else {
                bail!("Couldn't determine which car lives at index {idx}");
            };

            inner[idx] = ch;
        }

        Ok(Self(inner))
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
