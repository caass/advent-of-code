use std::{cmp::Ordering, str::FromStr};

use eyre::{OptionExt, Result};
use rayon::prelude::*;

use crate::meta::Problem;

/// https://adventofcode.com/2015/day/17
pub const NO_SUCH_THING_AS_TOO_MUCH: Problem = Problem::solved(
    &|input| {
        input
            .parse::<ContainerCollection>()?
            .combinations_that_hold(LITERS_OF_EGGNOG)
            .map(|c| c.len())
            .ok_or_eyre("No combinations of containers can hold that much eggnog!")
    },
    &|input| {
        input
            .parse::<ContainerCollection>()?
            .combinations_that_hold(LITERS_OF_EGGNOG)
            .map(|c| c.num_smallest_combinations())
            .ok_or_eyre("No combinations of containers can hold that much eggnog!")
    },
);

const LITERS_OF_EGGNOG: u8 = 150;

/// A set of all the container combinations that can hold a given volume of eggnog.
#[derive(Debug, PartialEq)]
struct ContainerCombinationSet {
    combinations: Vec<ContainerCombination>,
}

impl ContainerCombinationSet {
    #[cfg(test)]
    fn new(mut combinations: Vec<ContainerCombination>) -> Self {
        combinations.par_sort_unstable();
        Self { combinations }
    }

    /// Get the number of combinations in this set.
    #[inline]
    fn len(&self) -> usize {
        self.combinations.len()
    }

    /// Return the number of combinations that use the minimum number of containers
    fn num_smallest_combinations(&self) -> usize {
        let mut n = usize::MAX;
        let mut k = 0;

        for combination in self.combinations.iter() {
            let m = combination.len();

            match m.cmp(&n) {
                Ordering::Less => {
                    // This combination uses less containers
                    n = m;
                    k = 1;
                }
                Ordering::Equal => {
                    // This is another combination with the smallest number of containers
                    k += 1;
                }
                Ordering::Greater => {
                    // This combination uses more than the minimum number
                    continue;
                }
            }
        }

        k
    }

    /// Insert the given container in all the `ContainerCombination`s in `self`
    fn insert(&mut self, container: Container) {
        self.combinations
            .par_iter_mut()
            .for_each(|combination| combination.add(container));
    }

    /// Construct a set of container combinations that contains only a single combination,
    /// where that combination contains only a single container.
    fn single(container: Container) -> Self {
        Self {
            combinations: vec![ContainerCombination::single(container)],
        }
    }
}

impl ParallelExtend<ContainerCombination> for ContainerCombinationSet {
    fn par_extend<I>(&mut self, par_iter: I)
    where
        I: IntoParallelIterator<Item = ContainerCombination>,
    {
        self.combinations.par_extend(par_iter);
        self.combinations.par_sort_unstable();
        self.combinations.dedup();
    }
}

impl FromParallelIterator<ContainerCombination> for ContainerCombinationSet {
    fn from_par_iter<I>(par_iter: I) -> Self
    where
        I: IntoParallelIterator<Item = ContainerCombination>,
    {
        let mut combinations = Vec::from_par_iter(par_iter);
        combinations.par_sort_unstable();
        combinations.dedup();
        Self { combinations }
    }
}

impl FromIterator<ContainerCombination> for ContainerCombinationSet {
    fn from_iter<T: IntoIterator<Item = ContainerCombination>>(iter: T) -> Self {
        let mut combinations = Vec::from_iter(iter);
        combinations.par_sort_unstable();
        combinations.dedup();
        Self { combinations }
    }
}

impl IntoParallelIterator for ContainerCombinationSet {
    type Iter = rayon::vec::IntoIter<ContainerCombination>;

    type Item = ContainerCombination;

    fn into_par_iter(self) -> Self::Iter {
        self.combinations.into_par_iter()
    }
}

/// A particular combination of containers that can hold a given volume of eggnog.
#[derive(Debug, Default, Hash, PartialEq, Eq, PartialOrd, Ord)]
struct ContainerCombination {
    containers: Vec<Container>,
}

impl ContainerCombination {
    /// Add a container to this combination
    fn add(&mut self, container: Container) {
        match self.containers.binary_search(&container) {
            Err(i) | Ok(i) => self.containers.insert(i, container),
        }
    }

    #[cfg(test)]
    fn new(mut containers: Vec<Container>) -> Self {
        containers.sort_unstable();
        Self { containers }
    }

    /// Construct a combination of containers consisting of a single container.
    #[inline]
    fn single(container: Container) -> Self {
        Self {
            containers: vec![container],
        }
    }

    #[inline]
    fn len(&self) -> usize {
        self.containers.len()
    }
}

impl FromParallelIterator<Container> for ContainerCombination {
    fn from_par_iter<I>(par_iter: I) -> Self
    where
        I: IntoParallelIterator<Item = Container>,
    {
        let mut containers = Vec::from_par_iter(par_iter);
        containers.sort_unstable();
        Self { containers }
    }
}

