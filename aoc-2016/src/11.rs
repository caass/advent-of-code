use std::collections::{BTreeSet, btree_set};
use std::ops;
use std::sync::mpsc;

use std::array;
use std::iter::Copied;
use std::iter::Flatten;
use std::slice;

use aoc_common::TryFromStr;
use aoc_common::TryParse;
use dashmap::{DashMap, Entry};
use deranged::RangedUsize;
use eyre::{Report, Result, bail, eyre};
use fnv::FnvBuildHasher;
use itertools::Itertools;
use rayon::prelude::*;
use winnow::ascii::alpha1;
use winnow::combinator::{alt, preceded, separated, terminated};
use winnow::error::ContextError;
use winnow::prelude::*;
use winnow::stream::Accumulate;

use aoc_meta::Problem;

pub const RADIOISOTOPE_THERMOELECTRIC_GENERATORS: Problem =
    Problem::solved(&minimum_steps, &minimum_steps_with_two_more_pairs);

fn minimum_steps_with_two_more_pairs(input: &str) -> Result<usize> {
    let mut column = input.try_parse::<Column>()?;
    for item in [
        Item::Generator { element: "elerium" },
        Item::Microchip { element: "elerium" },
        Item::Generator {
            element: "dilithium",
        },
        Item::Microchip {
            element: "dilithium",
        },
    ] {
        column.floors[0].0.insert(item);
    }

    column.fewest_steps_to_solve()
}

fn minimum_steps(input: &str) -> Result<usize> {
    input.try_parse().and_then(Column::fewest_steps_to_solve)
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct Column<'a> {
    floors: [Floor<'a>; 4],
    elevator: RangedUsize<0, 3>,
}

impl<'a> Column<'a> {
    fn fewest_steps_to_solve(self) -> Result<usize> {
        let target_state = self.target();

        let (tx, rx) = mpsc::channel();
        let visited = DashMap::with_hasher(FnvBuildHasher::default());

        tx.send((self.clone(), 0usize))
            .expect("rx to still be open");
        visited.insert(self, 0usize);

        while let Ok((current_state, steps)) = rx.recv() {
            if current_state == target_state {
                return Ok(steps);
            }

            current_state.next_valid_states().for_each(|next| {
                if let Entry::Vacant(vac) = visited.entry(next.clone()) {
                    vac.insert(steps + 1);
                    tx.send((next.clone(), steps + 1))
                        .expect("rx to still be open");
                }
            })
        }

        Err(eyre!("no solution"))
    }
    fn target(&self) -> Column<'a> {
        Column {
            elevator: RangedUsize::new_static::<3>(),
            floors: array::from_fn(|i| {
                if i == 3 {
                    self.items().collect()
                } else {
                    Floor::default()
                }
            }),
        }
    }

    fn new() -> Column<'a> {
        Self {
            floors: array::from_fn(|_| Floor::default()),
            elevator: RangedUsize::new_static::<0>(),
        }
    }

    fn items(&self) -> Copied<Flatten<slice::Iter<'_, Floor<'a>>>> {
        self.floors.iter().flatten().copied()
    }

    fn next_valid_states(&self) -> impl ParallelIterator<Item = Column<'a>> {
        self.elevator
            .checked_add(1)
            .into_par_iter()
            .chain(self.elevator.checked_sub(1))
            .flat_map(|to| self.valid_swaps_to_floor(to))
    }

    // returns a parallel iterator over the states this column can be in when moving 0, 1, or 2
    // items from the floor the elevator is currently on to a given floor
    fn valid_swaps_to_floor(
        &self,
        to: RangedUsize<0, 3>,
    ) -> impl ParallelIterator<Item = Column<'a>> {
        let from = self.elevator;

        let from_floor = &self.floors[from.get()];
        let to_floor = &self.floors[to.get()];

        self.tuple_combinations_on_current_floor()
            .map(|(a, b)| (Some(a), Some(b)))
            .chain(self.items_on_current_floor().map(|it| (Some(it), None)))
            .filter_map(move |(maybe_a, maybe_b)| {
                let mut new_from = from_floor.clone();
                let mut new_to = to_floor.clone();

                if let Some(a) = maybe_a {
                    new_from.remove(&a);
                    new_to.insert(a);
                }

                if let Some(b) = maybe_b {
                    new_from.remove(&b);
                    new_to.insert(b);
                };

                (new_from.is_valid() && new_to.is_valid()).then(|| {
                    let mut new_col = self.clone();

                    new_col.floors[from.get()] = new_from;
                    new_col.floors[to.get()] = new_to;
                    new_col.elevator = to;

                    new_col
                })
            })
    }

    fn tuple_combinations_on_current_floor(
        &self,
    ) -> impl ParallelIterator<Item = (Item<'a>, Item<'a>)> {
        self.floors[self.elevator.get()]
            .iter()
            .copied()
            .tuple_combinations()
            .par_bridge()
    }

    fn items_on_current_floor(&self) -> impl ParallelIterator<Item = Item<'a>> {
        self.floors[self.elevator.get()].par_iter().copied()
    }
}

