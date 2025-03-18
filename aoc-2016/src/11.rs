use std::borrow::Cow;
use std::collections::{BTreeSet, btree_set};
use std::iter;

use either::Either;
use eyre::{OptionExt, Report, Result, bail, eyre};
use fnv::FnvHashSet;
use itertools::Itertools;
use rayon::prelude::*;
use winnow::ascii::alpha1;
use winnow::combinator::{alt, preceded, separated, seq, terminated};
use winnow::prelude::*;

use aoc_common::{TryFromStr, TryParse};
use aoc_meta::Problem;

pub const RADIOISOTOPE_THERMOELECTRIC_GENERATORS: Problem = Problem::partially_solved(&|input| {
    let column: Column = input.try_parse()?;

    Ok::<_, Report>(format!("{:#?}", column))
});

#[derive(Debug, Clone, Default, PartialEq, Eq, PartialOrd, Ord)]
struct Floor<'s>(BTreeSet<Item<'s>>);

type Items<'a, 's> = iter::Copied<btree_set::Iter<'a, Item<'s>>>;
type ItemPairs<'a, 's> = itertools::TupleCombinations<Items<'a, 's>, (Item<'s>, Item<'s>)>;

impl<'s> Floor<'s> {
    #[inline]
    fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Returns `true` if the given floor is valid, which is true if:
    /// - the floor does not contain any generators
    /// - all microchips have a corresponding generator
    #[inline]
    fn is_valid(&self) -> bool {
        self.0.iter().all(Item::is_microchip)
            || self
                .0
                .iter()
                .filter_map(Item::as_microchip)
                .all(|Microchip { element }| self.has_generator_for(element))
    }

    fn remove(&mut self, item: &Item<'s>) -> bool {
        self.0.remove(item)
    }

    fn remove_both(&mut self, a: &Item<'s>, b: &Item<'s>) -> bool {
        self.remove(a) && self.remove(b)
    }

    fn remove_either(&mut self, items: Either<&Item<'s>, &(Item<'s>, Item<'s>)>) -> bool {
        match items {
            Either::Left(item) => self.remove(item),
            Either::Right((a, b)) => self.remove_both(a, b),
        }
    }

    fn is_valid_with(&self, item: &Item<'s>) -> bool {
        self.0.iter().chain([item]).all(Item::is_microchip)
            || self
                .0
                .iter()
                .chain([item])
                .filter_map(Item::as_microchip)
                .all(|Microchip { element }| {
                    self.has_generator_for(element) || item.is_generator_for(element)
                })
    }

    fn is_valid_with_both(&self, a: &Item<'s>, b: &Item<'s>) -> bool {
        self.0.iter().chain([a, b]).all(Item::is_microchip)
            || self
                .0
                .iter()
                .chain([a, b])
                .filter_map(Item::as_microchip)
                .all(|Microchip { element }| {
                    self.has_generator_for(element)
                        || a.is_generator_for(element)
                        || b.is_generator_for(element)
                })
    }

    fn is_valid_with_either(&self, items: Either<&Item<'s>, &(Item<'s>, Item<'s>)>) -> bool {
        match items {
            Either::Left(item) => self.is_valid_with(item),
            Either::Right((a, b)) => self.is_valid_with_both(a, b),
        }
    }

    fn is_valid_without(&self, removed: &Item) -> bool {
        self.0
            .iter()
            .filter(|item| *item != removed)
            .all(Item::is_microchip)
            || self
                .0
                .iter()
                .filter(|item| *item != removed)
                .filter_map(Item::as_microchip)
                .all(|Microchip { element }| self.has_generator_for(element))
    }

    fn is_valid_without_both(&self, a: &Item, b: &Item) -> bool {
        self.0
            .iter()
            .filter(|item| *item != a && *item != b)
            .all(Item::is_microchip)
            || self
                .0
                .iter()
                .filter(|item| *item != a && *item != b)
                .filter_map(Item::as_microchip)
                .all(|Microchip { element }| self.has_generator_for(element))
    }

