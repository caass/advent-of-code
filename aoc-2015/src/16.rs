use std::collections::HashMap;
use std::str::FromStr;

use eyre::{Report, Result, bail, eyre};
use rayon::prelude::*;

use nohash_hasher::BuildNoHashHasher;
use winnow::ascii::alpha1;
use winnow::combinator::{delimited, separated, separated_pair};
use winnow::{ascii::dec_uint, prelude::*};

use aoc_meta::Problem;

/// <https://adventofcode.com/2015/day/16>
pub const AUNT_SUE: Problem = Problem::solved(
    &|input| input.parse::<Aunts>()?.find(|sue| sue.matches(&READOUT)),
    &|input| {
        input
            .parse::<Aunts>()?
            .find(|sue| sue.matches_range(&READOUT))
    },
);

const READOUT: MfcsamReadout = MfcsamReadout {
    children: 3,
    cats: 7,
    samoyeds: 2,
    pomeranians: 3,
    akitas: 0,
    vizslas: 0,
    goldfish: 5,
    trees: 3,
    cars: 2,
    perfumes: 1,
};

struct Aunts(HashMap<u16, Sue, BuildNoHashHasher<u16>>);

impl Aunts {
    fn find<F: Send + Sync + Fn(&Sue) -> bool>(&self, f: F) -> Result<u16> {
        match *self
            .0
            .par_iter()
            .filter(|(_, sue)| f(sue))
            .map(|(n, _)| n)
            .copied()
            .collect::<Vec<_>>()
            .as_slice()
        {
            [sue] => Ok(sue),
            [] => bail!("No sues matched filter"),
            _ => bail!("Multiple sues matched filter"),
        }
    }
}

impl FromParallelIterator<(u16, Sue)> for Aunts {
    fn from_par_iter<I>(par_iter: I) -> Self
    where
        I: IntoParallelIterator<Item = (u16, Sue)>,
    {
        Self(HashMap::from_par_iter(par_iter))
    }
}

impl FromStr for Aunts {
    type Err = Report;

    fn from_str(s: &str) -> Result<Self> {
        s.par_lines()
            .map(|line| parse_ticker_tape(line.trim()))
            .collect()
    }
}

#[derive(Debug, Clone, Copy)]
struct MfcsamReadout {
    children: u8,
    cats: u8,
    samoyeds: u8,
    pomeranians: u8,
    akitas: u8,
    vizslas: u8,
    goldfish: u8,
    trees: u8,
    cars: u8,
    perfumes: u8,
}

#[derive(Debug, Clone, Copy)]
struct Sue {
    children: Option<u8>,
    cats: Option<u8>,
    samoyeds: Option<u8>,
    pomeranians: Option<u8>,
    akitas: Option<u8>,
    vizslas: Option<u8>,
    goldfish: Option<u8>,
    trees: Option<u8>,
    cars: Option<u8>,
    perfumes: Option<u8>,
}

impl Sue {
    fn matches(&self, readout: &MfcsamReadout) -> bool {
        macro_rules! equals_if_present {
            ($($field:ident),+) => {
                $(self.$field.map(|n| n == readout.$field).unwrap_or(true) && )+ true
            };
        }

        equals_if_present!(
            children,
            cats,
            samoyeds,
            pomeranians,
            akitas,
            vizslas,
            goldfish,
            trees,
            cars,
            perfumes
        )
    }

    fn matches_range(&self, readout: &MfcsamReadout) -> bool {
        self.children.is_none_or(|n| n == readout.children)
            && self.samoyeds.is_none_or(|n| n == readout.samoyeds)
            && self.akitas.is_none_or(|n| n == readout.akitas)
            && self.vizslas.is_none_or(|n| n == readout.vizslas)
            && self.cars.is_none_or(|n| n == readout.cars)
            && self.perfumes.is_none_or(|n| n == readout.perfumes)

            // compensate for nuclear decay of cat dander and tree pollen
            && self.trees.is_none_or(|n| n > readout.trees)
            && self.cats.is_none_or(|n| n > readout.cats)

            // compensate for modial interaction of magnetoreluctance
            && self.pomeranians.is_none_or(|n| n < readout.pomeranians)
            && self.goldfish.is_none_or(|n| n < readout.goldfish)
    }
}

impl FromIterator<Compound> for Sue {
    fn from_iter<T: IntoIterator<Item = Compound>>(iter: T) -> Self {
        let mut children = None;
        let mut kitties = None;
        let mut samoyeds = None;
        let mut pomeranians = None;
        let mut akitas = None;
        let mut vizslas = None;
        let mut goldfish = None;
        let mut trees = None;
        let mut cars = None;
        let mut perfumes = None;

        for Compound { kind, count } in iter {
            let opt = match kind {
                CompoundKind::Children => &mut children,
                CompoundKind::Cats => &mut kitties,
                CompoundKind::Samoyeds => &mut samoyeds,
                CompoundKind::Pomeranians => &mut pomeranians,
                CompoundKind::Akitas => &mut akitas,
                CompoundKind::Vizslas => &mut vizslas,
                CompoundKind::Goldfish => &mut goldfish,
                CompoundKind::Trees => &mut trees,
                CompoundKind::Cars => &mut cars,
                CompoundKind::Perfumes => &mut perfumes,
            };
            debug_assert!(opt.is_none());
            *opt = Some(count);
        }

        Sue {
            children,
            cats: kitties,
            samoyeds,
            pomeranians,
            akitas,
            vizslas,
            goldfish,
            trees,
            cars,
            perfumes,
        }
    }
}

#[derive(Debug)]
struct Compound {
    kind: CompoundKind,
    count: u8,
}

#[derive(Debug)]
enum CompoundKind {
    Children,
    Cats,
    Samoyeds,
    Pomeranians,
    Akitas,
    Vizslas,
    Goldfish,
    Trees,
    Cars,
    Perfumes,
}

impl FromStr for CompoundKind {
    type Err = Report;

    fn from_str(s: &str) -> Result<Self> {
        match s {
            "children" => Ok(CompoundKind::Children),
            "cats" => Ok(CompoundKind::Cats),
            "samoyeds" => Ok(CompoundKind::Samoyeds),
            "pomeranians" => Ok(CompoundKind::Pomeranians),
            "akitas" => Ok(CompoundKind::Akitas),
            "vizslas" => Ok(CompoundKind::Vizslas),
            "goldfish" => Ok(CompoundKind::Goldfish),
            "trees" => Ok(CompoundKind::Trees),
            "cars" => Ok(CompoundKind::Cars),
            "perfumes" => Ok(CompoundKind::Perfumes),
            _ => Err(eyre!("Unknown compound kind {s}")),
        }
    }
}

fn parse_ticker_tape(line: &str) -> Result<(u16, Sue)> {
    let (number, compounds): (_, Vec<_>) =
        separated_pair(parse_sue_number, ' ', separated(1.., parse_compound, ", "))
            .parse(line)
            .map_err(|e| eyre!("error parsing \"{}\" at index {}", e.input(), e.offset()))?;
    let sue = Sue::from_iter(compounds);
    Ok((number, sue))
}

fn parse_compound(input: &mut &str) -> ModalResult<Compound> {
    separated_pair(alpha1.parse_to(), ": ", dec_uint)
        .map(|(kind, count)| Compound { kind, count })
        .parse_next(input)
}

fn parse_sue_number(input: &mut &str) -> ModalResult<u16> {
    delimited("Sue ", dec_uint, ':').parse_next(input)
}