impl Default for Column<'_> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a> TryFromStr<'a> for Column<'a> {
    type Err = Report;

    fn try_from_str(s: &'a str) -> Result<Self, Self::Err> {
        let mut col = Column::default();

        for (i, line) in s.lines().enumerate() {
            if i > 3 {
                bail!("too many floors ({}) in column!", i + 1);
            }

            let Some((prefix, contains)) = line.split_once(" contains ") else {
                bail!("expected to find string \"contains\" in {line}")
            };

            match (i, prefix) {
                (0, "The first floor")
                | (1, "The second floor")
                | (2, "The third floor")
                | (3, "The fourth floor") => Ok(()),
                _ => Err(eyre!("unexpected prefix \"{prefix}\" on floor {i}")),
            }?;

            col.floors[i] = parse_contains(contains)?;
        }

        Ok(col)
    }
}

fn parse_contains(input: &str) -> eyre::Result<Floor<'_>> {
    terminated(
        alt((
            "nothing relevant".map(|_| Floor::default()),
            preceded(
                "a ",
                separated(
                    1..,
                    alt((
                        terminated(alpha1::<_, ContextError>, "-compatible microchip")
                            .map(|elem| Item::Microchip { element: elem }),
                        terminated(alpha1, " generator")
                            .map(|elem| Item::Generator { element: elem }),
                    )),
                    alt((", and a ", ", a ", " and a ")),
                ),
            ),
        )),
        '.',
    )
    .parse(input)
    .map_err(|e| eyre!("{e:#?}"))
}

#[cfg(test)]
mod test {
    use aoc_common::TryParse;

    use super::*;

    #[test]
    fn from_str() {
        let input = "The first floor contains a hydrogen-compatible microchip and a lithium-compatible microchip.
The second floor contains a hydrogen generator.
The third floor contains a lithium generator.
The fourth floor contains nothing relevant.";

        let col = input.try_parse::<Column>().unwrap();
        assert_eq!(
            col,
            Column {
                elevator: RangedUsize::new_static::<0>(),
                floors: [
                    Floor::from_iter(vec![
                        Item::Microchip {
                            element: "hydrogen"
                        },
                        Item::Microchip { element: "lithium" }
                    ]),
                    Floor::from_iter(vec![Item::Generator {
                        element: "hydrogen"
                    }]),
                    Floor::from_iter(vec![Item::Generator { element: "lithium" }]),
                    Floor::default()
                ]
            }
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
enum Item<'a> {
    Microchip { element: &'a str },
    Generator { element: &'a str },
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
struct Floor<'a>(BTreeSet<Item<'a>>);

impl<'a> FromIterator<Item<'a>> for Floor<'a> {
    fn from_iter<T: IntoIterator<Item = Item<'a>>>(iter: T) -> Self {
        Self(BTreeSet::from_iter(iter))
    }
}

impl<'a> Accumulate<Item<'a>> for Floor<'a> {
    fn initial(_capacity: Option<usize>) -> Self {
        Self(BTreeSet::new())
    }

    fn accumulate(&mut self, acc: Item<'a>) {
        self.0.insert(acc);
    }
}

impl<'a> Floor<'a> {
    fn is_valid(&self) -> bool {
        self.0
            .iter()
            .all(|item| matches!(item, Item::Microchip { .. }))
            || self.0.iter().all(|&item| match item {
                Item::Generator { .. } => true,
                Item::Microchip { element } => self.0.contains(&Item::Generator { element }),
            })
    }
}

impl<'a> ops::Deref for Floor<'a> {
    type Target = BTreeSet<Item<'a>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl ops::DerefMut for Floor<'_> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<'a, 'b> IntoIterator for &'a Floor<'b> {
    type Item = &'a Item<'b>;

    type IntoIter = btree_set::Iter<'a, Item<'b>>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}
