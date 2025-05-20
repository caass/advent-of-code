// use std::collections::BTreeSet;

// use either::Either;
// use eyre::{OptionExt, Report, Result, bail, eyre};
// use rayon::prelude::*;

// use aoc_common::{TryFromStr, TryParse};
use aoc_meta::Problem;

pub const RADIOISOTOPE_THERMOELECTRIC_GENERATORS: Problem = Problem::unsolved();

// #[derive(Debug, Clone, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
// struct Floor<'s>(BTreeSet<Item<'s>>);

// impl<'s> Floor<'s> {
//     #[inline]
//     fn is_empty(&self) -> bool {
//         self.0.is_empty()
//     }

//     /// Returns `true` if the given floor is valid, which is true if:
//     /// - the floor does not contain any generators
//     /// - all microchips have a corresponding generator
//     #[inline]
//     fn is_valid(&self) -> bool {
//         self.0.iter().all(Item::is_microchip)
//             || self
//                 .0
//                 .iter()
//                 .filter_map(Item::as_microchip)
//                 .all(|Microchip { element }| self.has_generator_for(element))
//     }

//     fn remove(&mut self, item: &Item<'s>) -> bool {
//         self.0.remove(item)
//     }

//     fn remove_both(&mut self, a: &Item<'s>, b: &Item<'s>) -> bool {
//         self.remove(a) && self.remove(b)
//     }

//     fn remove_either(&mut self, items: Either<&Item<'s>, &(Item<'s>, Item<'s>)>) -> bool {
//         match items {
//             Either::Left(item) => self.remove(item),
//             Either::Right((a, b)) => self.remove_both(a, b),
//         }
//     }

//     fn is_valid_with(&self, item: &Item<'s>) -> bool {
//         self.0.iter().chain([item]).all(Item::is_microchip)
//             || self
//                 .0
//                 .iter()
//                 .chain([item])
//                 .filter_map(Item::as_microchip)
//                 .all(|Microchip { element }| {
//                     self.has_generator_for(element) || item.is_generator_for(element)
//                 })
//     }

//     fn is_valid_with_both(&self, a: &Item<'s>, b: &Item<'s>) -> bool {
//         self.0.iter().chain([a, b]).all(Item::is_microchip)
//             || self
//                 .0
//                 .iter()
//                 .chain([a, b])
//                 .filter_map(Item::as_microchip)
//                 .all(|Microchip { element }| {
//                     self.has_generator_for(element)
//                         || a.is_generator_for(element)
//                         || b.is_generator_for(element)
//                 })
//     }

//     fn is_valid_with_either(&self, items: Either<&Item<'s>, &(Item<'s>, Item<'s>)>) -> bool {
//         match items {
//             Either::Left(item) => self.is_valid_with(item),
//             Either::Right((a, b)) => self.is_valid_with_both(a, b),
//         }
//     }

//     fn is_valid_without(&self, removed: &Item) -> bool {
//         self.0
//             .iter()
//             .filter(|item| *item != removed)
//             .all(Item::is_microchip)
//             || self
//                 .0
//                 .iter()
//                 .filter(|item| *item != removed)
//                 .filter_map(Item::as_microchip)
//                 .all(|Microchip { element }| self.has_generator_for(element))
//     }

//     fn is_valid_without_both(&self, a: &Item, b: &Item) -> bool {
//         self.0
//             .iter()
//             .filter(|item| *item != a && *item != b)
//             .all(Item::is_microchip)
//             || self
//                 .0
//                 .iter()
//                 .filter(|item| *item != a && *item != b)
//                 .filter_map(Item::as_microchip)
//                 .all(|Microchip { element }| self.has_generator_for(element))
//     }

//     fn is_valid_without_either(&self, items: Either<&Item<'s>, &(Item<'s>, Item<'s>)>) -> bool {
//         match items {
//             Either::Left(removed) => self.is_valid_without(removed),
//             Either::Right((a, b)) => self.is_valid_without_both(a, b),
//         }
//     }

//     fn insert(&mut self, item: Item<'s>) -> bool {
//         self.0.insert(item)
//     }

//     fn insert_both(&mut self, a: Item<'s>, b: Item<'s>) -> bool {
//         self.0.insert(a) && self.0.insert(b)
//     }

