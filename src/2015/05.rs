use eyre::Result;
use rayon::prelude::*;

use crate::meta::problem;

problem!(part_1, part_2);

const VOWELS: [char; 5] = ['a', 'e', 'i', 'o', 'u'];
const BANNED_PAIRS: [&[u8]; 4] = [b"ab", b"cd", b"pq", b"xy"];

fn part_1(input: &str) -> Result<usize> {
    // A nice string is one with all of the following properties:
    //
    // - It contains at least three vowels (aeiou only), like aei, xazegov, or aeiouaeiouaeiou.
    // - It contains at least one letter that appears twice in a row, like xx, abcdde (dd), or aabbccdd (aa, bb, cc, or dd).
    // - It does not contain the strings ab, cd, pq, or xy, even if they are part of one of the other requirements.

    Ok(input
        .par_lines()
        .filter(|line| {
            std::thread::scope(|s| {
                let vowels_handle =
                    s.spawn(|| line.chars().filter(|c| VOWELS.contains(c)).count() >= 3);
                let pairs_handle = s.spawn(|| line.as_bytes().windows(2).any(|w| w[0] == w[1]));
                let banned_handle = s.spawn(|| {
                    line.as_bytes()
                        .windows(2)
                        .any(|w| BANNED_PAIRS.contains(&w))
                });

                let has_three_vowels = vowels_handle.join().expect("thread not to panic");
                let has_paired_letter = pairs_handle.join().expect("thread not to panic");
                let has_banned_pair = banned_handle.join().expect("thread not to panic");

                has_three_vowels && has_paired_letter && !has_banned_pair
            })
        })
        .count())
}

fn part_2(input: &str) -> Result<usize> {
    Ok(input
        .par_lines()
        .filter(|line| {
            std::thread::scope(|s| {
                let pair_handle = s.spawn(|| {
                    let start = 0..=(line.len() - 4);
                    let end = 2..=(line.len() - 2);
                    start.zip(end).any(|(a, b)| line[b..].contains(&line[a..b]))
                });

                let sandwich_handle =
                    s.spawn(|| line.as_bytes().windows(3).any(|slice| slice[0] == slice[2]));

                let has_nonoverlapping_pair = pair_handle.join().expect("thread not to panic");
                let has_sandwich_pair = sandwich_handle.join().expect("thread not to panic");

                has_nonoverlapping_pair && has_sandwich_pair
            })
        })
        .count())
}
