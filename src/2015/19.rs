use std::collections::{HashMap, HashSet};
use std::ops::Deref;
use std::str::FromStr;
use std::sync::atomic::{AtomicUsize, Ordering};

use eyre::{bail, eyre, OptionExt, Report};
use fnv::FnvBuildHasher;
use rayon::prelude::*;
use tinystr::TinyAsciiStr;

use crate::meta::Problem;

/// <https://adventofcode.com/2015/day/19>
pub const MEDICINE_FOR_RUDOLPH: Problem = Problem::solved(
    &|input| input.parse().map(|lab: Lab| lab.plus_ultra().len()),
    &|input| input.parse().map(|lab: Lab| lab.num_steps()),
);

#[derive(Debug)]
struct Lab {
    reactions: Reactions,
    target: Molecule,
}

impl Lab {
    /// Go even further beyond; determine the number of syntheses possible after synthesizing the target molecule.
    fn plus_ultra(&self) -> HashSet<Molecule, FnvBuildHasher> {
        self.target
            .atoms()
            .enumerate()
            .flat_map(|(idx, atom)| {
                self.reactions
                    .get(atom)
                    .map(move |molecules| (idx, molecules))
            })
            .flat_map(|(from, tos)| tos.par_iter().map(move |to| self.target.replace(from, to)))
            .collect()
    }

    /// Determine the number of steps required to synthesize the target molecule from a single electron.
    fn num_steps(&self) -> usize {
        let rn_ar_count = AtomicUsize::new(0);
        let y_count = AtomicUsize::new(0);

        self.target.atoms().copied().for_each(|atom| {
            if atom == Atom::RN || atom == Atom::AR {
                rn_ar_count.fetch_add(1, Ordering::Relaxed);
            } else if atom == Atom::Y {
                y_count.fetch_add(1, Ordering::Relaxed);
            };
        });

        self.target.len() - rn_ar_count.into_inner() - 2 * y_count.into_inner() - 1
    }
}

#[derive(Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
struct Molecule(Vec<Atom>);

impl Molecule {
    fn atoms(&self) -> impl IndexedParallelIterator<Item = &Atom> + use<'_> {
        self.0.par_iter()
    }

    fn replace(&self, from: usize, to: &Molecule) -> Molecule {
        let mut out = Molecule::with_capacity(self.len() + to.len() - 1);
        out.0.extend_from_slice(&self.0[..from]);
        out.0.extend_from_slice(&to.0);
        if let Some(after) = self.0.get((from + 1)..) {
            out.0.extend_from_slice(after);
        }

        out
    }

    #[inline]
    fn len(&self) -> usize {
        self.0.len()
    }

    fn with_capacity(capacity: usize) -> Self {
        Self(Vec::with_capacity(capacity))
    }
}

impl FromStr for Molecule {
    type Err = Report;

    fn from_str(mut s: &str) -> Result<Self, Self::Err> {
        let mut atoms = Vec::with_capacity(s.len());

        let remaining = &mut s;
        while !remaining.is_empty() {
            let next_atom_start = remaining[1..]
                .find(|ch: char| ch.is_ascii_uppercase())
                .map_or(remaining.len(), |up| up + 1);

            let (this_atom, rest) = remaining.split_at(next_atom_start);

            atoms.push(this_atom.parse()?);
            *remaining = rest;
        }

        Ok(Self(atoms))
    }
}

#[derive(Debug, Hash, PartialEq, Eq, PartialOrd, Ord, Copy, Clone)]
struct Atom(TinyAsciiStr<2>);

impl Atom {
    const RN: Atom = Atom(unsafe { TinyAsciiStr::from_utf8_unchecked(*b"Rn") });
    const AR: Atom = Atom(unsafe { TinyAsciiStr::from_utf8_unchecked(*b"Ar") });
    const Y: Atom = Atom(unsafe { TinyAsciiStr::from_utf8_unchecked(*b"Y\0") });
}

impl FromStr for Atom {
    type Err = <TinyAsciiStr<2> as FromStr>::Err;

    #[inline]
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        s.parse().map(Atom)
    }
}

impl Deref for Atom {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        #[allow(
            clippy::explicit_auto_deref,
            reason = "the compiler actually won't auto-deref here for some reason"
        )]
        &*self.0
    }
}

#[derive(Debug, Default)]
struct Reactions(HashMap<Atom, HashSet<Molecule, FnvBuildHasher>, FnvBuildHasher>);

impl Deref for Reactions {
    type Target = HashMap<Atom, HashSet<Molecule, FnvBuildHasher>, FnvBuildHasher>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl FromStr for Lab {
    type Err = Report;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut lines = s.trim().lines().map(str::trim);
        let target = lines
            .next_back()
            .ok_or_eyre("puzzle input was empty")?
            .parse()?;

        if !lines.next_back().is_some_and(str::is_empty) {
            bail!("Expected a blank line before target molecule");
        }

        let mut reactions = Reactions::default();
        lines.try_for_each(|line| {
            let (source, target) = line.split_once(" => ").ok_or_else(|| {
                eyre!("Couldn't parse line \"{line}\" as a reaction -- missing `=>`")
            })?;

            reactions
                .0
                .entry(source.parse()?)
                .or_default()
                .insert(target.parse()?);

            Ok::<_, Report>(())
        })?;

        Ok(Self { reactions, target })
    }
}
