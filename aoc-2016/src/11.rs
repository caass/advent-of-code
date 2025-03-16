use std::borrow::Cow;

use eyre::{Report, Result, eyre};
use fnv::FnvHashSet;
use itertools::Itertools;
use winnow::ascii::alpha1;
use winnow::combinator::{alt, preceded, separated, seq, terminated};
use winnow::prelude::*;

use aoc_common::{TryFromStr, TryParse};
use aoc_meta::Problem;

pub const RADIOISOTOPE_THERMOELECTRIC_GENERATORS: Problem = Problem::partially_solved(&|input| {
    let column: Column = input.try_parse()?;

    Ok::<_, Report>(format!("{:#?}", column))
});

#[derive(Debug)]
struct Column<'s>(
    FnvHashSet<Item<'s>>,
    FnvHashSet<Item<'s>>,
    FnvHashSet<Item<'s>>,
    FnvHashSet<Item<'s>>,
);

impl<'s> TryFromStr<'s> for Column<'s> {
    type Err = Report;

    fn try_from_str(s: &'s str) -> Result<Self, Self::Err> {
        s.lines().map(Floor::try_from_str).collect()
    }
}

impl<'s> FromIterator<Floor<'s>> for Column<'s> {
    fn from_iter<T: IntoIterator<Item = Floor<'s>>>(iter: T) -> Self {
        let mut floors = [
            FnvHashSet::default(),
            FnvHashSet::default(),
            FnvHashSet::default(),
            FnvHashSet::default(),
        ];

        for Floor { level, items } in iter {
            floors[level as usize] = items;
        }

        let [a, b, c, d] = floors;

        Self(a, b, c, d)
    }
}

#[derive(Debug, Hash, PartialEq, Eq)]
struct Microchip<'s> {
    element: &'s str,
}

#[derive(Debug, Hash, PartialEq, Eq)]
struct Generator<'s> {
    element: &'s str,
}

#[derive(Debug, Hash, PartialEq, Eq)]
enum Item<'s> {
    Microchip(Microchip<'s>),
    Generator(Generator<'s>),
}

#[derive(Debug)]
struct Floor<'s> {
    level: u8,
    items: FnvHashSet<Item<'s>>,
}

impl<'s> TryFromStr<'s> for Floor<'s> {
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

fn parse_floor<'s>(input: &mut &'s str) -> ModalResult<Floor<'s>> {
    seq! { Floor {
        _: "The ",
        level: alt((
            "first".map(|_| 0),
            "second".map(|_| 1),
            "third".map(|_| 2),
            "fourth".map(|_| 3)
        )),
        _: " floor contains ",
        items: alt((
            "nothing relevant.".map(|_| FnvHashSet::default()),
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
        ))
    }}
    .parse_next(input)
}