//     fn insert_either(&mut self, items: Either<Item<'s>, (Item<'s>, Item<'s>)>) -> bool {
//         match items {
//             Either::Left(item) => self.insert(item),
//             Either::Right((a, b)) => self.insert_both(a, b),
//         }
//     }

//     fn has_generator_for(&self, element: &str) -> bool {
//         self.0.contains(&Item::Generator(Generator { element }))
//     }

//     fn iter(&self) -> iter::Iter<'_, 's> {
//         iter::Iter::new(self)
//     }

//     fn par_iter(&self) -> iter::ParIter<'_, 's> {
//         iter::ParIter::new(self)
//     }

//     fn iter_pairs(&self) -> iter::IterPairs<'_, 's> {
//         iter::IterPairs::new(self)
//     }

//     fn par_iter_pairs(&self) -> iter::ParIterPairs<'_, 's> {
//         iter::ParIterPairs::new(self)
//     }

//     fn candidates_for_removal(&self) -> iter::CandidatesForRemoval<'_, 's> {
//         iter::CandidatesForRemoval::new(self)
//     }

//     fn par_candidates_for_removal(&self) -> iter::ParCandidatesForRemoval<'_, 's> {
//         iter::ParCandidatesForRemoval::new(self)
//     }

//     fn next_valid_states<'a, 'b>(
//         &'a self,
//         other: &'b Floor<'s>,
//     ) -> iter::NextValidFloorStates<'a, 'b, 's> {
//         iter::NextValidFloorStates::new(self, other)
//     }

//     fn par_next_valid_states<'a, 'b>(
//         &'a self,
//         other: &'b Floor<'s>,
//     ) -> iter::ParNextValidFloorStates<'a, 'b, 's> {
//         iter::ParNextValidFloorStates::new(self, other)
//     }
// }

// #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
// struct Column<'s> {
//     floors: [Floor<'s>; 4],
//     elevator: Level,
// }

// impl<'s> Column<'s> {
//     fn done(&self) -> bool {
//         self.floors[0].is_empty() && self.floors[1].is_empty() && self.floors[2].is_empty()
//     }

//     fn next_valid_states<'a>(&'a self) -> iter::NextValidColumnStates<'a, 's> {
//         iter::NextValidColumnStates::new(self)
//     }

//     fn par_next_valid_states<'a>(&'a self) -> iter::ParNextValidColumnStates<'a, 's> {
//         iter::ParNextValidColumnStates::new(self)
//     }
// }

// #[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
// #[repr(u8)]
// enum Level {
//     #[default]
//     First = 0,
//     Second,
//     Third,
//     Fourth,
// }

// impl Level {
//     fn as_usize(&self) -> usize {
//         (*self) as u8 as usize
//     }
// }

// #[derive(Debug, Hash, PartialEq, Eq, Clone, Copy, PartialOrd, Ord)]
// struct Microchip<'s> {
//     element: &'s str,
// }

// #[derive(Debug, Hash, PartialEq, Eq, Clone, Copy, PartialOrd, Ord)]
// struct Generator<'s> {
//     element: &'s str,
// }

// #[derive(Debug, Hash, PartialEq, Eq, Clone, Copy, PartialOrd, Ord)]
// enum Item<'s> {
//     Microchip(Microchip<'s>),
//     Generator(Generator<'s>),
// }

// impl<'s> Item<'s> {
//     fn is_microchip(&self) -> bool {
//         matches!(self, Item::Microchip(_))
//     }

//     fn as_microchip(&self) -> Option<Microchip<'s>> {
//         match self {
//             Item::Microchip(microchip) => Some(*microchip),
//             Item::Generator(_) => None,
//         }
//     }

//     fn is_generator_for(&self, element: &str) -> bool {
//         matches!(self, &Item::Generator(Generator { element: element2 }) if element == element2)
//     }
// }

// mod parse {
//     use std::collections::BTreeSet;

//     use eyre::{Report, Result, eyre};
//     use winnow::ascii::alpha1;
//     use winnow::combinator::{alt, preceded, separated, seq, terminated};
//     use winnow::prelude::*;

//     use aoc_common::TryFromStr;

//     use super::{Column, Floor, Generator, Item, Level, Microchip};

//     impl<'s> TryFromStr<'s> for Column<'s> {
//         type Err = Report;

