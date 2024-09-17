use eyre::{eyre, Result};
use itertools::Itertools;
use petgraph::graph::NodeIndex;
use petgraph::{Directed, Graph};
use rayon::prelude::*;
use winnow::ascii::{alpha1, digit1};
use winnow::combinator::{alt, eof, preceded, separated_pair, seq};
use winnow::error::{ContextError, ParseError};
use winnow::prelude::*;

use crate::types::{problem, Problem};

pub const KNIGHTS_OF_THE_DINNER_TABLE: Problem =
    problem!(optimize_happiness, optimize_happiness_with_self);

fn optimize_happiness(input: &str) -> Result<isize> {
    let table: Table = input.try_into().map_err(|e| eyre!("{e}"))?;
    Ok(table.optimal_happiness())
}

fn optimize_happiness_with_self(input: &str) -> Result<isize> {
    let mut table: Table = input.try_into().map_err(|e| eyre!("{e}"))?;
    table.seat_self();
    Ok(table.optimal_happiness())
}

#[derive(Debug)]
struct Table<'s>(Graph<&'s str, isize, Directed, u8>);

impl Table<'_> {
    fn optimal_happiness(&self) -> isize {
        let graph = &self.0;
        let k = graph.node_count();
        graph
            .node_indices()
            .permutations(k)
            .par_bridge()
            .map(|arrangement| {
                (0..k)
                    .map(|i| {
                        let a = arrangement[(i + k - 1) % k];
                        let b = arrangement[i];
                        let c = arrangement[(i + 1) % k];

                        let left = graph.find_edge(b, a).unwrap();
                        let right = graph.find_edge(b, c).unwrap();

                        graph[left] + graph[right]
                    })
                    .sum()
            })
            .max()
            .unwrap()
    }

    fn seat_self(&mut self) {
        let graph = &mut self.0;
        let me = graph.add_node("Cass");
        graph.node_indices().filter(|i| *i != me).for_each(|i| {
            graph.add_edge(i, me, 0);
            graph.add_edge(me, i, 0);
        });
    }
}

impl<'s> TryFrom<&'s str> for Table<'s> {
    type Error = ParseError<&'s str, ContextError>;

    fn try_from(value: &'s str) -> Result<Self, Self::Error> {
        value.trim().lines().map(Relationship::try_from).collect()
    }
}

impl<'s> FromIterator<Relationship<'s>> for Table<'s> {
    fn from_iter<T: IntoIterator<Item = Relationship<'s>>>(iter: T) -> Self {
        let mut graph = Graph::default();

        for Relationship {
            subject,
            object,
            feeling,
        } in iter
        {
            let a = graph
                .node_weights()
                .position(|&name| name == subject)
                .map(NodeIndex::<u8>::new)
                .unwrap_or_else(|| graph.add_node(subject));
            let b = graph
                .node_weights()
                .position(|&name| name == object)
                .map(NodeIndex::<u8>::new)
                .unwrap_or_else(|| graph.add_node(object));

            graph.add_edge(a, b, feeling);
        }

        Self(graph)
    }
}

#[derive(Debug)]
struct Relationship<'s> {
    subject: &'s str,
    object: &'s str,
    feeling: isize,
}

impl<'s> TryFrom<&'s str> for Relationship<'s> {
    type Error = ParseError<&'s str, ContextError>;

    fn try_from(value: &'s str) -> Result<Self, Self::Error> {
        fn parse_feeling(input: &mut &str) -> PResult<isize> {
            let sign = alt(("gain".map(|_| 1isize), "lose".map(|_| -1isize)));
            let magnitude = digit1.parse_to::<isize>();

            separated_pair(sign, " ", magnitude)
                .map(|(s, m)| s * m)
                .parse_next(input)
        }

        seq! {Relationship{
            subject: alpha1,
            _: " would ",
            feeling: parse_feeling,
            _: " happiness units by sitting next to ",
            object: alpha1,
            _: preceded('.', eof)
        }}
        .parse(value.trim())
    }
}

#[test]
fn example() {
    let input = "Alice would gain 54 happiness units by sitting next to Bob.
    Alice would lose 79 happiness units by sitting next to Carol.
    Alice would lose 2 happiness units by sitting next to David.
    Bob would gain 83 happiness units by sitting next to Alice.
    Bob would lose 7 happiness units by sitting next to Carol.
    Bob would lose 63 happiness units by sitting next to David.
    Carol would lose 62 happiness units by sitting next to Alice.
    Carol would gain 60 happiness units by sitting next to Bob.
    Carol would gain 55 happiness units by sitting next to David.
    David would gain 46 happiness units by sitting next to Alice.
    David would lose 7 happiness units by sitting next to Bob.
    David would gain 41 happiness units by sitting next to Carol.";

    let table: Table = input.try_into().unwrap();

    assert_eq!(table.optimal_happiness(), 330);
}