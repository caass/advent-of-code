use bitvec::{index::BitIdx, prelude::*};
use eyre::{Result, eyre};
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
const N: usize = 0x8000;

fn sixty_fourth_key<F: Send + Sync + Fn(&Md5, usize) -> HashInfo>(
    salt: &str,
    f: F,
) -> Result<usize> {
    let salt = Md5::new_with_prefix(salt);
    let mut hashes = Vec::with_capacity(N);

    (0..N)
        .into_par_iter()
        .map(|i| f(&salt, i))
        .collect_into_vec(&mut hashes);

    hashes
        .iter()
        .enumerate()
        .filter(|(i, h)| {
            h.triplet.is_some_and(|hex_char| {
                (i + 1..=i + 1000)
                    .into_par_iter()
                    .any(|j| hashes[j].quintet_mask.get_bit::<Msb0>(hex_char))
            })
        })
        .nth(63)
        .map(|(i, _)| i)
        .ok_or_else(|| eyre!("fewer than 64 keys in first {N} indices!"))
}

struct HashInfo {
    triplet: Option<BitIdx<u16>>,
    quintet_mask: u16,
}

impl HashInfo {
    fn new(salt: &Md5, index: usize) -> Self {
        let mut buf = itoa::Buffer::new();
        let hash = salt.clone().chain_update(buf.format(index)).finalize();

        Self::with_hash(hash)
    }

    fn stretched(salt: &Md5, index: usize) -> Self {
        let mut hex_hash = [0u8; 32];
        let mut bin_hash = Output::<Md5>::default();
        let mut hasher = salt.clone().chain_update(itoa::Buffer::new().format(index));

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