//         fn try_from_str(s: &'s str) -> Result<Self, Self::Err> {
//             s.lines().map(FloorDescription::try_from_str).collect()
//         }
//     }

//     impl<'s> FromIterator<FloorDescription<'s>> for Column<'s> {
//         fn from_iter<T: IntoIterator<Item = FloorDescription<'s>>>(iter: T) -> Self {
//             let mut floors = [
//                 Floor::default(),
//                 Floor::default(),
//                 Floor::default(),
//                 Floor::default(),
//             ];

//             for FloorDescription { level, items } in iter {
//                 floors[level.as_usize()] = items;
//             }

//             Self {
//                 floors,
//                 elevator: Level::First,
//             }
//         }
//     }

//     #[derive(Debug)]
//     struct FloorDescription<'s> {
//         level: Level,
//         items: Floor<'s>,
//     }

//     impl<'s> TryFromStr<'s> for FloorDescription<'s> {
//         type Err = Report;

//         fn try_from_str(s: &'s str) -> Result<Self, Self::Err> {
//             parse_floor.parse(s).map_err(|e| eyre!("{e}"))
//         }
//     }

//     fn parse_floor<'s>(input: &mut &'s str) -> ModalResult<FloorDescription<'s>> {
//         seq! { FloorDescription {
//             _: "The ",
//             level: alt((
//                 "first".map(|_| Level::First),
//                 "second".map(|_| Level::Second),
//                 "third".map(|_| Level::Third),
//                 "fourth".map(|_| Level::Fourth)
//             )),
//             _: " floor contains ",
//             items: alt((
//                 "nothing relevant.".map(|_| BTreeSet::default()),
//                 terminated(
//                     separated(
//                         1..,
//                         preceded(
//                             "a ",
//                             alt((
//                                 terminated(alpha1, "-compatible microchip")
//                                     .map(|element| Microchip { element })
//                                     .map(Item::Microchip),
//                                 terminated(alpha1, " generator")
//                                     .map(|element| Generator { element })
//                                     .map(Item::Generator),
//                             )),
//                         ),
//                         alt((", and ", ", ", " and ")),
//                     ),
//                     '.',
//                 )
//             )).map(Floor)
//         }}
//         .parse_next(input)
//     }
// }

// mod iter {
//     use either::Either;
//     use itertools::Itertools;
//     use rayon::{iter::plumbing::UnindexedConsumer, prelude::*};

//     use super::{Column, Floor, Item, Level};

//     #[derive(Debug, Clone)]
//     pub(super) struct Iter<'a, 's> {
//         inner: std::iter::Copied<std::collections::btree_set::Iter<'a, Item<'s>>>,
//     }

//     impl<'a, 's> Iter<'a, 's> {
//         pub(super) fn new(floor: &'a Floor<'s>) -> Self {
//             Self {
//                 inner: floor.0.iter().copied(),
//             }
//         }
//     }

//     impl<'s> Iterator for Iter<'_, 's> {
//         type Item = Item<'s>;

//         fn next(&mut self) -> Option<Self::Item> {
//             self.inner.next()
//         }
//     }

//     #[derive(Debug, Clone)]
//     pub(super) struct ParIter<'a, 's> {
//         inner: rayon::iter::Copied<rayon::collections::btree_set::Iter<'a, Item<'s>>>,
//     }

//     impl<'a, 's> ParIter<'a, 's> {
//         pub(super) fn new(floor: &'a Floor<'s>) -> Self {
//             Self {
//                 inner: floor.0.par_iter().copied(),
//             }
//         }
//     }

//     impl<'s> ParallelIterator for ParIter<'_, 's> {
//         type Item = Item<'s>;

//         fn drive_unindexed<C>(self, consumer: C) -> C::Result
//         where
//             C: UnindexedConsumer<Self::Item>,
//         {
//             self.inner.drive_unindexed(consumer)
//         }
//     }

//     #[derive(Debug, Clone)]
//     pub(super) struct IterPairs<'a, 's> {
//         inner: itertools::TupleCombinations<Iter<'a, 's>, (Item<'s>, Item<'s>)>,
//     }

//     impl<'s> Iterator for IterPairs<'_, 's> {
//         type Item = (Item<'s>, Item<'s>);

//         fn next(&mut self) -> Option<Self::Item> {
//             self.inner.next()
//         }
//     }

