use std::collections::HashMap;
use std::str::FromStr;
use std::sync::OnceLock;

use eyre::{eyre, OptionExt, Report, Result};
use fnv::FnvBuildHasher;
use rayon::prelude::*;
use tinystr::TinyAsciiStr;
use winnow::{
    ascii::{alpha1, digit1},
    combinator::{alt, separated_pair},
    error::StrContext,
    prelude::*,
};

use crate::meta::Problem;

type WireName = TinyAsciiStr<4>;
const A: WireName = unsafe { WireName::from_bytes_unchecked(*b"a\0\0\0") };
const B: WireName = unsafe { WireName::from_bytes_unchecked(*b"b\0\0\0") };

/// <https://adventofcode.com/2015/day/7>
pub const SOME_ASSEMBLY_REQUIRED: Problem = Problem::solved(
    &|input| {
        input
            .par_lines()
            .map(|line| line.trim().parse())
            .collect::<Result<WireKit>>()?
            .measure(A)
            .ok_or_eyre("failed to measure wire A")
    },
    &|input| {
        let mut kit = input
            .par_lines()
            .map(|line| line.trim().parse())
            .collect::<Result<WireKit>>()?;

        let a = kit.measure(A).ok_or_eyre("failed to measure wire A")?;
        kit.reset();
        kit.set(B, a)?;

        kit.measure(A).ok_or_eyre("failed to measure wire A")
    },
);

#[derive(Debug)]
struct WireKit(HashMap<WireName, MeasuredInput, FnvBuildHasher>);

impl WireKit {
    fn measure(&self, wire: WireName) -> Option<u16> {
        self.0.get(&wire)?.measure(self)
    }

    fn reset(&mut self) {
        self.0.par_iter_mut().for_each(|(_, input)| input.reset());
    }

    fn set(&mut self, wire: WireName, voltage: u16) -> Result<MeasuredInput> {
        self.0
            .insert(
                wire,
                MeasuredInput::new(Input::Constant(Source::Constant(voltage))),
            )
            .ok_or_else(|| eyre!("wire {wire} didn't exist in kit"))
    }
}

impl FromParallelIterator<Connection> for WireKit {
    fn from_par_iter<I>(par_iter: I) -> Self
    where
        I: IntoParallelIterator<Item = Connection>,
    {
        let inner = par_iter
            .into_par_iter()
            .map(|Connection { input, output }| (output, MeasuredInput::new(input)))
            .collect();
        Self(inner)
    }
}

#[derive(Debug)]
struct MeasuredInput {
    input: Input,
    measured: OnceLock<Option<u16>>,
}

impl MeasuredInput {
    fn measure(&self, kit: &WireKit) -> Option<u16> {
        self.measured
            .get_or_init(|| match self.input {
                Input::Constant(Source::Constant(n)) => Some(n),
                Input::Constant(Source::Wire(w)) => kit.measure(w),
                Input::Not(Source::Constant(n)) => Some(!n),
                Input::Not(Source::Wire(w)) => kit.measure(w).map(|n| !n),
                Input::And(Source::Constant(n), Source::Constant(m)) => Some(n & m),
                Input::And(Source::Constant(n), Source::Wire(w))
                | Input::And(Source::Wire(w), Source::Constant(n)) => {
                    kit.measure(w).map(move |m| m & n)
                }
                Input::And(Source::Wire(w1), Source::Wire(w2)) => std::thread::scope(|s| {
                    let m_handle = s.spawn(move || kit.measure(w1));
                    let n_handle = s.spawn(move || kit.measure(w2));

                    let m = m_handle.join().expect("thread not to panic")?;
                    let n = n_handle.join().expect("thread not to panic")?;

                    Some(m & n)
                }),
                Input::Or(Source::Constant(n), Source::Constant(m)) => Some(n | m),
                Input::Or(Source::Constant(n), Source::Wire(w))
                | Input::Or(Source::Wire(w), Source::Constant(n)) => {
                    kit.measure(w).map(move |m| m | n)
                }
                Input::Or(Source::Wire(w1), Source::Wire(w2)) => std::thread::scope(|s| {
                    let m_handle = s.spawn(move || kit.measure(w1));
                    let n_handle = s.spawn(move || kit.measure(w2));

                    let m = m_handle.join().expect("thread not to panic")?;
                    let n = n_handle.join().expect("thread not to panic")?;

                    Some(m | n)
                }),
                Input::LShift(Source::Constant(n), shift) => Some(n << shift),
                Input::LShift(Source::Wire(w), shift) => kit.measure(w).map(move |n| n << shift),
                Input::RShift(Source::Constant(n), shift) => Some(n >> shift),
                Input::RShift(Source::Wire(w), shift) => kit.measure(w).map(move |n| n >> shift),
            })
            .as_ref()
            .copied()
    }

    fn reset(&mut self) {
        self.measured.take();
    }
}

impl MeasuredInput {
    fn new(input: Input) -> Self {
        Self {
            input,
            measured: OnceLock::new(),
        }
    }
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
