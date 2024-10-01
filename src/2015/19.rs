use std::{
    borrow::Cow,
    collections::{HashMap, HashSet},
    fmt::{self, Display, Formatter},
    iter::{Copied, Flatten},
    ops::Deref,
};

use elsa::FrozenIndexSet;
use eyre::{bail, eyre, Context, OptionExt, Report, Result};
use fnv::FnvBuildHasher;
use rayon::prelude::*;
use stable_deref_trait::StableDeref;
use winnow::{ascii::alphanumeric1, combinator::separated_pair, error::ContextError, prelude::*};

#[cfg(test)]
use pretty_assertions::assert_eq;

use crate::meta::{problem, Problem};

pub const MEDICINE_FOR_RUDOLPH: Problem = problem!(|input: &str| {
    let lab: ChemLab = input.try_into()?;
    let molecules = lab.calibrate();
    Ok::<_, Report>(molecules.len())
});

struct ChemLab<'s> {
    map: TransformationMap<'s>,
    molecule: Molecule,
}

struct MoleculeSet<'l, 's> {
    lab: &'l ChemLab<'s>,
    set: HashSet<Molecule, FnvBuildHasher>,
}

impl MoleculeSet<'_, '_> {
    fn len(&self) -> usize {
        self.set.len()
    }
}

impl ChemLab<'_> {
    fn calibrate(&self) -> MoleculeSet {
        let set = self
            .molecule
            .atoms()
            .flat_map(|atom| {
                self.map
                    .transformations(&atom)
                    .map(move |new_atom| self.molecule.transform(atom, new_atom))
            })
            .collect();

        MoleculeSet { lab: self, set }
    }

    fn synthesize(&self) -> usize {
        let root = Molecule::E;
        let molecules = FrozenIndexSet::<Molecule, FnvBuildHasher>::default();
        molecules.
        let (tx, rx) = std::sync::mpsc::channel();

        todo!()
    }
}

impl<'s> TryFrom<&'s str> for ChemLab<'s> {
    type Error = Report;

    fn try_from(input: &'s str) -> Result<Self> {
        let mut lines = input.trim().lines().map(|line| line.trim());
        let last_line = lines.next_back().ok_or_eyre("empty input!")?.trim();
        let molecule = Molecule::new(last_line);

        let blank = lines.next_back().ok_or_eyre("expected blank line between transformations and molecules, but found no more lines")?.trim();

        if !blank.is_empty() {
            bail!(
                "Expected empty line between molecule and transformations, but found \"{blank}\""
            );
        }

        let transformations = lines
            .enumerate()
            .map(|(i, line)| {
                line.try_into()
                    .wrap_err_with(|| format!("on line {}", i + 1))
            })
            .collect::<Result<_>>()?;

        Ok(Self {
            map: transformations,
            molecule,
        })
    }
}