//     impl<'a, 's> IterPairs<'a, 's> {
//         pub(super) fn new(floor: &'a Floor<'s>) -> Self {
//             Self {
//                 inner: Iter::new(floor).tuple_combinations(),
//             }
//         }
//     }

//     #[derive(Debug, Clone)]
//     pub(super) struct ParIterPairs<'a, 's> {
//         inner: rayon::iter::IterBridge<IterPairs<'a, 's>>,
//     }

//     impl<'s> ParallelIterator for ParIterPairs<'_, 's> {
//         type Item = (Item<'s>, Item<'s>);

//         fn drive_unindexed<C>(self, consumer: C) -> C::Result
//         where
//             C: UnindexedConsumer<Self::Item>,
//         {
//             self.inner.drive_unindexed(consumer)
//         }
//     }

//     impl<'a, 's> ParIterPairs<'a, 's> {
//         pub(super) fn new(floor: &'a Floor<'s>) -> Self {
//             Self {
//                 inner: IterPairs::new(floor).par_bridge(),
//             }
//         }
//     }

//     #[derive(Debug)]
//     pub(super) struct CandidatesForRemoval<'a, 's> {
//         singletons: Iter<'a, 's>,
//         pairs: IterPairs<'a, 's>,
//         floor: &'a Floor<'s>,
//     }

//     impl<'a, 's> CandidatesForRemoval<'a, 's> {
//         pub(super) fn new(floor: &'a Floor<'s>) -> Self {
//             let singletons = floor.iter();
//             let pairs = floor.iter_pairs();

//             Self {
//                 singletons,
//                 pairs,
//                 floor,
//             }
//         }
//     }

//     impl<'s> Iterator for CandidatesForRemoval<'_, 's> {
//         type Item = Either<Item<'s>, (Item<'s>, Item<'s>)>;

//         fn next(&mut self) -> Option<Self::Item> {
//             let next = self
//                 .singletons
//                 .next()
//                 .map(Either::Left)
//                 .or_else(|| self.pairs.next().map(Either::Right))?;

//             self.floor
//                 .is_valid_without_either(next.as_ref())
//                 .then_some(next)
//                 .or_else(|| self.next())
//         }
//     }

//     #[derive(Debug)]
//     pub(super) struct ParCandidatesForRemoval<'a, 's> {
//         singletons: ParIter<'a, 's>,
//         pairs: ParIterPairs<'a, 's>,
//         floor: &'a Floor<'s>,
//     }

//     impl<'a, 's> ParCandidatesForRemoval<'a, 's> {
//         pub(super) fn new(floor: &'a Floor<'s>) -> Self {
//             let singletons = floor.par_iter();
//             let pairs = floor.par_iter_pairs();

//             Self {
//                 singletons,
//                 pairs,
//                 floor,
//             }
//         }
//     }

//     impl<'s> ParallelIterator for ParCandidatesForRemoval<'_, 's> {
//         type Item = Either<Item<'s>, (Item<'s>, Item<'s>)>;

//         fn drive_unindexed<C>(self, consumer: C) -> C::Result
//         where
//             C: UnindexedConsumer<Self::Item>,
//         {
//             let singletons = self.singletons.map(Either::Left);
//             let pairs = self.pairs.map(Either::Right);

//             let iter = singletons
//                 .chain(pairs)
//                 .filter(|items| self.floor.is_valid_without_either(items.as_ref()));

//             iter.drive_unindexed(consumer)
//         }
//     }

//     #[derive(Debug)]
//     pub(super) struct NextValidFloorStates<'a, 'b, 's> {
//         a: &'a Floor<'s>,
//         b: &'b Floor<'s>,
//         iter: CandidatesForRemoval<'a, 's>,
//     }

//     impl<'a, 'b, 's> NextValidFloorStates<'a, 'b, 's> {
//         pub(super) fn new(a: &'a Floor<'s>, b: &'b Floor<'s>) -> Self {
//             Self {
//                 a,
//                 b,
//                 iter: a.candidates_for_removal(),
//             }
//         }
//     }

//     impl<'s> Iterator for NextValidFloorStates<'_, '_, 's> {
//         type Item = (Floor<'s>, Floor<'s>);

