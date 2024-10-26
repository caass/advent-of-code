use std::fmt::{self, Display, Formatter};
use std::hint::unreachable_unchecked;
use std::mem::transmute;
use std::num::NonZeroU8;
use std::ops::Deref;

use eyre::{OptionExt, Result};
use md5::{digest::Output, Digest, Md5};
use rayon::prelude::*;

use crate::common::U32_MAX;
use crate::meta::Problem;

pub const HOW_ABOUT_A_NICE_GAME_OF_CHESS: Problem =
    Problem::solved(&Password::for_door_1, &Password::for_door_2);

#[derive(Default, Debug)]
struct Password([u8; 8]);

impl Deref for Password {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        unsafe { std::str::from_utf8_unchecked(&self.0) }
    }
}

impl Display for Password {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        Display::fmt(&**self, f)
    }
}

impl Password {
    fn from_hex_digits(digits: [Option<NonZeroU8>; 8]) -> Result<Self> {
        digits
            .iter()
            .all(Option::is_some)
            .then_some(Self(unsafe {
                transmute::<[Option<NonZeroU8>; 8], [u8; 8]>(digits)
            }))
            .ok_or_eyre("Didn't fill in all the digits in the password")
    }

    fn for_door_1(door_id: &str) -> Result<Self> {
        let f = Password::generator(
            // Return `true` if the hash starts with five zeroes
            |hash, _| hash[0] == 0 && hash[1] == 0 && hash[2] < 0x10,
            // Update the password digit-by-digit.
            |hash, digits| {
                if let Some(slot) = digits.iter_mut().find(|opt| opt.is_none()) {
                    *slot = Some(lower_bits_to_hex(hash[2]));
                };
            },
        );

        f(door_id)
    }

    fn for_door_2(door_id: &str) -> Result<Self> {
        let f = Password::generator(
            // Return `true` if
            // - the hash starts with five zeroes
            // - the sixth digit is between 0-7
            // - the sixth digit indexes to an empty slot
            |hash, digits| {
                hash[0] == 0
                    && hash[1] == 0
                    && hash[2] <= 0x7
                    // Safety: we just checked that `hash[2]` is <= 7
                    && unsafe { digits.get_unchecked(hash[2] as usize) }.is_none()
            },
            // Set the digit
            // ...at the position indicated by the sixth hex digit
            // ...to the value indicated by the seventh digit
            |hash, digits| {
                digits[hash[2] as usize] = Some(upper_bits_to_hex(hash[3]));
            },
        );

        f(door_id)
    }

    /// Construct a password generator that applies `F` to find valid hashes,
    /// and then `U` to use that hash to update the password.
    fn generator<F, U>(finder: F, updater: U) -> impl Fn(&str) -> Result<Self>
    where
        F: Sync + Fn(&Output<Md5>, &[Option<NonZeroU8>; 8]) -> bool,
        U: Fn(Output<Md5>, &mut [Option<NonZeroU8>; 8]),
    {
        move |door_id| {
            let mut digits = [None; 8];
            let mut hash_idx_start = 0;

            let mut base = Md5::new();
            base.update(door_id);

            while digits.iter().any(Option::is_none) {
                let (hash, hash_idx) = (hash_idx_start..U32_MAX)
                    .into_par_iter()
                    .by_exponential_blocks()
                    .find_map_first(|index| {
                        let mut buf = itoa::Buffer::new();
                        let slice = buf.format(index);

                        let mut hasher: Md5 = Md5::clone(&base);
                        Digest::update(&mut hasher, slice);
                        let result = Digest::finalize(hasher);

                        (finder)(&result, &digits).then_some((result, index))
                    })
                    .ok_or_eyre("ran out of hashes")?;

                hash_idx_start = hash_idx + 1;
                (updater)(hash, &mut digits);
            }

            Self::from_hex_digits(digits)
        }
    }
}

#[inline]
const fn lower_bits_to_hex(byte: u8) -> NonZeroU8 {
    bits_to_hex::<false>(byte)
}

#[inline]
const fn upper_bits_to_hex(byte: u8) -> NonZeroU8 {
    bits_to_hex::<true>(byte)
}

#[inline]
const fn bits_to_hex<const UPPER: bool>(byte: u8) -> NonZeroU8 {
    let hex_char = match if UPPER { byte >> 4 } else { byte & 0xF } {
        0x0 => b'0',
        0x1 => b'1',
        0x2 => b'2',
        0x3 => b'3',
        0x4 => b'4',
        0x5 => b'5',
        0x6 => b'6',
        0x7 => b'7',
        0x8 => b'8',
        0x9 => b'9',
        0xA => b'a',
        0xB => b'b',
        0xC => b'c',
        0xD => b'd',
        0xE => b'e',
        0xF => b'f',

        _ => unsafe { unreachable_unchecked() },
    };

    unsafe { NonZeroU8::new_unchecked(hex_char) }
}

#[test]
fn example() {
    assert_eq!(&*Password::for_door_1("abc").unwrap(), "18f47a30");
    assert_eq!(&*Password::for_door_2("abc").unwrap(), "05ace8e3");
}
