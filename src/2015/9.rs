use eyre::{eyre, Result};
use itertools::Itertools;
use petgraph::{graph::NodeIndex, Graph, Undirected};
use rayon::prelude::*;

use winnow::ascii::{alpha1, digit1};
use winnow::combinator::seq;
use winnow::error::{ContextError, ErrMode, ParseError, StrContext};
use winnow::prelude::*;

use crate::types::{problem, Problem};

pub const ALL_IN_A_SINGLE_NIGHT: Problem = problem!(part1, part2);

fn part1(input: &str) -> Result<usize> {
    let stops = input
        .lines()
        .map(Leg::try_from)
        .collect::<Result<Locations, _>>()
        .map_err(|e| eyre!("{e}"))?;

    Ok(stops.shortest_distance())
}

fn part2(input: &str) -> Result<usize> {
    let stops = input
        .lines()
        .map(Leg::try_from)
        .collect::<Result<Locations, _>>()
        .map_err(|e| eyre!("{e}"))?;

    Ok(stops.longest_distance())
}

/// A list of locations Santa has to visit and how far apart they are from each other.
#[derive(Debug)]
struct Locations<'s>(Graph<&'s str, usize, Undirected, u8>);

impl<'s> Locations<'s> {
    /// Construct a new list of stops.
    fn new(graph: Graph<&'s str, usize, Undirected, u8>) -> Self {
        Locations(graph)
    }

    /// Find the shortest distance through this list of stops.
    fn shortest_distance(&self) -> usize {
        self.distances().reduce(|| usize::MAX, std::cmp::min)
    }

    /// Find the longest distance through this list of stops
    fn longest_distance(&self) -> usize {
        self.distances().reduce(|| usize::MIN, std::cmp::max)
    }

    /// Find all possible distances through this list of stops
    fn distances(&self) -> impl ParallelIterator<Item = usize> + '_ {
        let graph = &self.0;

        graph
            .node_indices()
            .permutations(graph.node_count())
            .par_bridge()
            .map(|nodes| {
                nodes
                    .windows(2)
                    .map(|slice| {
                        let a = slice[0];
                        let b = slice[1];
                        let distance = {
                            let distance_maybe = graph.find_edge(a, b);
                            debug_assert!(distance_maybe.is_some());
                            // Safety: the graph is strongly connected
                            unsafe { distance_maybe.unwrap_unchecked() }
                        };

                        graph[distance]
                    })
                    .sum()
            })
    }
}

impl<'s> FromIterator<Leg<'s>> for Locations<'s> {
    fn from_iter<T: IntoIterator<Item = Leg<'s>>>(iter: T) -> Self {
        let mut graph = Graph::default();
        iter.into_iter().for_each(|Leg { from, to, distance }| {
            let from_idx = graph
                .node_weights()
                .position(|&weight| weight == from)
                .map(NodeIndex::<u8>::new)
                .unwrap_or_else(|| graph.add_node(from));

            let to_idx = graph
                .node_weights()
                .position(|&weight| weight == to)
                .map(NodeIndex::<u8>::new)
                .unwrap_or_else(|| graph.add_node(to));

            graph.add_edge(from_idx, to_idx, distance);
        });

        Locations::new(graph)
    }
}

/// A possible leg of Santa's journey
#[derive(Debug)]
struct Leg<'s> {
    from: &'s str,
    to: &'s str,
    distance: usize,
}

impl<'s> TryFrom<&'s str> for Leg<'s> {
    type Error = ParseError<&'s str, ErrMode<ContextError>>;

    fn try_from(input: &'s str) -> Result<Self, Self::Error> {
        seq! { Leg {
            from: alpha1.context(StrContext::Label("from")),
            _: " to ",
            to: alpha1.context(StrContext::Label("to")),
            _: " = ",
            distance: digit1.context(StrContext::Label("distance")).parse_to()
        }}
        .parse(input.trim())
    }
}