use std::ops::{Bound, RangeBounds};

use rayon::prelude::*;
use wide::u8x16;

// TODO: use this?
pub trait AsciiExt {
    fn is_ascii_alphabetic(&self) -> bool;

    fn is_ascii_alphanumeric(&self) -> bool;

    fn is_ascii_lowercase(&self) -> bool;

    fn is_ascii_uppercase(&self) -> bool;

    fn is_ascii_numeric(&self) -> bool;
}

impl<T: AsRef<[u8]> + ?Sized> AsciiExt for T {
    #[inline(always)]
    fn is_ascii_alphabetic(&self) -> bool {
        self.as_ref().iter().all(|byte| byte.is_ascii_alphabetic())
    }

    #[inline(always)]
    fn is_ascii_alphanumeric(&self) -> bool {
        self.as_ref()
            .iter()
            .all(|byte| byte.is_ascii_alphanumeric())
    }

    #[inline(always)]
    fn is_ascii_lowercase(&self) -> bool {
        is_in_range(self, b'a'..=b'z')
    }

    #[inline(always)]
    fn is_ascii_uppercase(&self) -> bool {
        is_in_range(self, b'A'..=b'Z')
    }

    #[inline(always)]
    fn is_ascii_numeric(&self) -> bool {
        is_in_range(self, b'0'..=b'9')
    }
}

#[inline(always)]
fn is_in_range<B: AsRef<[u8]> + ?Sized, R: RangeBounds<u8>>(bytes: &B, range: R) -> bool {
    #[inline(never)]
    fn is_in_range_inner(bytes: &[u8], start: u8, end: u8) -> bool {
        const CHUNK_SIZE: usize = u8x16::LANES as usize;

        let min = u8x16::splat(start);
        let max = u8x16::splat(end);

        let chunks = bytes.par_chunks_exact(CHUNK_SIZE);
        chunks
            .remainder()
            .iter()
            .all(|byte| (start..=end).contains(byte))
            && chunks.all(|chunk| {
                // Safety: we know that the chunk will have size `CHUNK_SIZE`.
                let array: [u8; CHUNK_SIZE] = unsafe { chunk.try_into().unwrap_unchecked() };

                // Check that all values are inside the given range
                let simd = u8x16::new(array);
                simd.min(min).max(max) == simd
            })
    }

    let start = match range.start_bound() {
        Bound::Included(byte) => *byte,
        Bound::Excluded(byte) => *byte + 1,
        Bound::Unbounded => u8::MIN,
    };

    let end = match range.end_bound() {
        Bound::Included(byte) => *byte,
        Bound::Excluded(byte) => *byte - 1,
        Bound::Unbounded => u8::MAX,
    };

    is_in_range_inner(bytes.as_ref(), start, end)
}
