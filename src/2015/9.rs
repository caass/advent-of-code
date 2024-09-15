use std::{hash::Hash, iter::FusedIterator, sync::mpsc, usize};

use eyre::{eyre, Report, Result};
use petgraph::{
    algo,
    csr::IndexType,
    data::DataMap,
    graph::{NodeIndex, NodeIndices},
    EdgeType, Graph, IntoWeightedEdge, Undirected,
};
use rayon::prelude::*;
use winnow::{
    ascii::{alpha1, digit1},
    combinator::seq,
    error::{ContextError, ErrMode, ParseError, StrContext},
    PResult, Parser,
};

use crate::types::{problem, Problem};

pub const ALL_IN_A_SINGLE_NIGHT: Problem = problem!(part1);

fn part1(input: &str) -> Result<usize> {
    let map = input
        .lines()
        .map(Distance::try_from)
        .collect::<Result<Map, _>>()
        .unwrap();

    Ok(0)
}

#[derive(Debug)]
struct Map<'s>(Graph<&'s str, usize, Undirected, u8>);

// fn par_node_indices<N, E, Ty: EdgeType, Ix: IndexType + Send>(
//     graph: &Graph<N, E, Ty, Ix>,
// ) -> impl IndexedParallelIterator<Item = NodeIndex<Ix>> {
//     (0..graph.node_count())
//         .into_par_iter()
//         .map(NodeIndex::<Ix>::new)
// }

impl<'s> Map<'s> {
    fn shortest_route<'m: 's>(&'m self) -> Route<'s, 'm> {
        self.0
            .node_indices()
            .par_bridge()
            .map(|start| self.shortest_route_from(start))
            .reduce(
                || Route {
                    map: self,
                    stops: vec![],
                    distance: usize::MAX,
                },
                |a, b| a.shorter(b),
            )
    }

    fn shortest_route_from(&self, start: NodeIndex<u8>) -> Route {
        let stops = Stops::new(start, self.0.node_indices());
        self.shortest_route_through(start, stops)
    }

    fn shortest_route_through<I: ExactSizeIterator<Item = NodeIndex<u8>> + Clone>(
        &self,
        start: NodeIndex<u8>,
        mut stops: I,
    ) -> Route {
        let graph = &self.0;
        // TODO: cache already visited segments in a map to de-duplicate work

        match stops.len() {
            // The shortest distance through 0 stops is the identity route
            0 => Route {
                map: self,
                stops: vec![start],
                distance: 0,
            },

            // The shortest distance through 1 stop is from `start` to `end`.
            1 => {
                let end = {
                    let end_maybe = stops.next();
                    debug_assert!(end_maybe.is_some());
                    // Safety: `.len()` said there's 1 stop left
                    unsafe { end_maybe.unwrap_unchecked() }
                };
                debug_assert!(stops.next().is_none());

                let distance_ix = graph.find_edge(start, end).unwrap_or_else(|| {
                    let start_name = graph[start];
                    let end_name = graph[end];
                    panic!("Couldn't find a distance between {start_name} and {end_name}")
                });
                let distance = graph[distance_ix];

                Route {
                    map: self,
                    stops: vec![start, end],
                    distance,
                }
            }

            // The shortest distance through 2 stops is either `start -> a -> b` or `start -> b -> a`
            2 => {
                let a = {
                    let a_maybe = stops.next();
                    debug_assert!(a_maybe.is_some());
                    // Safety: `.len()` said there's 2 stops left.
                    unsafe { a_maybe.unwrap_unchecked() }
                };
                let b = {
                    let b_maybe = stops.next();
                    debug_assert!(b_maybe.is_some());
                    // Safety: `.len()` said there's 2 stops left.
                    unsafe { b_maybe.unwrap_unchecked() }
                };
                debug_assert!(stops.next().is_none());

                let start_to_a = graph.find_edge(start, a).unwrap_or_else(|| {
                    let start_name = graph[start];
                    let a_name = graph[a];
                    panic!("Couldn't find a distance between {start_name} and {a_name}");
                });
                let start_to_b = graph.find_edge(start, b).unwrap_or_else(|| {
                    let start_name = graph[start];
                    let b_name = graph[b];
                    panic!("Couldn't find a distance between {start_name} and {b_name}");
                });

                let a_to_b = graph.find_edge(a, b).unwrap_or_else(|| {
                    let a_name = graph[a];
                    let b_name = graph[b];
                    panic!("Couldn't find a distance between {a_name} and {b_name}");
                });

                let mut route = Route {
                    map: self,
                    stops: Vec::with_capacity(3),
                    distance: graph[a_to_b],
                };
                route.stops.push(start);

                if graph[start_to_a] < graph[start_to_b] {
                    route.distance += graph[start_to_a];

                    route.stops.push(a);
                    route.stops.push(b);
                } else {
                    route.distance += graph[start_to_b];

                    route.stops.push(b);
                    route.stops.push(a);
                };

                route
            }
            3.. => {
                let
            }
        }
    }
}

