use std::str::FromStr;

use eyre::{OptionExt, Result};
use itertools::Itertools;
use rayon::prelude::*;

use crate::meta::Problem;

/// https://adventofcode.com/2015/day/24
pub const IT_HANGS_IN_THE_BALANCE: Problem = Problem::solved(
    &|input| {
        input
            .parse::<PackingList>()?
            .pack(3)
            .map(|list| list.quantum_entanglement())
            .ok_or_eyre("impossible to pack sleigh")
    },
    &|input| {
        input
            .parse::<PackingList>()?
            .pack(4)
            .map(|list| list.quantum_entanglement())
            .ok_or_eyre("impossible to pack sleigh")
    },
);

#[derive(Debug, Clone)]
struct PackingList {
    packages: Vec<Package>,
}

impl PackingList {
    fn pack(&self, num_compartments: usize) -> Option<PackingList> {
        if self.weight() % num_compartments != 0 {
            return None;
        };

        let target_weight = self.weight() / num_compartments;

        (1..self.packages.len())
            .into_par_iter()
            .find_map_first(|k| {
                let lists_of_len_k = self
                    .packages
                    .iter()
                    .copied()
                    .combinations(k)
                    .map(|packages| PackingList { packages });

                lists_of_len_k
                    .filter(|list| list.weight() == target_weight)
                    .min_by_key(PackingList::quantum_entanglement)
            })
    }

    fn weight(&self) -> usize {
        self.packages
            .iter()
            .copied()
            .map(|Package { weight }| usize::from(weight))
            .sum()
    }

    fn quantum_entanglement(&self) -> u64 {
        self.packages
            .iter()
            .copied()
            .map(|Package { weight }| u64::from(weight))
            .reduce(|a, b| a.saturating_mul(b))
            .unwrap_or_default()
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
    weight: u8,
}

impl FromStr for Package {
    type Err = <usize as FromStr>::Err;

    #[inline(always)]
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        s.parse().map(|weight| Package { weight })
    }
}
