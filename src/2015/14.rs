use std::{
    collections::HashMap,
    hash::{Hash, Hasher},
    ops::AddAssign,
};

use eyre::{eyre, OptionExt, Report, Result};
use fnv::FnvBuildHasher;
use itertools::Itertools;
use rayon::prelude::*;
use winnow::{
    ascii::{alpha1, digit1},
    combinator::{eof, preceded, seq},
    error::{ContextError, ParseError},
    Parser,
};

use crate::meta::Problem;

/// https://adventofcode.com/2015/day/14
pub const REINDEER_OLYMPICS: Problem = Problem::solved(&race, &tick_race);

const RACE_DURATION: usize = 2503;

fn race(input: &str) -> Result<usize> {
    let roster = Roster::try_from(input)?;

    let results = roster.race(RACE_DURATION);
    let (_name, distance) = results
        .into_iter()
        .max_by_key(|(_, distance)| *distance)
        .ok_or_eyre("no reindeer in the race")?;

    Ok(distance)
}

fn tick_race(input: &str) -> Result<usize> {
    let roster = Roster::try_from(input)?;

    let results = roster.tick_race(RACE_DURATION);
    let (_name, points) = results
        .into_iter()
        .max_by_key(|(_, distance)| *distance)
        .ok_or_eyre("no reindeer in the race")?;

    Ok(points)
}

struct Roster<'s> {
    reindeer: Vec<Reindeer<'s>>,
}

impl<'s> Roster<'s> {
    fn race(&self, duration: usize) -> HashMap<&'s str, usize, FnvBuildHasher> {
        self.reindeer
            .iter()
            .map(|rd| {
                // let T = the length of the race (in seconds)
                // Let N = the number of full fly/rest cycles the reindeer goes through in T
                // let t = the time remaining after going through N cycles
                // let f = the duration for which the reindeer flies each cycle (in seconds)
                // let r = the duration for which the reindeer rests each cycle (in seconds)
                // let s = the reindeer's flying speed (in km/s)
                // let d = the reindeer's total distance (in km)
                //
                // We can compute the reindeer's distance as
                // N = T / (f + r)
                // t = T % (f + r)
                //
                // N * f * s + t.min(f) * s
                let num_cycles = duration / rd.cycle_duration();
                let time_remaining = duration % rd.cycle_duration();

                let full_cycles_distance = num_cycles * rd.fly_duration * rd.speed;
                let partial_cycle_distance = time_remaining.min(rd.fly_duration) * rd.speed;
                let distance = full_cycles_distance + partial_cycle_distance;

                (rd.name, distance)
            })
            .collect()
    }

    fn tick_race(&self, duration: usize) -> HashMap<&'s str, usize, FnvBuildHasher> {
        // The new rules are equivalent to running a bunch of races under the old rules and giving 1 point
        // to each winner, so let's do that.

        (1..=duration)
            .into_par_iter()
            .map(|n| {
                let standings = self.race(n);
                standings
                    .into_iter()
                    .max_set_by_key(|(_name, distance)| *distance)
            })
            .fold(
                HashMap::<&str, usize, FnvBuildHasher>::default,
                |mut map, leaders| {
                    for (name, _distance) in leaders {
                        map.entry(name).or_default().add_assign(1);
                    }

                    map
                },
            )
            .reduce(HashMap::default, |a, mut b| {
                for (name, points) in a {
                    b.entry(name).or_default().add_assign(points);
                }

                b
            })
    }
}

impl<'s> TryFrom<&'s str> for Roster<'s> {
    type Error = Report;

    fn try_from(value: &'s str) -> Result<Self, Self::Error> {
        value.trim().par_lines().map(Reindeer::try_from).collect()
    }
}

impl<'s> FromParallelIterator<Reindeer<'s>> for Roster<'s> {
    fn from_par_iter<I>(par_iter: I) -> Self
    where
        I: IntoParallelIterator<Item = Reindeer<'s>>,
    {
        Self {
            reindeer: Vec::from_par_iter(par_iter),
        }
    }
}

struct Reindeer<'s> {
    name: &'s str,
    speed: usize,
    fly_duration: usize,
    rest_duration: usize,
}

impl Hash for Reindeer<'_> {
    #[inline(always)]
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.name.hash(state);
    }
}

impl PartialEq for Reindeer<'_> {
    #[inline(always)]
    fn eq(&self, other: &Self) -> bool {
        self.name.eq(other.name)
    }
}

impl Eq for Reindeer<'_> {}

impl Reindeer<'_> {
    #[inline(always)]
    fn cycle_duration(&self) -> usize {
        self.fly_duration + self.rest_duration
    }
}

impl<'s> TryFrom<&'s str> for Reindeer<'s> {
    type Error = Report;

    fn try_from(value: &'s str) -> Result<Self, Self::Error> {
        seq! {Reindeer{
            name: alpha1,
            _: " can fly ",
            speed: digit1.parse_to(),
            _: " km/s for ",
            fly_duration: digit1.parse_to(),
            _: " seconds, but then must rest for ",
            rest_duration: digit1.parse_to(),
            _: preceded(" seconds.", eof)
        }}
        .parse(value.trim())
        .map_err(|e: ParseError<_, ContextError>| eyre!("{e}"))
    }
}