//         fn next(&mut self) -> Option<Self::Item> {
//             let next = self.iter.next()?;
//             self.b
//                 .is_valid_with_either(next.as_ref())
//                 .then(|| {
//                     let mut a2 = self.a.clone();
//                     let mut b2 = self.b.clone();
//                     a2.remove_either(next.as_ref());
//                     b2.insert_either(next);

//                     (a2, b2)
//                 })
//                 .or_else(|| self.next())
//         }
//     }

//     #[derive(Debug)]
//     pub(super) struct ParNextValidFloorStates<'a, 'b, 's> {
//         a: &'a Floor<'s>,
//         b: &'b Floor<'s>,
//         iter: ParCandidatesForRemoval<'a, 's>,
//     }

//     impl<'a, 'b, 's> ParNextValidFloorStates<'a, 'b, 's> {
//         pub(super) fn new(a: &'a Floor<'s>, b: &'b Floor<'s>) -> Self {
//             Self {
//                 a,
//                 b,
//                 iter: a.par_candidates_for_removal(),
//             }
//         }
//     }

//     impl<'s> ParallelIterator for ParNextValidFloorStates<'_, '_, 's> {
//         type Item = (Floor<'s>, Floor<'s>);

//         fn drive_unindexed<C>(self, consumer: C) -> C::Result
//         where
//             C: UnindexedConsumer<Self::Item>,
//         {
//             self.iter
//                 .filter(|items| self.b.is_valid_with_either(items.as_ref()))
//                 .map(|items| {
//                     let mut a2 = self.a.clone();
//                     let mut b2 = self.b.clone();
//                     a2.remove_either(items.as_ref());
//                     b2.insert_either(items);

//                     (a2, b2)
//                 })
//                 .drive_unindexed(consumer)
//         }
//     }

//     #[derive(Debug)]
//     pub(super) struct NextValidColumnStates<'a, 's> {
//         column: &'a Column<'s>,
//         up: Option<NextValidFloorStates<'a, 'a, 's>>,
//         down: Option<NextValidFloorStates<'a, 'a, 's>>,
//     }

//     impl<'a, 's> NextValidColumnStates<'a, 's> {
//         pub(super) fn new(column: &'a Column<'s>) -> Self {
//             let current_floor_index = column.elevator.as_usize();

//             let middle_floor = &column.floors[current_floor_index];
//             let up = current_floor_index
//                 .checked_add(1)
//                 .and_then(|i| column.floors.get(i))
//                 .map(|upper_floor| middle_floor.next_valid_states(upper_floor));
//             let down = current_floor_index
//                 .checked_sub(1)
//                 .and_then(|i| column.floors.get(i))
//                 .map(|lower_floor| lower_floor.next_valid_states(middle_floor));

//             Self { column, up, down }
//         }
//     }

//     impl<'s> Iterator for NextValidColumnStates<'_, 's> {
//         type Item = Column<'s>;

//         fn next(&mut self) -> Option<Self::Item> {
//             if let Some((middle_floor, upper_floor)) =
//                 self.up.as_mut().and_then(NextValidFloorStates::next)
//             {
//                 Some(match self.column.elevator {
//                     Level::First => Column {
//                         elevator: Level::Second,
//                         floors: [
//                             middle_floor,
//                             upper_floor,
//                             self.column.floors[2].clone(),
//                             self.column.floors[3].clone(),
//                         ],
//                     },
//                     Level::Second => Column {
//                         elevator: Level::Third,
//                         floors: [
//                             self.column.floors[0].clone(),
//                             middle_floor,
//                             upper_floor,
//                             self.column.floors[3].clone(),
//                         ],
//                     },
//                     Level::Third => Column {
//                         elevator: Level::Fourth,
//                         floors: [
//                             self.column.floors[0].clone(),
//                             self.column.floors[1].clone(),
//                             middle_floor,
//                             upper_floor,
//                         ],
//                     },
//                     Level::Fourth => {
//                         unreachable!("there can't be an upper floor above the top floor")
//                     }
//                 })
//             } else if let Some((lower_floor, middle_floor)) =
//                 self.down.as_mut().and_then(NextValidFloorStates::next)
//             {
//                 Some(match self.column.elevator {
//                     Level::First => {
//                         unreachable!("there can't be an lower floor below the bottom floor")
//                     }
//                     Level::Second => Column {
//                         elevator: Level::First,
//                         floors: [
//                             lower_floor,
//                             middle_floor,
//                             self.column.floors[2].clone(),
//                             self.column.floors[3].clone(),
//                         ],
//                     },
//                     Level::Third => Column {
//                         elevator: Level::Second,
//                         floors: [
//                             self.column.floors[0].clone(),
//                             lower_floor,
//                             middle_floor,
//                             self.column.floors[3].clone(),
//                         ],
//                     },
//                     Level::Fourth => Column {
//                         elevator: Level::Third,
//                         floors: [
//                             self.column.floors[0].clone(),
//                             self.column.floors[1].clone(),
//                             lower_floor,
//                             middle_floor,
//                         ],
//                     },
//                 })
//             } else {
//                 return None;
//             }
//         }
//     }