#[derive(Debug, PartialEq, Eq, Hash)]
struct Molecule(Cow<'static, str>);

struct MoleculeRef(str);

impl Deref for MoleculeRef {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl MoleculeRef {

    fn atoms(&self) -> Atoms {
            Atoms::new(self)
        }

        fn transform(&self, from: Atom, to: &str) -> Molecule {
            // Double check that this atom came from this molecule.
            debug_assert_eq!(&self[from.idx..from.symbol.len()], from.symbol);

            let mut out = String::with_capacity(self.len() + to.len() - from.symbol.len());

            let before = &self[..from.idx];
            let after = &self[(from.idx + from.symbol.len())..];

            out.push_str(before);
            out.push_str(to);
            out.push_str(after);

            Molecule(Cow::Owned(out))
        }
}

impl Molecule {
    const E: Molecule = Molecule(Cow::Borrowed("e"));

    fn new<S: ToString>(molecule: S) -> Self {
        Self(Cow::Owned(molecule.to_string()))
    }
}

// Safety: `StableDeref` is implemented for `String`, which this struct is a wrapper around.
unsafe impl StableDeref for Molecule {}

impl Deref for Molecule {
    type Target = MoleculeRef;

    fn deref(&self) -> &Self::Target {
        MoleculeRef(self.0)
    }
}

struct Atoms<'m> {
    molecule: &'m str,
    i: usize,
}

impl<'m> Atoms<'m> {
    fn new(molecule: &'m Molecule) -> Self {
        Self {
            molecule: molecule.deref(),
            i: 0,
        }
    }
}

impl<'m> Iterator for Atoms<'m> {
    type Item = Atom<'m>;

    fn next(&mut self) -> Option<Self::Item> {
        // Find the first uppercase character after byte 0, which indicates the start of the next atom.
        let rest = self.molecule.get(1..)?;
        let next_atom_start = rest
            .find(|ch: char| ch.is_ascii_uppercase())
            .unwrap_or(rest.len())
            // Add one to correct for the 1-byte offset.
            + 1;

        // Split off this atom from the rest of the molecule.
        let (atom_symbol, rest) = self.molecule.split_at_checked(next_atom_start)?;
        let atom = Atom {
            symbol: atom_symbol,
            idx: self.i,
        };

        self.molecule = rest;
        self.i += atom_symbol.len();

        Some(atom)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct Atom<'s> {
    /// The chemical symbol for this atom
    symbol: &'s str,

    /// The position of this atom in its parent molecule
    idx: usize,
}

impl Display for Molecule {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

#[derive(Default)]
struct TransformationMap<'s>(HashMap<&'s str, HashSet<&'s str, FnvBuildHasher>, FnvBuildHasher>);
type TransformationIter<'s, 'm> =
    Copied<Flatten<<Option<&'m HashSet<&'s str, FnvBuildHasher>> as IntoIterator>::IntoIter>>;

impl<'s> TransformationMap<'s> {
    fn transformations<'m>(&'m self, atom: &Atom) -> TransformationIter<'s, 'm> {
        self.0.get(atom.symbol).into_iter().flatten().copied()
    }
}

impl<'s> TryFrom<&'s str> for TransformationMap<'s> {
    type Error = Report;

    fn try_from(value: &'s str) -> Result<Self> {
        value.lines().map(|line| line.trim().try_into()).collect()
    }
}

impl<'s> FromIterator<Transformation<'s>> for TransformationMap<'s> {
    fn from_iter<T: IntoIterator<Item = Transformation<'s>>>(iter: T) -> Self {
        let mut map =
            HashMap::<&'s str, HashSet<&'s str, FnvBuildHasher>, FnvBuildHasher>::default();
        for Transformation { from, to } in iter {
            map.entry(from).or_default().insert(to);
        }

        Self(map)
    }
}

struct Transformation<'s> {
    from: &'s str,
    to: &'s str,
}

impl<'s> TryFrom<&'s str> for Transformation<'s> {
    type Error = Report;

    fn try_from(value: &'s str) -> Result<Self> {
        separated_pair(alphanumeric1::<_, ContextError>, " => ", alphanumeric1)
            .map(|(from, to)| Self { from, to })
            .parse(value)
            .map_err(|e| {
                eyre!(
                    "Error parsing {value} to Transformation at index {}",
                    e.offset()
                )
            })
    }
}

#[test]
fn atoms() {
    let molecule = Molecule::new("AbCdEFGhiJ");

    let expected = vec![
        Atom {
            symbol: "Ab",
            idx: 0,
        },
        Atom {
            symbol: "Cd",
            idx: 2,
        },
        Atom {
            symbol: "E",
            idx: 4,
        },
        Atom {
            symbol: "F",
            idx: 5,
        },
        Atom {
            symbol: "Ghi",
            idx: 6,
        },
        Atom {
            symbol: "J",
            idx: 9,
        },
    ];
    let actual: Vec<_> = molecule.atoms().collect();

    assert_eq!(expected, actual);
}