    fn is_valid_without_either(&self, items: Either<&Item<'s>, &(Item<'s>, Item<'s>)>) -> bool {
        match items {
            Either::Left(removed) => self.is_valid_without(removed),
            Either::Right((a, b)) => self.is_valid_without_both(a, b),
        }
    }

    fn insert(&mut self, item: Item<'s>) -> bool {
        self.0.insert(item)
    }

    fn insert_both(&mut self, a: Item<'s>, b: Item<'s>) -> bool {
        self.0.insert(a) && self.0.insert(b)
    }

    fn insert_either(&mut self, items: Either<Item<'s>, (Item<'s>, Item<'s>)>) -> bool {
        match items {
            Either::Left(item) => self.insert(item),
            Either::Right((a, b)) => self.insert_both(a, b),
        }
    }

    fn has_generator_for(&self, element: &str) -> bool {
        self.0.contains(&Item::Generator(Generator { element }))
    }

    fn items(&self) -> Items<'_, 's> {
        self.0.iter().copied()
    }

    fn candidates_for_removal(&self) -> CandidatesForRemoval<'_, 's> {
        let singletons = self.items();
        let pairs = self.item_tuples();

        CandidatesForRemoval {
            singletons,
            pairs,
            floor: self,
        }
    }

    fn next_valid_states<'a, 'b>(&'a self, other: &'b Floor<'s>) -> NextValidStates<'a, 'b, 's> {
        NextValidStates {
            this: self,
            other,
            candidates: self.candidates_for_removal(),
        }
    }

    fn item_tuples(&self) -> ItemPairs<'_, 's> {
        self.0.iter().copied().tuple_combinations()
    }
}

#[derive(Debug)]
struct NextValidStates<'a, 'b, 's> {
    this: &'a Floor<'s>,
    other: &'b Floor<'s>,
    candidates: CandidatesForRemoval<'a, 's>,
}

impl<'s> Iterator for NextValidStates<'_, '_, 's> {
    type Item = (Floor<'s>, Floor<'s>);

    fn next(&mut self) -> Option<Self::Item> {
        let next = self.candidates.next()?;
        self.other
            .is_valid_with_either(next.as_ref())
            .then(|| {
                let mut this = self.this.clone();
                let mut other = self.other.clone();
                this.remove_either(next.as_ref());
                other.insert_either(next);

                (this, other)
            })
            .or_else(|| self.next())
    }
}

#[derive(Debug)]
struct CandidatesForRemoval<'a, 's> {
    singletons: Items<'a, 's>,
    pairs: ItemPairs<'a, 's>,
    floor: &'a Floor<'s>,
}

impl<'s> Iterator for CandidatesForRemoval<'_, 's> {
    type Item = Either<Item<'s>, (Item<'s>, Item<'s>)>;

    fn next(&mut self) -> Option<Self::Item> {
        let next = self
            .singletons
            .next()
            .map(Either::Left)
            .or_else(|| self.pairs.next().map(Either::Right))?;

        self.floor
            .is_valid_without_either(next.as_ref())
            .then_some(next)
            .or_else(|| self.next())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct Column<'s> {
    floors: [Floor<'s>; 4],
    elevator: Level,
}

impl<'s> Column<'s> {
    fn all_on_fourth_floor(&self) -> bool {
        self.floors[0].is_empty() && self.floors[1].is_empty() && self.floors[2].is_empty()
    }

    fn next_valid_states(&self) -> Box<dyn Iterator<Item = Column<'s>> + '_> {
        match self.elevator {
            Level::First => Box::new(self.floors[0].next_valid_states(&self.floors[1]).map(
                |(first, second)| Column {
                    floors: [
                        first,
                        second,
                        self.floors[2].clone(),
                        self.floors[3].clone(),
                    ],
                    elevator: Level::Second,
                },
            )),
            Level::Second => {
                let down =
                    self.floors[1]
                        .next_valid_states(&self.floors[0])
                        .map(|(second, first)| Column {
                            floors: [
                                first,
                                second,
                                self.floors[2].clone(),
                                self.floors[3].clone(),
                            ],
                            elevator: Level::First,
                        });

                let up =
                    self.floors[1]
                        .next_valid_states(&self.floors[2])
                        .map(|(second, third)| Column {
                            floors: [
                                self.floors[0].clone(),
                                second,
                                third,
                                self.floors[3].clone(),
                            ],
                            elevator: Level::First,
                        });

                Box::new(down.chain(up))
            }
            Level::Third => todo!(),
            Level::Fourth => todo!(),
        }
    }
}

impl<'s> TryFromStr<'s> for Column<'s> {
    type Err = Report;

    fn try_from_str(s: &'s str) -> Result<Self, Self::Err> {
        s.lines().map(FloorDescription::try_from_str).collect()
    }
}