//     #[derive(Debug)]
//     pub(super) struct ParNextValidColumnStates<'a, 's> {
//         column: &'a Column<'s>,
//         up: Option<ParNextValidFloorStates<'a, 'a, 's>>,
//         down: Option<ParNextValidFloorStates<'a, 'a, 's>>,
//     }

//     impl<'a, 's> ParNextValidColumnStates<'a, 's> {
//         pub(super) fn new(column: &'a Column<'s>) -> Self {
//             let current_floor_index = column.elevator.as_usize();

//             let middle_floor = &column.floors[current_floor_index];
//             let up = current_floor_index
//                 .checked_add(1)
//                 .and_then(|i| column.floors.get(i))
//                 .map(|upper_floor| middle_floor.par_next_valid_states(upper_floor));
//             let down = current_floor_index
//                 .checked_sub(1)
//                 .and_then(|i| column.floors.get(i))
//                 .map(|lower_floor| lower_floor.par_next_valid_states(middle_floor));

//             Self { column, up, down }
//         }
//     }

//     impl<'s> ParallelIterator for ParNextValidColumnStates<'_, 's> {
//         type Item = Column<'s>;

//         fn drive_unindexed<C>(self, consumer: C) -> C::Result
//         where
//             C: UnindexedConsumer<Self::Item>,
//         {
//             let up =
//                 self.up
//                     .into_par_iter()
//                     .flatten()
//                     .map(|(middle_floor, upper_floor)| match self.column.elevator {
//                         Level::First => Column {
//                             elevator: Level::Second,
//                             floors: [
//                                 middle_floor,
//                                 upper_floor,
//                                 self.column.floors[2].clone(),
//                                 self.column.floors[3].clone(),
//                             ],
//                         },
//                         Level::Second => Column {
//                             elevator: Level::Third,
//                             floors: [
//                                 self.column.floors[0].clone(),
//                                 middle_floor,
//                                 upper_floor,
//                                 self.column.floors[3].clone(),
//                             ],
//                         },
//                         Level::Third => Column {
//                             elevator: Level::Fourth,
//                             floors: [
//                                 self.column.floors[0].clone(),
//                                 self.column.floors[1].clone(),
//                                 middle_floor,
//                                 upper_floor,
//                             ],
//                         },
//                         Level::Fourth => {
//                             unreachable!("there can't be an upper floor above the top floor")
//                         }
//                     });

//             let down = self
//                 .down
//                 .into_par_iter()
//                 .flatten()
//                 .map(|(lower_floor, middle_floor)| match self.column.elevator {
//                     Level::First => {
//                         unreachable!("there can't be an lower floor below the bottom floor")
//                     }
//                     Level::Second => Column {
//                         elevator: Level::First,
//                         floors: [
//                             lower_floor,
//                             middle_floor,
//                             self.column.floors[2].clone(),
//                             self.column.floors[3].clone(),
//                         ],
//                     },
//                     Level::Third => Column {
//                         elevator: Level::Second,
//                         floors: [
//                             self.column.floors[0].clone(),
//                             lower_floor,
//                             middle_floor,
//                             self.column.floors[3].clone(),
//                         ],
//                     },
//                     Level::Fourth => Column {
//                         elevator: Level::Third,
//                         floors: [
//                             self.column.floors[0].clone(),
//                             self.column.floors[1].clone(),
//                             lower_floor,
//                             middle_floor,
//                         ],
//                     },
//                 });

//             up.chain(down).drive_unindexed(consumer)
//         }
//     }
// }
