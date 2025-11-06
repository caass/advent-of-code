use std::convert::Infallible;
use std::sync::LazyLock;

use regex::{Regex, RegexBuilder};

use aoc_meta::Problem;

pub const EXPLOSIVES_IN_CYBERSPACE: Problem = Problem::solved(&v1, &v2);

static MARKER_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    RegexBuilder::new(r"\(\d+x\d+\)")
        .unicode(false)
        .build()
        .unwrap()
});

fn v1(compressed: &str) -> Result<usize, Infallible> {
    Ok(decompressed_len(false, compressed))
}

fn v2(compressed: &str) -> Result<usize, Infallible> {
    Ok(decompressed_len(true, compressed))
}

fn decompressed_len(recurse: bool, compressed: &str) -> usize {
    let Some(mat) = MARKER_REGEX.find(compressed) else {
        return compressed.len();
    };

    if mat.start() > 0 {
        return mat.start() + decompressed_len(recurse, &compressed[mat.start()..]);
    }

    let (pattern, compressed) = compressed.split_at(mat.end());
    // safety: we're using the regex pattern as an argument
    let (len, reps) = unsafe { to_len_and_reps(pattern) };

    let (to_repeat, rest) = compressed.split_at(len);

    let n = if recurse {
        decompressed_len(true, to_repeat)
    } else {
        to_repeat.len()
    };

    n * reps + decompressed_len(recurse, rest)
}

// safety: you must pass a string matching the regex MARKER_REGEX
unsafe fn to_len_and_reps(pattern: &str) -> (usize, usize) {
    // Safety: since our pattern starts and ends with parens, which are one byte long, this is safe.
    let inner = unsafe { pattern.get_unchecked(1..(pattern.len() - 1)) };

    // Safety: our pattern contains 'x', so this is always guarenteed to hit.
    let (len, reps) = unsafe { inner.split_once('x').unwrap_unchecked() };

    // Safety: it's always valid to `parse` decimal digits to `usize`
    unsafe {
        (
            len.parse().unwrap_unchecked(),
            reps.parse().unwrap_unchecked(),
        )
    }
}
