use std::collections::hash_map::Entry;
use std::collections::{HashMap, VecDeque};
use std::hash::{Hash, Hasher};
use std::ops::{Deref, DerefMut};
use std::str::FromStr;
use std::sync::Mutex;
use std::sync::atomic::{AtomicU8, Ordering};
use std::{array, iter};

use deranged::RangedU8;
use eyre::{Report, Result, bail, eyre};
use fnv::FnvHashMap;
use itertools::Itertools;
use nohash_hasher::BuildNoHashHasher;
use tinyvec::ArrayVec;
use winnow::ascii::alpha1;
use winnow::combinator::{alt, preceded, separated, terminated};
use winnow::error::ContextError;
use winnow::prelude::*;
use winnow::stream::Accumulate;

use aoc_meta::Problem;

pub const RADIOISOTOPE_THERMOELECTRIC_GENERATORS: Problem =
    Problem::solved(&minimum_steps, &minimum_steps_with_two_more_pairs);

fn minimum_steps(input: &str) -> Result<u8> {
    input.parse().and_then(Column::fewest_steps_to_solve)
}

fn minimum_steps_with_two_more_pairs(input: &str) -> Result<u8> {
    let mut start: Column = input.parse()?;
    start.add_pairs(RangedU8::new_static::<0>(), 2);
    start.fewest_steps_to_solve()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct Element(RangedU8<0, 127>);

impl Element {
    const fn new(id: RangedU8<0, 127>) -> Self {
        Self(id)
    }

    fn next(&self) -> Option<Element> {
        self.0.checked_add(1).map(Element)
    }
}

impl Default for Element {
    fn default() -> Self {
        Self(RangedU8::new_static::<0>())
    }
}

impl Hash for Element {
    fn hash<H: Hasher>(&self, state: &mut H) {
        state.write_u8(self.0.get());
    }
}

impl nohash_hasher::IsEnabled for Element {}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct Item(u8);

impl Item {
    #[inline(always)]
    const fn pair(element: Element) -> [Item; 2] {
        [Item::generator(element), Item::microchip(element)]
    }

    #[inline(always)]
    const fn generator(element: Element) -> Item {
        Item(element.0.get())
    }

    #[inline(always)]
    const fn microchip(element: Element) -> Item {
        Item(0b1000_0000 | element.0.get())
    }

    #[inline(always)]
    const fn element(&self) -> Element {
        // Safety: we know that an integer bitwise-anded with a leading 0 will always be <= 127.
        Element(unsafe { RangedU8::new_unchecked(self.0 & 0b0111_1111) })
    }

    #[inline(always)]
    const fn set_element(&mut self, element: Element) {
        self.0 = (self.0 & 0b1000_0000) | element.0.get()
    }

    #[inline(always)]
    const fn is_generator(&self) -> bool {
        self.0 >> 7 == 0
    }

    #[inline(always)]
    const fn is_microchip(&self) -> bool {
        !self.is_generator()
    }

    #[inline(always)]
    const fn paired_generator(&self) -> Item {
        Item(self.0 & 0b0111_1111)
    }
}

impl Hash for Item {
    fn hash<H: Hasher>(&self, state: &mut H) {
        state.write_u8(self.0);
    }
}

impl nohash_hasher::IsEnabled for Item {}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct Floor(ArrayVec<[Item; 16]>);

impl Floor {
    fn is_valid(&self) -> bool {
        self.iter().all(Item::is_microchip)
            || self
                .iter()
                .all(|item| item.is_generator() || self.contains(&item.paired_generator()))
    }

    #[inline(always)]
    fn push(&mut self, item: Item) {
        self.0.push(item);
    }

    #[inline(always)]
    fn swap_remove(&mut self, idx: usize) -> Item {
        self.0.swap_remove(idx)
    }

    fn from_str_with_parser<'s>(input: &'s str, f: impl Fn(&'s str) -> Element) -> Result<Self> {
        terminated(
            alt((
                "nothing relevant".map(|_| Floor::default()),
                preceded(
                    "a ",
                    separated(
                        1..,
                        alt((
                            terminated(alpha1::<_, ContextError>, "-compatible microchip")
                                .map(|elem| Item::microchip(f(elem))),
                            terminated(alpha1, " generator").map(|elem| Item::generator(f(elem))),
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

    fn extend<I: IntoIterator<Item = Item>>(&mut self, iter: I) {
        self.0.extend(iter);
    }
}

impl Deref for Floor {
    type Target = [Item];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Floor {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl FromIterator<Item> for Floor {
    fn from_iter<T: IntoIterator<Item = Item>>(iter: T) -> Self {
        Self(FromIterator::from_iter(iter))
    }
}

impl Accumulate<Item> for Floor {
    fn initial(_capacity: Option<usize>) -> Self {
        Self::default()
    }

    fn accumulate(&mut self, acc: Item) {
        self.push(acc);
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct Column {
    floors: [Floor; 4],
    elevator: RangedU8<0, 3>,
}

impl Column {
    fn target(&self) -> Column {
        let mut target = Column {
            elevator: RangedU8::new_static::<3>(),
            floors: array::from_fn(|i| {
                if i == 3 {
                    self.floors.iter().flat_map(|f| f.iter()).copied().collect()
                } else {
                    Floor(ArrayVec::new())
                }
            }),
        };
        target.canonicalize();

        target
    }

    fn add_pairs(&mut self, floor: RangedU8<0, 3>, n: usize) {
        let mut max = self
            .floors
            .iter()
            .flat_map(|f| f.iter())
            .max_by_key(|it| it.element())
            .and_then(|it| it.element().next())
            .unwrap_or_default();

        self.floors[usize::from(floor.get())].extend(
            iter::repeat_with(|| {
                let elem = max;
                max = max.next().expect("to have fewer than 127 elements");
                elem
            })
            .take(n)
            .flat_map(Item::pair),
        )
    }

    fn len(&self) -> usize {
        self.floors.iter().map(|f| f.len()).sum()
    }

    /// Converts this column into the equivalent canonical column.
    fn canonicalize(&mut self) {
        let mut map = HashMap::<_, _, BuildNoHashHasher<Element>>::with_capacity_and_hasher(
            self.len(),
            BuildNoHashHasher::default(),
        );
        let mut n = RangedU8::new_static::<0>();

        for item in self.floors.iter_mut().flat_map(|f| f.iter_mut()) {
            let new_element = *map.entry(item.element()).or_insert_with(|| {
                let next = Element::new(n);
                n = n.checked_add(1).expect("to have fewer than 127 elements");
                next
            });

            item.set_element(new_element);
        }

        for floor in &mut self.floors {
            floor.0.sort_unstable();
        }
    }

    fn next_valid_states(&self) -> impl Iterator<Item = Column> {
        self.elevator
            .checked_add(1)
            .into_iter()
            .chain(self.elevator.checked_sub(1))
            .flat_map(|to| self.valid_swaps_to(to))
    }

    fn fewest_steps_to_solve(mut self) -> Result<u8> {
        self.canonicalize();
        let target_state = self.target();

        let mut queue = VecDeque::default();
        let mut visited = FnvHashMap::default();

        visited.insert(self, 0);
        queue.push_back((self, 0));

        while let Some((next, steps)) = queue.pop_front() {
            if next == target_state {
                return Ok(steps);
            }

            for mut state in next.next_valid_states() {
                state.canonicalize();

                if let Entry::Vacant(vac) = visited.entry(state) {
                    vac.insert(steps + 1);
                    queue.push_back((state, steps + 1));
                }
            }
        }

        Err(eyre!("no solution"))
    }

    fn valid_swaps_to(&self, to_idx: RangedU8<0, 3>) -> impl Iterator<Item = Column> {
        let from_idx = self.elevator;

        let from = self.floors[usize::from(from_idx.get())];
        let to = self.floors[usize::from(to_idx.get())];

        (0..from.len())
            .tuple_combinations()
            .map(|(a, b)| (a, Some(b)))
            .chain((0..from.len()).map(|a| (a, None)))
            .filter_map(move |(a, maybe_b)| {
                let mut from = from;
                let mut to = to;

                match maybe_b {
                    Some(b) => {
                        // Always remove the higher index first to avoid invalidating the lower one
                        let (first, second) = if a > b { (a, b) } else { (b, a) };
                        to.push(from.swap_remove(first));
                        to.push(from.swap_remove(second));
                    }
                    None => {
                        to.push(from.swap_remove(a));
                    }
                }

                if from.is_valid() && to.is_valid() {
                    let mut next = *self;

                    next.elevator = to_idx;
                    next.floors[usize::from(to_idx.get())] = to;
                    next.floors[usize::from(from_idx.get())] = from;

                    Some(next)
                } else {
                    None
                }
            })
    }
}

impl Default for Column {
    fn default() -> Column {
        Column {
            floors: Default::default(),
            elevator: RangedU8::new_static::<0>(),
        }
    }
}

impl FromStr for Column {
    type Err = Report;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut col = Column::default();
        let n = AtomicU8::new(0);
        let registry = Mutex::new(FnvHashMap::default());

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

            col.floors[i] = Floor::from_str_with_parser(contains, |elem| {
                *registry
                    .lock()
                    .expect("to have fewer than 127 elements")
                    .entry(elem)
                    .or_insert_with(|| {
                        let id = RangedU8::new(n.fetch_add(1, Ordering::Relaxed))
                            .expect("to have fewer than 127 elements");
                        Element::new(id)
                    })
            })?;
        }

        Ok(col)
    }
}
