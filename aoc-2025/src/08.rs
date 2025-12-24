use std::collections::{HashMap, hash_map::Entry};
use std::hash::BuildHasherDefault;
use std::str::FromStr;

use eyre::{Report, Result, bail, eyre};
use itertools::Itertools;
use petgraph::{Graph, Undirected, algo::tarjan_scc, graph::NodeIndex};
use rayon::prelude::*;

use aoc_meta::Problem;
use seahash::SeaHasher;

pub const PLAYGROUND: Problem = Problem::partially_solved(&|input| {
    input
        .parse()
        .and_then(Playground::three_largest_circuits_product::<1000>)
});

#[derive(Debug, Clone)]
struct Playground {
    disconnected: Vec<(NodeIndex<u16>, NodeIndex<u16>)>,
    connected: Graph<Coordinate, (), Undirected, u16>,
}

impl Playground {
    fn three_largest_circuits_product<const N: usize>(mut self) -> Result<u64> {
        for _ in 0..N {
            let Some((a, b)) = self.disconnected.pop() else {
                bail!("fewer than {N} possible connections")
            };

            self.connected.add_edge(a, b, ());
        }

        let mut subgraphs = tarjan_scc(&self.connected);
        if subgraphs.len() < 3 {
            bail!("fewer than three subgraphs!");
        };

        subgraphs.par_sort_by_key(|sg| sg.len());

        subgraphs
            .into_iter()
            .rev()
            .take(3)
            .try_fold(1, |a, sg| sg.len().try_into().map(|b: u64| a * b))
            .map_err(|e| eyre!(e))
    }
}

impl FromStr for Playground {
    type Err = Report;
    fn from_str(s: &str) -> Result<Self> {
        let mut node_pairs: Vec<(Coordinate, Coordinate)> = s
            .lines()
            .tuple_combinations()
            .map(|(a, b)| Ok::<_, Report>((a.parse()?, b.parse()?)))
            .try_collect()?;

        node_pairs.par_sort_unstable_by_key(|(a, b)| a.distance_squared(b));

        let mut g = Graph::default();
        let mut cache = HashMap::with_hasher(BuildHasherDefault::<SeaHasher>::default());

        let node_pair_indices = node_pairs
            .into_iter()
            .rev()
            .map(|(a, b)| {
                let i = match cache.entry(a) {
                    Entry::Occupied(occ) => *occ.get(),
                    Entry::Vacant(vac) => *vac.insert(g.add_node(a)),
                };
                let j = match cache.entry(b) {
                    Entry::Occupied(occ) => *occ.get(),
                    Entry::Vacant(vac) => *vac.insert(g.add_node(b)),
                };

                (i, j)
            })
            .collect();

        Ok(Self {
            disconnected: node_pair_indices,
            connected: g,
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct Coordinate {
    x: u32,
    y: u32,
    z: u32,
}

impl Coordinate {
    const fn distance_squared(&self, other: &Coordinate) -> u64 {
        (self.x.abs_diff(other.x) as u64).pow(2)
            + (self.y.abs_diff(other.y) as u64).pow(2)
            + (self.z.abs_diff(other.z) as u64).pow(2)
    }
}

impl FromStr for Coordinate {
    type Err = Report;

    fn from_str(s: &str) -> Result<Self> {
        let Some([x, y, z]) = s.split(',').collect_array() else {
            bail!("unable to split input into coordinates")
        };

        Ok(Coordinate {
            x: x.parse()?,
            y: y.parse()?,
            z: z.parse()?,
        })
    }
}

#[test]
fn example() {
    use pretty_assertions::assert_eq;

    let input = "162,817,812
57,618,57
906,360,560
592,479,940
352,342,300
466,668,158
542,29,236
431,825,988
739,650,466
52,470,668
216,146,977
819,987,18
117,168,530
805,96,715
346,949,466
970,615,88
941,993,340
862,61,35
984,92,344
425,690,689";

    let playground: Playground = input.parse().unwrap();
    let result = playground.three_largest_circuits_product::<10>().unwrap();
    assert_eq!(result, 40);
}
