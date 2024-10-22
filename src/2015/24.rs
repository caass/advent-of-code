use std::str::FromStr;

use eyre::{OptionExt, Report, Result};
use itertools::Itertools;
use rayon::prelude::*;

use crate::meta::Problem;

/// https://adventofcode.com/2015/day/24
pub const IT_HANGS_IN_THE_BALANCE: Problem = Problem::partially_solved(&|input| {
    Ok::<_, Report>(
        input
            .parse::<PackingList>()?
            .pack()
            .ok_or_eyre("impossible to pack sleigh")?
            .quantum_entanglement(),
    )
});

struct Sleigh {
    center: PackingList,
    left: PackingList,
    right: PackingList,
}

impl Sleigh {
    fn quantum_entanglement(&self) -> usize {
        self.center
            .packages
            .iter()
            .copied()
            .map(|Package { weight }| weight)
            .product()
    }
}

struct PartialArrangement {
    center: PackingList,
    sides: PackingList,
}

impl PartialArrangement {
    fn is_balanced(&self) -> bool {
        let (center_weight, side_weight) =
            rayon::join(|| self.center.weight(), || self.sides.weight());

        center_weight * 2 == side_weight
    }

    fn try_balance(self) -> Option<Sleigh> {
        if !self.is_balanced() {
            return None;
        }

        let PartialArrangement { center, sides } = self;

        let PartialArrangement {
            center: left,
            sides: right,
        } = sides
            .splits()
            .flatten_iter()
            .find_any(|arrangement| arrangement.is_balanced())?;

        Some(Sleigh {
            center,
            left,
            right,
        })
    }
}

#[derive(Debug, Clone)]
struct PackingList {
    packages: Vec<Package>,
}

impl PackingList {
    fn pack(&self) -> Option<Sleigh> {
        self.splits().find_map_first(|arrangements| {
            arrangements
                .filter_map(PartialArrangement::try_balance)
                .min_by_key(Sleigh::quantum_entanglement)
        })
    }
    fn splits(
        &self,
    ) -> impl ParallelIterator<Item = impl Iterator<Item = PartialArrangement> + '_> + '_ {
        let k = self.packages.len().div_ceil(2);
        (0..k).into_par_iter().map(|k| {
            self.packages
                .iter()
                .copied()
                .combinations(k)
                .map(|packages| {
                    let center = PackingList { packages };
                    let sides = self.without(&center);

                    PartialArrangement { center, sides }
                })
        })
    }

    #[inline(always)]
    fn weight(&self) -> usize {
        self.packages
            .iter()
            .copied()
            .map(|Package { weight }| weight)
            .sum()
    }

    fn without(&self, center: &PackingList) -> PackingList {
        let mut sides = self.clone();
        sides
            .packages
            .retain(|package| !center.packages.contains(package));
        sides
    }
}

impl FromIterator<Package> for PackingList {
    fn from_iter<T: IntoIterator<Item = Package>>(iter: T) -> Self {
        let mut packages = Vec::from_iter(iter);
        packages.par_sort_unstable();
        Self { packages }
    }
}

impl FromStr for PackingList {
    type Err = <Package as FromStr>::Err;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        s.trim().lines().map(|line| line.trim().parse()).collect()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct Package {
    weight: usize,
}

impl FromStr for Package {
    type Err = <usize as FromStr>::Err;

    #[inline(always)]
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        s.parse().map(|weight| Package { weight })
    }
}
