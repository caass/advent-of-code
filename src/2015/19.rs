use std::{
    collections::{HashMap, HashSet},
    ops::Deref,
    str::FromStr,
};

use eyre::{bail, eyre, OptionExt, Report};
use fnv::FnvBuildHasher;
use rayon::prelude::*;
use tinystr::TinyAsciiStr;

use crate::meta::Problem;

pub const MEDICINE_FOR_RUDOLPH: Problem =
    Problem::partially_solved(&|input| input.parse().map(|lab: Lab| lab.plus_ultra().len()));

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
}

#[derive(Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
struct Molecule(Vec<Atom>);

impl Molecule {
    fn atoms(&self) -> impl IndexedParallelIterator<Item = &Atom> + '_ {
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

    #[inline(always)]
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
                .map(|up| up + 1)
                .unwrap_or(remaining.len());

            let (this_atom, rest) = remaining.split_at(next_atom_start);

            atoms.push(this_atom.parse()?);
            *remaining = rest;
        }

        Ok(Self(atoms))
    }
}

#[derive(Debug, Hash, PartialEq, Eq, PartialOrd, Ord, Copy, Clone)]
struct Atom(TinyAsciiStr<2>);

impl FromStr for Atom {
    type Err = <TinyAsciiStr<2> as FromStr>::Err;

    #[inline(always)]
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        s.parse().map(Atom)
    }
}

impl Deref for Atom {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        self.0.deref()
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
        let mut lines = s.trim().lines().map(|line| line.trim());
        let target = lines
            .next_back()
            .ok_or_eyre("puzzle input was empty")?
            .parse()?;

        if !lines.next_back().is_some_and(|line| line.is_empty()) {
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
