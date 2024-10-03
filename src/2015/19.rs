use std::{
    borrow::{Borrow, Cow},
    collections::{HashMap, HashSet},
    fmt::{self, Display, Formatter},
    iter::{Copied, Flatten},
    ops::Deref,
    ptr,
    str::FromStr,
};

use elsa::FrozenIndexSet;
use eyre::{bail, eyre, Context, OptionExt, Report, Result};
use fnv::FnvBuildHasher;
use rayon::prelude::*;
use stable_deref_trait::StableDeref;
use winnow::{
    ascii::{alpha1, alphanumeric1},
    combinator::separated_pair,
    error::ContextError,
    prelude::*,
};

#[cfg(test)]
use pretty_assertions::assert_eq;

use crate::common::ascii_ext::AsciiExt;
use crate::meta::{problem, Problem};

pub const MEDICINE_FOR_RUDOLPH: Problem = problem!();

impl<'a> TryFrom<&'a str> for &'a Atom {
    type Error = Report;

    fn try_from(s: &'a str) -> Result<&'a Atom> {
        Atom::new(s)
    }
}

#[derive(Debug)]
#[repr(transparent)]
struct Atom(str);

impl Atom {
    #[inline(always)]
    fn new<S: AsRef<str> + ?Sized>(s: &S) -> Result<&Self> {
        let s = s.as_ref();

        if s.is_ascii_alphabetic() {
            Ok(unsafe { Self::new_unchecked(s) })
        } else {
            Err(eyre!("{s} is not a valid atom"))
        }
    }

    #[inline(always)]
    unsafe fn new_unchecked(s: &str) -> &Self {
        // Safety: `Atom` is guaranteed to have the same layout as `str`
        unsafe { &*{ s as *const str as *const Atom } }
    }
}

impl Deref for Atom {
    type Target = str;

    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug)]
#[repr(transparent)]
struct Molecule(str);

impl<'a> TryFrom<&'a str> for &'a Molecule {
    type Error = Report;

    fn try_from(s: &'a str) -> Result<Self> {
        Molecule::new(s)
    }
}

impl Molecule {
    const ELECTRON: &'static Molecule = unsafe { Molecule::new_unchecked("e") };

    #[inline(always)]
    fn new<S: AsRef<str> + ?Sized>(s: &S) -> Result<&Molecule> {
        let s = s.as_ref();

        if s.is_ascii_alphabetic() {
            // Safety: `s` meets the critera for `new_unchecked` and the contract is upheld.
            Ok(unsafe { Molecule::new_unchecked(s) })
        } else {
            Err(eyre!("{s} is not a valid molecule"))
        }
    }

    /// Safety: The given string slice must contain only ASCII alphabetical characters.
    #[inline(always)]
    const unsafe fn new_unchecked(s: &str) -> &Molecule {
        // Safety:
        //
        // There are two safety concerns here:
        // 1. Does the contained `str` contain only ascii letters?
        // 2. Is it legal to cast from an `&str` to an `&Molecule`?
        //
        // The answer to 1 is yes, given that the caller upholds the contract for this function.
        // The answer to 2 is yes, since `Molecule` is a transparent wrapper around `str`.
        unsafe { &*{ s as *const str as *const Molecule } }
    }

    fn atoms(&self) -> Atoms {
        Atoms::new(self)
    }
}

impl Deref for Molecule {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> AsRef<T> for Molecule
where
    <Self as Deref>::Target: AsRef<T>,
{
    fn as_ref(&self) -> &T {
        self.deref().as_ref()
    }
}

impl ToOwned for Molecule {
    type Owned = SynthesizedMolecule;

    fn to_owned(&self) -> Self::Owned {
        SynthesizedMolecule(self.0.to_string())
    }
}

#[derive(Debug)]
struct SynthesizedMolecule(String);

impl<T> AsRef<T> for SynthesizedMolecule
where
    <Self as Deref>::Target: AsRef<T>,
{
    fn as_ref(&self) -> &T {
        self.deref().as_ref()
    }
}

impl Borrow<Molecule> for SynthesizedMolecule {
    fn borrow(&self) -> &Molecule {
        self.deref()
    }
}

impl Deref for SynthesizedMolecule {
    type Target = Molecule;

    fn deref(&self) -> &Self::Target {
        // Safety: we ke know the contained string is ascii alphabetic due to `SynthesizedMolecule`'s constructor.
        unsafe { Molecule::new_unchecked(&self.0) }
    }
}

#[derive(Debug)]
struct Transformation<'a> {
    from: &'a Atom,
    to: &'a Molecule,
}

impl<'a> TryFrom<&'a str> for Transformation<'a> {
    type Error = Report;

    fn try_from(value: &'a str) -> Result<Self> {
        let (from_str, to_str) = value
            .split_once(" => ")
            .ok_or_else(|| eyre!("No separator found in transformation: \"{value}\""))?;
        let from = from_str.try_into()?;
        let to = to_str.try_into()?;

        Ok(Self { from, to })
    }
}

#[derive(Debug)]
struct Atoms<'a> {
    molecule: &'a Molecule,
}

impl<'a> Atoms<'a> {
    fn new(molecule: &'a Molecule) -> Self {
        Self { molecule }
    }
}

impl<'a> Iterator for Atoms<'a> {
    type Item = &'a Atom;

    fn next(&mut self) -> Option<Self::Item> {
        let next_capital_letter_idx = self
            .molecule
            .get(1..)?
            .find(|ch: char| ch.is_ascii_uppercase())
            .map(|i| i + 1)
            .unwrap_or_else(|| self.molecule.len());

        let (atom, molecule) = self
            .molecule
            .split_at_checked(next_capital_letter_idx)
            .map(|(atom, molecule)|
                // Safety: since the molecule is ascii alphabetic, these parts of it are also ascii alphabetic
                unsafe {
                (Atom::new_unchecked(atom), Molecule::new_unchecked(molecule))
            })
            .unwrap_or_else(|| panic!("logic failure, this should always be ok"));

        // Safety: a subset of a molecule (an ascii alphabetic string) is also ascii alphabetic.
        self.molecule = molecule;
        Some(atom)
    }
}
