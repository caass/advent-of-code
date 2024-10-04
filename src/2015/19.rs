use std::collections::{HashMap, HashSet};
use std::fmt::{self, Display, Formatter};
use std::hash::{Hash, Hasher};
use std::ops::Deref;
use std::ptr;
use std::str::FromStr;

use eyre::{bail, eyre, Context, OptionExt, Report, Result};
use fnv::FnvBuildHasher;
use nohash_hasher::BuildNoHashHasher;
use rayon::prelude::*;
use tinystr::TinyAsciiStr;
use tinyvec::ArrayVec;

use crate::meta::{problem, Problem};

pub const MEDICINE_FOR_RUDOLPH: Problem =
    problem!(|input: &str| { input.parse::<ChemLab>().map(|lab| lab.calibrate()) });

type Syntheses = HashMap<Atom, HashSet<TargetMolecule, FnvBuildHasher>, AtomHasher>;
type Reductions = HashMap<TargetMolecule, HashSet<Atom, BuildNoHashHasher<Atom>>, FnvBuildHasher>;

#[derive(Debug)]
struct ChemLab {
    syntheses: Syntheses,
    reductions: Reductions,
    target: SynthesizedMolecule,
}

impl ChemLab {
    fn calibrate(&self) -> usize {
        self.target.syntheses(&self.syntheses).len()
    }
}

impl FromStr for ChemLab {
    type Err = Report;

    fn from_str(s: &str) -> Result<Self> {
        let mut lines = s.trim().lines().map(|line| line.trim());

        let target = lines.next_back().ok_or_eyre("empty input")?.parse()?;

        if !lines
            .next_back()
            .ok_or_eyre("input contained only one line")?
            .is_empty()
        {
            bail!("Expected empty line before target molecule");
        };

        let mut syntheses = Syntheses::default();
        let mut reductions = Reductions::default();

        for line in lines {
            let (atom_str, molecule_str) = line
                .split_once(" => ")
                .ok_or_eyre("couldn't find delimiter in transformation")?;

            let atom = atom_str.parse()?;
            let molecule = molecule_str.parse()?;

            syntheses.entry(atom).or_default().insert(molecule);
            reductions.entry(molecule).or_default().insert(atom);
        }

        Ok(ChemLab {
            syntheses,
            reductions,
            target,
        })
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
struct Atom(TinyAsciiStr<4>);

impl Atom {
    const ELECTRON: Atom = unsafe { Atom::new_unchecked(b"e\0\0\0") };

    fn new(s: &str) -> Result<Self> {
        let tiny = TinyAsciiStr::from_str(s).wrap_err("invalid atom")?;
        tiny.is_ascii_alphabetic()
            .then_some(Atom(tiny))
            .ok_or_else(|| eyre!("invalid atom \"{s}\", atoms must be ascii alphabetic"))
    }

    #[inline(always)]
    const unsafe fn new_unchecked(bytes: &[u8; 4]) -> Self {
        Atom(TinyAsciiStr::from_bytes_unchecked(*bytes))
    }
}

impl FromStr for Atom {
    type Err = Report;

    fn from_str(s: &str) -> Result<Self> {
        Atom::new(s)
    }
}

impl Default for Atom {
    #[inline(always)]
    fn default() -> Self {
        const { unsafe { Atom::new_unchecked(b"\0\0\0\0") } }
    }
}

impl Hash for Atom {
    fn hash<H: Hasher>(&self, state: &mut H) {
        let bytes: &[u8; 4] = self.0.all_bytes();
        let p: *const u32 = ptr::from_ref(bytes).cast();
        state.write_u32(unsafe { *p });
    }
}

impl nohash_hasher::IsEnabled for Atom {}
type AtomHasher = BuildNoHashHasher<Atom>;

impl Display for Atom {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str(self.0.as_str())
    }
}

#[repr(transparent)]
struct Molecule([Atom]);

impl Deref for Molecule {
    type Target = [Atom];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Molecule {
    #[inline(always)]
    const fn new(atoms: &[Atom]) -> &Molecule {
        let atoms: *const [Atom] = std::ptr::from_ref(atoms);
        let this: *const Molecule = atoms as _;
        unsafe { &*this }
    }

    #[inline(always)]
    fn atoms(&self) -> impl IndexedParallelIterator<Item = &Atom> + '_ {
        self.par_iter()
    }

    fn syntheses(
        &self,
        transformations: &Syntheses,
    ) -> HashSet<SynthesizedMolecule, FnvBuildHasher> {
        self.atoms()
            .enumerate()
            .flat_map(|(i, atom)| {
                transformations
                    .get(atom)
                    .map(|set| set.par_iter().map(move |target| self.synthesize(i, target)))
            })
            .flatten()
            .collect()
    }

    fn synthesize(&self, from: usize, to: &TargetMolecule) -> SynthesizedMolecule {
        let before = &self.0[..from];
        let after = self.0.get((from + 1)..).unwrap_or_default();

        let mut out = Vec::with_capacity(before.len() + to.len() + after.len());
        out.extend_from_slice(before);
        out.extend_from_slice(to);
        out.extend_from_slice(after);

        SynthesizedMolecule(out)
    }
}

/// Represents a molecule that can be transformed into.
#[derive(Default, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy)]
struct TargetMolecule(ArrayVec<[Atom; 8]>);

impl Deref for TargetMolecule {
    type Target = Molecule;

    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        Molecule::new(&self.0)
    }
}

impl Display for TargetMolecule {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        self.0.iter().try_for_each(|atom| Display::fmt(atom, f))
    }
}

impl FromStr for TargetMolecule {
    type Err = Report;

    fn from_str(s: &str) -> Result<Self> {
        split_uppercase(s).map(Atom::from_str).collect()
    }
}

impl FromIterator<Atom> for TargetMolecule {
    fn from_iter<T: IntoIterator<Item = Atom>>(iter: T) -> Self {
        Self(ArrayVec::from_iter(iter))
    }
}

/// Represents a molecule synthesized via a sequence of transformations
#[derive(Default, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct SynthesizedMolecule(Vec<Atom>);

impl Display for SynthesizedMolecule {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        self.0.iter().try_for_each(|atom| Display::fmt(atom, f))
    }
}

impl FromStr for SynthesizedMolecule {
    type Err = Report;

    fn from_str(s: &str) -> Result<Self> {
        split_uppercase(s).map(Atom::from_str).collect()
    }
}

impl FromIterator<Atom> for SynthesizedMolecule {
    fn from_iter<T: IntoIterator<Item = Atom>>(iter: T) -> Self {
        Self(Vec::from_iter(iter))
    }
}

impl Deref for SynthesizedMolecule {
    type Target = Molecule;

    fn deref(&self) -> &Self::Target {
        Molecule::new(&self.0)
    }
}

fn split_uppercase(s: &str) -> SplitUppercase {
    SplitUppercase(s)
}

#[derive(Debug)]
struct SplitUppercase<'a>(&'a str);

impl<'a> Iterator for SplitUppercase<'a> {
    type Item = &'a str;

    fn next(&mut self) -> Option<Self::Item> {
        let chunk_end = self
            .0
            .get(1..)?
            .find(|ch: char| ch.is_ascii_uppercase())
            .map(|i| i + 1)
            .unwrap_or(self.0.len());

        // Safety: it's always legal to split at `chunk_end`, since `chunk_end` <= `self.0.len()`.
        let (next, rest) = unsafe { self.0.split_at_checked(chunk_end).unwrap_unchecked() };
        self.0 = rest;

        Some(next)
    }
}
