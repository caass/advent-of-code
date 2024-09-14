use std::{
    cell::UnsafeCell, collections::HashMap, hint::unreachable_unchecked, str::FromStr, sync::Once,
};

use dashmap::{DashMap, Entry};
use eyre::{bail, eyre, OptionExt, Report, Result};
use fnv::FnvBuildHasher;
use rayon::{iter::ParallelIterator, str::ParallelString};
use tinystr::TinyAsciiStr;
use winnow::{
    ascii::{alpha1, digit1},
    combinator::{alt, fail, separated_pair},
    error::StrContext,
    prelude::*,
};

use crate::types::{problem, Problem};

pub const SOME_ASSEMBLY_REQUIRED: Problem = problem!(part1);
type WireName = TinyAsciiStr<4>;
type WireKit = DashMap<WireName, Input, FnvBuildHasher>;

const A: WireName = unsafe { WireName::from_bytes_unchecked(*b"a\0\0\0") };

fn part1(input: &str) -> Result<u16> {
    let num_wires = input.par_lines().count();
    let kit =
        WireKit::with_capacity_and_hasher_and_shard_amount(num_wires, FnvBuildHasher::default(), 2);
    input.par_lines().try_for_each(|line| {
        let Connection { input, output } = line.trim().parse()?;
        if let Some(previous) = kit.insert(output, input) {
            Err(eyre!("Found duplicate value for {output}: {previous:#?}"))
        } else {
            Ok(())
        }
    })?;

    todo!()
}

fn part2(input: &str) -> usize {
    todo!()
}

#[derive(Debug)]
struct Connection {
    input: Input,
    output: WireName,
}

impl FromStr for Connection {
    type Err = Report;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        separated_pair(
            input,
            " -> ",
            alpha1.parse_to().context(StrContext::Label("output")),
        )
        .context(StrContext::Label("line"))
        .map(|(input, output)| Connection { input, output })
        .parse(s)
        .map_err(|e| eyre!("Error parsing instruction from {e}"))
    }
}

#[derive(Debug)]
enum Input {
    Constant(Source),
    Not(Source),
    And(Source, Source),
    Or(Source, Source),
    LShift(Source, u8),
    RShift(Source, u8),
}

fn input(input: &mut &str) -> PResult<Input> {
    alt((not, lshift, rshift, and, or, constant))
        .context(StrContext::Label("input"))
        .parse_next(input)
}

fn constant(input: &mut &str) -> PResult<Input> {
    source
        .map(Input::Constant)
        .context(StrContext::Label("constant wire statement"))
        .parse_next(input)
}

fn not(input: &mut &str) -> PResult<Input> {
    ("NOT ", source)
        .map(|(_, name)| Input::Not(name))
        .context(StrContext::Label("NOT statement"))
        .parse_next(input)
}

fn lshift(input: &mut &str) -> PResult<Input> {
    separated_pair(
        source,
        " LSHIFT ",
        digit1.parse_to().context(StrContext::Label("shift amount")),
    )
    .map(|(name, shift)| Input::LShift(name, shift))
    .context(StrContext::Label("LSHIFT statement"))
    .parse_next(input)
}

fn rshift(input: &mut &str) -> PResult<Input> {
    separated_pair(
        source,
        " RSHIFT ",
        digit1.parse_to().context(StrContext::Label("shift amount")),
    )
    .map(|(name, shift)| Input::RShift(name, shift))
    .context(StrContext::Label("RSHIFT statement"))
    .parse_next(input)
}

fn and(input: &mut &str) -> PResult<Input> {
    separated_pair(source, " AND ", source)
        .map(|(a, b)| Input::And(a, b))
        .context(StrContext::Label("AND statement"))
        .parse_next(input)
}

fn or(input: &mut &str) -> PResult<Input> {
    separated_pair(source, " OR ", source)
        .map(|(a, b)| Input::Or(a, b))
        .context(StrContext::Label("OR statement"))
        .parse_next(input)
}

#[derive(Debug)]
enum Source {
    Wire(WireName),
    Constant(u16),
}

fn source(input: &mut &str) -> PResult<Source> {
    alt((
        alpha1
            .parse_to()
            .map(Source::Wire)
            .context(StrContext::Label("constant voltage")),
        digit1
            .parse_to()
            .map(Source::Constant)
            .context(StrContext::Label("wire name")),
    ))
    .context(StrContext::Label("input source"))
    .parse_next(input)
}