impl FromIterator<Container> for ContainerCombination {
    fn from_iter<T: IntoIterator<Item = Container>>(iter: T) -> Self {
        let mut containers = Vec::from_iter(iter);
        containers.sort_unstable();
        Self { containers }
    }
}

/// A collection of containers, i.e. your tupperware drawer. May contain duplicates.
#[derive(Debug)]
struct ContainerCollection {
    containers: Vec<Container>,
}

impl ContainerCollection {
    #[cfg(test)]
    fn new(containers: Vec<Container>) -> Self {
        Self { containers }
    }

    #[inline]
    fn containers(&self) -> &[Container] {
        &self.containers
    }

    #[inline]
    fn iter(&self) -> std::slice::Iter<Container> {
        self.containers().iter()
    }

    #[inline]
    fn par_iter(&self) -> rayon::iter::Copied<rayon::slice::Iter<Container>> {
        self.containers().par_iter().copied()
    }

    /// Find the set of combinations of containers that can hold the given volume of eggnog
    fn combinations_that_hold(&self, volume: u8) -> Option<ContainerCombinationSet> {
        self.par_iter()
            .map(|container| {
                match volume.checked_sub(container.volume) {
                    None => {
                        // the container is bigger than the given volume;
                        // we're looking for exact fits.
                        None
                    }
                    Some(0) => {
                        // the container is exactly the given volume;
                        // there's exactly one combination that involves this container,
                        // and it's the combination of "just this container".
                        Some(ContainerCombinationSet::single(container))
                    }
                    Some(new_volume) => {
                        // the container has is smaller than the given volume,
                        // so compute the subsets and tack this onto the end
                        let mut combinations = self
                            .subset(container.id)
                            .combinations_that_hold(new_volume)?;
                        combinations.insert(container);
                        Some(combinations)
                    }
                }
            })
            .reduce(
                || None,
                |a, b| match (a, b) {
                    (None, None) => None,
                    (Some(set), None) | (None, Some(set)) => Some(set),
                    (Some(mut a), Some(b)) => {
                        a.par_extend(b);
                        Some(a)
                    }
                },
            )
    }

    /// Return a subset of this container collection without the container with the given `id`.
    fn subset(&self, id: u8) -> ContainerCollection {
        self.iter()
            .copied()
            .filter(move |container| container.id != id)
            .collect()
    }
}

impl FromStr for ContainerCollection {
    type Err = <usize as FromStr>::Err;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        s.lines().map(|line| line.trim().parse::<u8>()).collect()
    }
}

impl FromIterator<u8> for ContainerCollection {
    fn from_iter<T: IntoIterator<Item = u8>>(iter: T) -> Self {
        let containers = iter
            .into_iter()
            .enumerate()
            .map(|(id, volume)| Container {
                volume,
                id: id as u8,
            })
            .collect();
        // No need to sort, since the ids are assigned in order
        // containers.sort_unstable();

        Self { containers }
    }
}

impl FromIterator<Container> for ContainerCollection {
    fn from_iter<T: IntoIterator<Item = Container>>(iter: T) -> Self {
        let mut containers = Vec::from_iter(iter);
        containers.sort_unstable();
        Self { containers }
    }
}

/// A container capable of holding eggnog
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct Container {
    /// The volume of eggnog this container can hold
    volume: u8,

    /// A random unique ID assigned to this container
    id: u8,
}

#[test]
fn example() {
    use pretty_assertions::assert_eq;

    const TWENTY_LITERS: Container = Container { volume: 20, id: 0 };
    const FIFTEEN_LITERS: Container = Container { volume: 15, id: 1 };
    const TEN_LITERS: Container = Container { volume: 10, id: 2 };
    const FIVE_LITERS_A: Container = Container { volume: 5, id: 3 };
    const FIVE_LITERS_B: Container = Container { volume: 5, id: 4 };

    let containers = ContainerCollection::new(vec![
        TWENTY_LITERS,
        FIFTEEN_LITERS,
        TEN_LITERS,
        FIVE_LITERS_A,
        FIVE_LITERS_B,
    ]);

    let expected = ContainerCombinationSet::new(vec![
        ContainerCombination::new(vec![FIFTEEN_LITERS, TEN_LITERS]),
        ContainerCombination::new(vec![TWENTY_LITERS, FIVE_LITERS_A]),
        ContainerCombination::new(vec![TWENTY_LITERS, FIVE_LITERS_B]),
        ContainerCombination::new(vec![FIFTEEN_LITERS, FIVE_LITERS_A, FIVE_LITERS_B]),
    ]);

    let actual = containers.combinations_that_hold(25).unwrap();
    assert_eq!(expected, actual);
}
