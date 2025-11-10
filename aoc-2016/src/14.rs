use std::iter::FusedIterator;
use std::mem;

use bitvec::{index::BitIdx, prelude::*};
use eyre::{OptionExt, Result};
use itertools::Itertools;
use md5::digest::Output;
use md5::{Digest, Md5};
use rayon::prelude::*;

use aoc_meta::Problem;

pub const ONE_TIME_PAD: Problem =
    Problem::solved(&|input| sixty_fourth_key(input, HashInfo::new), &|input| {
        sixty_fourth_key(input, HashInfo::stretched)
    });

const HEX: [u8; 16] = [
    b'0', b'1', b'2', b'3', b'4', b'5', b'6', b'7', b'8', b'9', b'a', b'b', b'c', b'd', b'e', b'f',
];

fn sixty_fourth_key<F>(salt: &str, f: F) -> Result<usize>
where
    F: Fn(&Md5, usize) -> HashInfo + Send + Sync + 'static,
{
    let seed = Md5::new_with_prefix(salt);
    keys(seed, f).nth(63).ok_or_eyre("couldn't find 64 keys")
}

fn keys<F>(seed: Md5, f: F) -> Keys<F>
where
    F: Fn(&Md5, usize) -> HashInfo + Send + Sync + 'static,
{
    Keys::new(seed, f)
}

struct Keys<F> {
    hashes: Hashes<F>,
    i: usize,
    n: usize,
    a: Vec<HashInfo>,
    b: Vec<HashInfo>,
}

impl<F: Fn(&Md5, usize) -> HashInfo + Send + Sync + 'static> Keys<F> {
    fn new(seed: Md5, f: F) -> Self {
        let mut hashes = Hashes::new(seed, f);
        let a = hashes.next().expect("to find at least 1,000 hashes");
        let b = hashes.next().expect("to find at least 2,000 hashes");

        Self {
            hashes,
            i: 0,
            n: 0,
            a,
            b,
        }
    }
}

impl<F: Fn(&Md5, usize) -> HashInfo + Send + Sync + 'static> Iterator for Keys<F> {
    type Item = usize;

    fn next(&mut self) -> Option<Self::Item> {
        for i in self.i..1000 {
            let Some(hex) = self.a[i].triplet else {
                continue;
            };

            let has_quints = self.a[i + 1..]
                .par_iter()
                .chain(&self.b[..i + 1])
                .any(|h2| h2.quintet_mask.get_bit::<Msb0>(hex));

            if has_quints {
                self.i = i + 1;
                return Some(self.n * 1000 + i);
            }
        }

        self.i = 0;
        self.n += 1;
        self.a = mem::take(&mut self.b);
        self.b = self.hashes.next()?;

        self.next()
    }
}

impl<F: Fn(&Md5, usize) -> HashInfo + Send + Sync + 'static> FusedIterator for Keys<F> {}

struct Hashes<F> {
    n: Option<usize>,
    f: F,
    seed: Md5,
}

impl<F: Fn(&Md5, usize) -> HashInfo + Send + Sync + 'static> Hashes<F> {
    fn new(seed: Md5, f: F) -> Self {
        Self {
            n: Some(0),
            f,
            seed,
        }
    }
}

impl<F: Fn(&Md5, usize) -> HashInfo + Send + Sync + 'static> Iterator for Hashes<F> {
    type Item = Vec<HashInfo>;

    fn next(&mut self) -> Option<Self::Item> {
        let n = self.n?;
        self.n = n.checked_add(1);

        let start = n * 1000;
        let end = start + 1000;

        let mut hashes = Vec::default();
        (start..end)
            .into_par_iter()
            .map(|i| (self.f)(&self.seed, i))
            .collect_into_vec(&mut hashes);

        Some(hashes)
    }
}

impl<F: Fn(&Md5, usize) -> HashInfo + Send + Sync + 'static> FusedIterator for Hashes<F> {}

struct HashInfo {
    triplet: Option<BitIdx<u16>>,
    quintet_mask: u16,
}

impl HashInfo {
    fn new(seed: &Md5, index: usize) -> Self {
        let mut buf = itoa::Buffer::new();
        let hash = seed.clone().chain_update(buf.format(index)).finalize();

        Self::with_hash(hash)
    }

    fn stretched(seed: &Md5, index: usize) -> Self {
        let mut hex_hash = [0u8; 32];
        let mut bin_hash = Output::<Md5>::default();
        let mut hasher = seed.clone().chain_update(itoa::Buffer::new().format(index));

        for _ in 0..=2016 {
            hasher.finalize_into_reset(&mut bin_hash);
            bin_hash
                .view_bits::<Msb0>()
                .chunks(4)
                .map(|nibble| HEX[nibble.load::<usize>()])
                .enumerate()
                .for_each(|(i, byte)| hex_hash[i] = byte);
            hasher.update(hex_hash);
        }

        Self::with_hash(bin_hash)
    }

    fn with_hash(hash: Output<Md5>) -> Self {
        let bits = hash.view_bits::<Msb0>();

        Self {
            triplet: bits.chunks_exact(4).tuple_windows().find_map(|(a, b, c)| {
                if a == b && b == c {
                    let idx = BitIdx::new(a.load()).unwrap();
                    Some(idx)
                } else {
                    None
                }
            }),
            quintet_mask: bits
                .chunks_exact(4)
                .tuple_windows()
                .filter_map(|(a, b, c, d, e)| {
                    if a == b && b == c && c == d && d == e {
                        Some(a.load::<u8>())
                    } else {
                        None
                    }
                })
                .fold(0u16, |mask, b| mask | (0b1000_0000_0000_0000 >> b)),
        }
    }
}

#[test]
fn example() {
    use pretty_assertions::assert_eq;

    assert_eq!(22728, sixty_fourth_key("abc", HashInfo::new).unwrap());
    assert_eq!(22551, sixty_fourth_key("abc", HashInfo::stretched).unwrap());
}
