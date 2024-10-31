use std::{borrow::Cow, iter::FusedIterator, mem, sync::LazyLock};

use regex::{Regex, RegexBuilder};

use crate::meta::Problem;

pub const EXPLOSIVES_IN_CYBERSPACE: Problem =
    Problem::solved(&str::decompressed_size::<V1>, &str::decompressed_size::<V2>);

static MARKER_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    RegexBuilder::new(r"\(\d+x\d+\)")
        .unicode(false)
        .build()
        .unwrap()
});

trait DecompressionStrategy {
    type Decompressor<'a>: Iterator<Item = Cow<'a, str>> + From<&'a str>;
}

trait Decompressable {
    fn decompress<S: DecompressionStrategy>(&self) -> Cow<str>;

    #[inline]
    fn decompressed_size<S: DecompressionStrategy>(&self) -> usize {
        self.decompress::<S>().len()
    }
}

impl<T: AsRef<str> + ?Sized> Decompressable for T {
    fn decompress<S: DecompressionStrategy>(&self) -> Cow<str> {
        let mut decompressor = S::Decompressor::from(self.as_ref());

        let Some(first_chunk) = decompressor.next() else {
            return Cow::Borrowed("");
        };

        let Some(second_chunk) = decompressor.next() else {
            return first_chunk;
        };

        let mut out = first_chunk.into_owned();
        out.push_str(&second_chunk);

        for chunk in decompressor {
            out.push_str(&chunk);
        }

        Cow::Owned(out)
    }
}

#[derive(Debug)]
struct V1;

impl DecompressionStrategy for V1 {
    type Decompressor<'a> = V1Decompressor<'a>;
}

#[derive(Debug)]
struct V1Decompressor<'s> {
    source: Cow<'s, str>,
}

impl<'s> V1Decompressor<'s> {
    #[inline]
    const fn new(source: Cow<'s, str>) -> Self {
        Self { source }
    }
}

impl<'s> From<&'s str> for V1Decompressor<'s> {
    #[inline]
    fn from(source: &'s str) -> Self {
        Self::new(Cow::Borrowed(source))
    }
}

impl<'s> Iterator for V1Decompressor<'s> {
    type Item = Cow<'s, str>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.source.is_empty() {
            return None;
        }

        let Some(marker) = MARKER_REGEX.find(&self.source) else {
            // No more markers, return the whole string

            let rest = mem::replace(&mut self.source, Cow::Borrowed(""));
            return Some(rest);
        };

        let marker_start = marker.start();

        if marker_start > 0 {
            // The next marker is offset from the start of `self.source`, so we have an uncompressed section.

            match &mut self.source {
                Cow::Borrowed(source) => {
                    let (next_chunk, rest) = source.split_at(marker_start);
                    *source = rest;

                    Some(Cow::Borrowed(next_chunk))
                }
                Cow::Owned(source) => {
                    let rest = source.split_off(marker_start);
                    let next_chunk = mem::replace(source, rest);

                    Some(Cow::Owned(next_chunk))
                }
            }
        } else {
            // The next marker is at the start of `self.source`, so we have a compressed section.
            let marker_end = marker.end();

            // Retrieve the `(NxN)` pattern
            let pattern = match &mut self.source {
                Cow::Borrowed(source) => {
                    let (pattern, rest) = source.split_at(marker_end);
                    *source = rest;
                    Cow::Borrowed(pattern)
                }
                Cow::Owned(source) => {
                    let rest = source.split_off(marker_end);
                    let pattern = mem::replace(source, rest);
                    Cow::Owned(pattern)
                }
            };

            // Safety: since our pattern starts and ends with parens, which are one byte long, this is safe.
            let pattern_inner = unsafe { pattern.get_unchecked(1..(pattern.len() - 1)) };

            // Safety: our pattern contains 'x', so this is always guarenteed to hit.
            let (num_chars_str, num_reps_str) =
                unsafe { pattern_inner.split_once('x').unwrap_unchecked() };

            // Safety: it's always valid to `parse` decimal digits to `usize`
            let (num_chars, num_reps) = unsafe {
                (
                    num_chars_str.parse().unwrap_unchecked(),
                    num_reps_str.parse().unwrap_unchecked(),
                )
            };

            let to_repeat = match &mut self.source {
                Cow::Borrowed(source) => {
                    let (to_repeat, rest) = source.split_at(num_chars);
                    *source = rest;
                    Cow::Borrowed(to_repeat)
                }
                Cow::Owned(source) => {
                    let rest = source.split_off(num_chars);
                    let to_repeat = mem::replace(source, rest);
                    Cow::Owned(to_repeat)
                }
            };

            let decompressed = if num_reps <= 1 {
                to_repeat
            } else {
                Cow::Owned(to_repeat.repeat(num_reps))
            };
            Some(decompressed)
        }
    }
}

impl FusedIterator for V1Decompressor<'_> {}

#[derive(Debug)]
struct V2;

impl DecompressionStrategy for V2 {
    type Decompressor<'a> = V2Decompressor<'a>;
}

#[derive(Debug)]
struct V2Decompressor<'a> {
    inner: V1Decompressor<'a>,
    stack: Option<Box<V2Decompressor<'a>>>,
}

impl<'a> V2Decompressor<'a> {
    #[inline]
    const fn new(source: Cow<'a, str>) -> Self {
        Self {
            inner: V1Decompressor::new(source),
            stack: None,
        }
    }
}

impl<'a> From<&'a str> for V2Decompressor<'a> {
    #[inline]
    fn from(value: &'a str) -> Self {
        Self::new(Cow::Borrowed(value))
    }
}

impl<'a> Iterator for V2Decompressor<'a> {
    type Item = Cow<'a, str>;

    fn next(&mut self) -> Option<Self::Item> {
        self.stack
            .as_deref_mut()
            .and_then(V2Decompressor::next)
            .or_else(|| {
                let next_chunk = self.inner.next()?;

                if MARKER_REGEX.is_match(&next_chunk) {
                    self.stack = Some(Box::new(V2Decompressor::new(next_chunk)));
                    self.next()
                } else {
                    Some(next_chunk)
                }
            })
    }
}