impl<'s> FromIterator<FloorDescription<'s>> for Column<'s> {
    fn from_iter<T: IntoIterator<Item = FloorDescription<'s>>>(iter: T) -> Self {
        let mut floors = [
            Floor::default(),
            Floor::default(),
            Floor::default(),
            Floor::default(),
        ];

        for FloorDescription { level, items } in iter {
            floors[level.as_usize()] = items;
        }

        Self {
            floors,
            elevator: Level::First,
        }
    }
}

#[derive(Debug, Hash, PartialEq, Eq, Clone, Copy, PartialOrd, Ord)]
struct Microchip<'s> {
    element: &'s str,
}

#[derive(Debug, Hash, PartialEq, Eq, Clone, Copy, PartialOrd, Ord)]
struct Generator<'s> {
    element: &'s str,
}

#[derive(Debug, Hash, PartialEq, Eq, Clone, Copy, PartialOrd, Ord)]
enum Item<'s> {
    Microchip(Microchip<'s>),
    Generator(Generator<'s>),
}

impl<'s> Item<'s> {
    fn is_microchip(&self) -> bool {
        matches!(self, Item::Microchip(_))
    }

    fn as_microchip(&self) -> Option<Microchip<'s>> {
        match self {
            Item::Microchip(microchip) => Some(*microchip),
            Item::Generator(_) => None,
        }
    }

    fn is_generator_for(&self, element: &str) -> bool {
        matches!(self, &Item::Generator(Generator { element: element2 }) if element == element2)
    }
}

#[derive(Debug)]
struct FloorDescription<'s> {
    level: Level,
    items: Floor<'s>,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u8)]
enum Level {
    #[default]
    First = 0,
    Second,
    Third,
    Fourth,
}

impl Level {
    fn as_usize(&self) -> usize {
        (*self) as u8 as usize
    }
}

impl<'s> TryFromStr<'s> for FloorDescription<'s> {
    type Err = Report;

    fn try_from_str(s: &'s str) -> Result<Self, Self::Err> {
        parse_floor.parse(s).map_err(|e| {
            let context: String = Itertools::intersperse(
                e.inner().context().map(ToString::to_string).map(Cow::Owned),
                Cow::Borrowed(": "),
            )
            .collect();

            eyre!("{context}: {}", e.input())
        })
    }
}

fn parse_floor<'s>(input: &mut &'s str) -> ModalResult<FloorDescription<'s>> {
    seq! { FloorDescription {
        _: "The ",
        level: alt((
            "first".map(|_| Level::First),
            "second".map(|_| Level::Second),
            "third".map(|_| Level::Third),
            "fourth".map(|_| Level::Fourth)
        )),
        _: " floor contains ",
        items: alt((
            "nothing relevant.".map(|_| BTreeSet::default()),
            terminated(
                separated(
                    1..,
                    preceded(
                        "a ",
                        alt((
                            terminated(alpha1, "-compatible microchip")
                                .map(|element| Microchip { element })
                                .map(Item::Microchip),
                            terminated(alpha1, " generator")
                                .map(|element| Generator { element })
                                .map(Item::Generator),
                        )),
                    ),
                    alt((", and ", ", ", " and ")),
                ),
                '.',
            )
        )).map(Floor)
    }}
    .parse_next(input)
}
