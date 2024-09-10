use std::{collections::HashMap, str::FromStr};

use dashmap::{DashMap, Entry};
use eyre::{eyre, OptionExt, Report, Result};
use fnv::FnvBuildHasher;
use rayon::{iter::ParallelIterator, str::ParallelString};
use tinystr::TinyAsciiStr;
use winnow::{
    ascii::{alpha1, digit1},
    combinator::{alt, separated_pair},
    prelude::*,
};

use crate::types::{problem, Problem};

pub const SOME_ASSEMBLY_REQUIRED: Problem = problem!(part1);
type WireName = TinyAsciiStr<4>;
type WireKit = DashMap<WireName, Wire, FnvBuildHasher>;

fn part1(input: &str) -> Result<u16> {
    let num_wires = input.par_lines().count();
    let kit = WireKit::with_capacity_and_hasher(num_wires, FnvBuildHasher::default());
    input
        .par_lines()
        .map(|line| line.parse().unwrap())
        .for_each(|Instruction { input, output }| {
            kit.insert(output, input);
        });

    let a = TinyAsciiStr::from_str("a")?;
    current_on(&kit, a)?.ok_or_eyre("no wire named a")
}

fn part2(input: &str) -> usize {
    todo!()
}

fn current_on(kit: &WireKit, wire: WireName) -> Result<Option<u16>> {
    let Entry::Occupied(mut entry) = kit.entry(wire) else {
        return Ok(None);
    };

    let current = match *entry.get() {
        Wire::And(a, b) => std::thread::scope(|s| {
            let a_handle = s.spawn(|| current_on(kit, a));
            let b_handle = s.spawn(|| current_on(kit, b));

            let a_current = a_handle
                .join()
                .expect("thread not to panic")?
                .ok_or_eyre(format!("No wire named {a} while computing wire {wire}"))?;
            let b_current = b_handle
                .join()
                .expect("thread not to panic")?
                .ok_or_eyre(format!("No wire named {b} while computing wire {wire}"))?;

            Ok::<_, Report>(a_current & b_current)
        })?,
        Wire::LShift(w, n) => current_on(kit, w)?.ok_or_eyre(format!("no wire {w}"))? << n,
        Wire::Not(w) => !current_on(kit, w)?.ok_or_eyre(format!("no wire named {w}"))?,
        Wire::Or(a, b) => std::thread::scope(|s| {
            let a_handle = s.spawn(|| current_on(kit, a));
            let b_handle = s.spawn(|| current_on(kit, b));

            let a_current = a_handle
                .join()
                .expect("child thread not to panic")?
                .ok_or_eyre(format!("no wire {a}"))?;
            let b_current = b_handle
                .join()
                .expect("child thread not to panic")?
                .ok_or_eyre(format!("no wire {b}"))?;

            Ok::<_, Report>(a_current | b_current)
        })?,
        Wire::RShift(w, n) => current_on(kit, w)?.ok_or_eyre(format!("no wire named {w}"))? >> n,

        Wire::Constant(n) => return Ok(Some(n)),
    };

    entry.insert(Wire::Constant(current));
    Ok(Some(current))
}

#[derive(Debug)]
struct Instruction {
    input: Wire,
    output: WireName,
}

impl FromStr for Instruction {
    type Err = Report;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        separated_pair(parse_lhs, " -> ", alpha1.parse_to())
            .map(|(input, output)| Instruction { input, output })
            .parse(s)
            .map_err(|e| eyre!("{e}"))
    }
}

#[derive(Debug)]
enum Wire {
    Constant(u16),
    And(WireName, WireName),
    LShift(WireName, u8),
    Not(WireName),
    Or(WireName, WireName),
    RShift(WireName, u8),
}

fn parse_lhs(input: &mut &str) -> PResult<Wire> {
    alt((constant, and, lshift, not, or, rshift)).parse_next(input)
}

fn constant(input: &mut &str) -> PResult<Wire> {
    digit1.parse_to().map(Wire::Constant).parse_next(input)
}

fn and(input: &mut &str) -> PResult<Wire> {
    separated_pair(alpha1.parse_to(), " AND ", alpha1.parse_to())
        .map(|(a, b)| Wire::And(a, b))
        .parse_next(input)
}

fn lshift(input: &mut &str) -> PResult<Wire> {
    separated_pair(alpha1.parse_to(), " LSHIFT ", digit1.parse_to())
        .map(|(name, shift)| Wire::LShift(name, shift))
        .parse_next(input)
}

fn not(input: &mut &str) -> PResult<Wire> {
    ("NOT ", alpha1.parse_to())
        .map(|(_, name)| Wire::Not(name))
        .parse_next(input)
}

fn or(input: &mut &str) -> PResult<Wire> {
    separated_pair(digit1.parse_to(), " OR ", digit1.parse_to())
        .map(|(a, b)| Wire::Or(a, b))
        .parse_next(input)
}

fn rshift(input: &mut &str) -> PResult<Wire> {
    separated_pair(alpha1.parse_to(), " RSHIFT ", digit1.parse_to())
        .map(|(name, shift)| Wire::RShift(name, shift))
        .parse_next(input)
}
