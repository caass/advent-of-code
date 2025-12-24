use std::cmp::Reverse;
use std::collections::HashMap;
use std::str::FromStr;

use disjoint_sets::UnionFind;
use eyre::{OptionExt, Report, Result, bail};
use itertools::Itertools;
use nohash_hasher::BuildNoHashHasher;

use aoc_meta::Problem;

pub const PLAYGROUND: Problem = Problem::solved(
    &|input| input.parse().and_then(Playground::connect_n::<1000>),
    &|input| input.parse().and_then(Playground::unify),
);

#[derive(Debug, Clone)]
struct Playground {
    circuits: UnionFind<u16>,
    junction_pairs: Vec<(u16, u16)>,
    coords: HashMap<u16, Coordinate, BuildNoHashHasher<u16>>,
}

impl Playground {
    // Returns the product of the size of the three largest circuits after making N connections
    fn connect_n<const N: usize>(mut self) -> Result<u64> {
        for _ in 0..N {
            let Some((a, b)) = self.junction_pairs.pop() else {
                bail!("fewer than {N} possible connections")
            };

            self.circuits.union(a, b);
        }

        let mut circuit_sizes: HashMap<u16, u64, BuildNoHashHasher<u16>> = HashMap::default();

        for k in self.coords.into_keys() {
            let root = self.circuits.find(k);
            *circuit_sizes.entry(root).or_default() += 1;
        }

        Ok(circuit_sizes
            .into_values()
            .sorted_unstable()
            .rev()
            .take(3)
            .product())
    }

    // Returns the product of the X coordinates of the final two junction boxes needed to make a single circuit.
    fn unify(mut self) -> Result<u64> {
        let mut num_independent_circuits = self.coords.len();
        let (mut a, mut b) = (0, 0);

        while num_independent_circuits > 1 {
            (a, b) = self
                .junction_pairs
                .pop()
                .ok_or_eyre("impossible to make a complete circuit")?;

            if self.circuits.union(a, b) {
                num_independent_circuits -= 1;
            }
        }

        u64::checked_mul(self.coords[&a].x.into(), self.coords[&b].x.into())
            .ok_or_eyre("answer would overflow")
    }
}

impl FromStr for Playground {
    type Err = Report;

    fn from_str(s: &str) -> Result<Self> {
        let parsed_coordinates = s
            .lines()
            .map(Coordinate::from_str)
            .collect::<Result<Vec<_>>>()?;

        let mut circuits = UnionFind::new(parsed_coordinates.len());
        let mut coords = HashMap::with_capacity_and_hasher(
            parsed_coordinates.len(),
            BuildNoHashHasher::default(),
        );

        for coord in parsed_coordinates {
            coords.insert(circuits.alloc(), coord);
        }

        let mut junction_pairs = coords.keys().copied().tuple_combinations().collect_vec();

        junction_pairs
            .sort_unstable_by_key(|(a, b)| Reverse(coords[a].distance_squared(&coords[b])));

        Ok(Self {
            circuits,
            junction_pairs,
            coords,
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

    assert_eq!(playground.clone().connect_n::<10>().unwrap(), 40);
    assert_eq!(playground.unify().unwrap(), 25272);
}