#[derive(Debug, Clone)]
struct Stops<I> {
    indices: I,
    start: Option<NodeIndex<u8>>,
}

impl<I: Iterator<Item = NodeIndex<u8>>> Stops<I> {
    fn new(start: NodeIndex<u8>, indices: I) -> Self {
        Self {
            indices,
            start: Some(start),
        }
    }

    fn skip_start(&mut self, next: NodeIndex<u8>) -> Option<NodeIndex<u8>> {
        if self.start.is_some_and(|ix| ix == next) {
            self.start = None;
            None
        } else {
            Some(next)
        }
    }
}

impl<I: Iterator<Item = NodeIndex<u8>>> Iterator for Stops<I> {
    type Item = NodeIndex<u8>;

    fn next(&mut self) -> Option<Self::Item> {
        self.indices.next().and_then(|ix| self.skip_start(ix))
    }
}

impl<I: ExactSizeIterator + Iterator<Item = NodeIndex<u8>>> ExactSizeIterator for Stops<I> {
    #[inline(always)]
    fn len(&self) -> usize {
        self.indices.len() - (self.start.is_some() as u8 as usize)
    }
}

impl<I: DoubleEndedIterator + Iterator<Item = NodeIndex<u8>>> DoubleEndedIterator for Stops<I> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.indices.next_back().and_then(|ix| self.skip_start(ix))
    }
}

impl<I: FusedIterator + Iterator<Item = NodeIndex<u8>>> FusedIterator for Stops<I> {}

impl<'s> FromIterator<Distance<'s>> for Map<'s> {
    fn from_iter<T: IntoIterator<Item = Distance<'s>>>(iter: T) -> Self {
        let mut graph = Graph::default();

        for Distance { from, to, distance } in iter {
            let a = graph
                .node_weights()
                .position(|&location| location == from)
                .map(|ix| (ix as u8).into())
                .unwrap_or_else(|| graph.add_node(from));

            let b = graph
                .node_weights()
                .position(|&location| location == to)
                .map(|ix| (ix as u8).into())
                .unwrap_or_else(|| graph.add_node(to));
            graph.add_edge(a, b, distance);
        }

        Self(graph)
    }
}

#[derive(Debug)]
struct Route<'s, 'm> {
    map: &'m Map<'s>,
    stops: Vec<NodeIndex<u8>>,
    distance: usize,
}

impl<'s, 'm> Route<'s, 'm> {
    fn shorter(self, other: Route<'s, 'm>) -> Route<'s, 'm> {
        if self.distance < other.distance {
            self
        } else {
            other
        }
    }
}

#[derive(Debug)]
struct Distance<'s> {
    from: &'s str,
    to: &'s str,
    distance: usize,
}

impl<'s> TryFrom<&'s str> for Distance<'s> {
    type Error = ParseError<&'s str, ErrMode<ContextError>>;

    fn try_from(input: &'s str) -> Result<Self, Self::Error> {
        seq! {Distance {
            from: alpha1.context(StrContext::Label("from")),
            _: " to ",
            to: alpha1.context(StrContext::Label("to")),
            _: " = ",
            distance: digit1.context(StrContext::Label("distance")).parse_to()
        }}
        .parse(input.trim())
    }
}

impl<'s> IntoWeightedEdge<usize> for Distance<'s> {
    type NodeId = &'s str;

    fn into_weighted_edge(self) -> (Self::NodeId, Self::NodeId, usize) {
        let Distance { from, to, distance } = self;
        (from, to, distance)
    }
}
