use bitvec::{index::BitIdx, prelude::*};
use eyre::{Result, bail};
use itertools::Itertools;
use md5::{Digest, Md5};

use aoc_meta::Problem;

pub const ONE_TIME_PAD: Problem = Problem::partially_solved(&sixty_fourth_key);

fn sixty_fourth_key(salt: &str) -> Result<usize> {
    let salt = Md5::new_with_prefix(salt);

    let mut key_count = 0;
    let mut hashes = (0..1000).map(|i| HashInfo::new(&salt, i)).collect_vec();

    for i in 0..isize::MAX as usize {
        hashes.push(HashInfo::new(&salt, hashes.len()));

        if let Some(hex_char) = hashes[i].triplet
            && (i + 1..=i + 1000).any(|j| hashes[j].quintet_mask.get_bit::<Msb0>(hex_char))
        {
            key_count += 1;
            if key_count == 64 {
                return Ok(i);
            }
        }
    }

    bail!("fewer than 64 keys!")
}

struct HashInfo {
    triplet: Option<BitIdx<u16>>,
    quintet_mask: u16,
}

impl HashInfo {
    fn new(salt: &Md5, index: usize) -> Self {
        let mut buf = itoa::Buffer::new();
        let hash = salt.clone().chain_update(buf.format(index)).finalize();
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
