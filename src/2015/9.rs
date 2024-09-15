use std::collections::HashMap;
use std::fmt::{self, Display, Formatter};
use std::hash::{Hash, Hasher};
use std::sync::OnceLock;

use eyre::{eyre, OptionExt, Result};
use fnv::FnvBuildHasher;
use itertools::Itertools;
use petgraph::algo::min_spanning_tree;
use petgraph::{graph::NodeIndex, Graph, Undirected};
use rayon::prelude::*;

use winnow::ascii::{alpha1, digit1};
use winnow::combinator::seq;
use winnow::error::{ContextError, ErrMode, ParseError, StrContext};
use winnow::prelude::*;

use crate::types::{problem, Problem};

pub const ALL_IN_A_SINGLE_NIGHT: Problem = problem!(part1);

fn part1(input: &str) -> Result<usize> {
    let stops = input
        .lines()
        .map(Distance::try_from)
        .collect::<Result<Locations, _>>()
        .map_err(|e| eyre!("{e}"))?;

    let routes = Routes::new(&stops);
    let route = routes
        .route_for(&stops)
        .ok_or_eyre("route list didn't have a route for the full stop list")?;
    Ok(route.distance)
}

/// A list of routes that minimize distance between locations.
struct Routes<'s>(HashMap<Locations<'s>, OnceLock<Route<'s>>, FnvBuildHasher>);

impl<'s> Routes<'s> {
    /// Construct a list of routes for a given list of locations and all of its sub-lists.
    fn new(stops: &Locations<'s>) -> Routes<'s> {
        #[allow(
            clippy::mutable_key_type,
            reason = "hash is based on `.stops()`, which is ordered and immutable"
        )]
        let inner = stops.subgraphs().map(|sg| (sg, OnceLock::new())).collect();
        Routes(inner)
    }

    /// Get the shortest route for a list of stops. Returns `None` if the given `Locations` doesn't
    /// exist in the map.
    fn route_for(&self, stops: &Locations<'s>) -> Option<&Route<'s>> {
        self.0
            .get(stops)
            .map(|once| once.get_or_init(|| stops.shortest_route(self)))
    }
}

/// A list of locations Santa has to visit and how far apart they are from each other.
struct Locations<'s> {
    graph: Graph<&'s str, usize, Undirected, u8>,
    stops: OnceLock<Vec<&'s str>>,
}

impl<'s> Locations<'s> {
    /// Construct a new list of stops
    fn new(graph: Graph<&'s str, usize, Undirected, u8>) -> Self {
        Locations {
            graph,
            stops: OnceLock::new(),
        }
    }

    /// Get a sorted slice of all the stops in this list.
    fn stops(&self) -> &[&'s str] {
        self.stops.get_or_init(|| {
            let mut stops = Vec::with_capacity(self.graph.node_count());
            self.graph
                .raw_nodes()
                .iter()
                .for_each(|node| stops.push(node.weight));
            stops.par_sort_unstable();
            stops
        })
    }

    /// Get the number of stops in this list.
    #[inline]
    fn num_stops(&self) -> usize {
        self.stops().len()
    }

    /// Return a subgraph of this list with the given stop removed.
    fn subgraph(&self, except: &str) -> Locations<'s> {
        let mut sg = self.graph.clone();
        if let Some(node) = sg
            .node_indices()
            .find(|ix| sg.node_weight(*ix).is_some_and(|&s| s == except))
        {
            sg.remove_node(node);
        }

        Locations::new(sg)
    }

    /// Get all combinations of stops (n choose k for k in 1..=self.num_stops()) in this list.
    fn subgraphs(&self) -> impl ParallelIterator<Item = Locations<'s>> + '_ {
        (1..=self.num_stops())
            .into_par_iter()
            .flat_map_iter(|k| self.graph.raw_nodes().iter().combinations(k))
            .map(|nodes| {
                let mut stops: Vec<&str> = nodes.into_iter().map(|node| node.weight).collect();
                stops.sort_unstable();

                let mut subgraph = self.graph.clone();
                subgraph.retain_nodes(|g, n| stops.binary_search(&g[n]).is_ok());

                Locations {
                    graph: subgraph,
                    stops: stops.into(),
                }
            })
    }

    /// Find the shortest route through this list of stops, using the given cache to store results for subgraphs.
    fn shortest_route(&self, cache: &Routes<'s>) -> Route<'s> {
        self.stops()
            .into_par_iter()
            .map(|&start| self.shortest_route_from(start, cache))
            .reduce(|| Route::MAX, |a, b| a.min(b))
    }

    /// Find the shortest route through this list of stops, starting at a given stop,
    /// using the given cache to store results for subgraphs.
    fn shortest_route_from(&self, start: &'s str, cache: &Routes<'s>) -> Route<'s> {
        match self.num_stops() {
            // Base case: compute the shortest route starting at `start` through 2 other stops
            3 => {
                // Find nodes `s` (start), `a`, and `b`
                let s = self
                    .graph
                    .node_indices()
                    .find(|i| self.graph[*i] == start)
                    .expect("graph to contain `start`");
                let (a, b) = match s.index() {
                    0 => (NodeIndex::<u8>::new(1), NodeIndex::<u8>::new(2)),
                    1 => (NodeIndex::<u8>::new(0), NodeIndex::<u8>::new(2)),
                    2 => (NodeIndex::<u8>::new(0), NodeIndex::<u8>::new(1)),
                    _ => unreachable!("the list only contains 3 locations"),
                };

                // Find edges `s -> a`, `s -> b`, and `a -> b`.
                let s_a = self
                    .graph
                    .find_edge(s, a)
                    .expect("`start` and `a` to be connected");
                let s_b = self
                    .graph
                    .find_edge(s, b)
                    .expect("`start` and `b` to be connected");
                let a_b = self
                    .graph
                    .find_edge(a, b)
                    .expect("`a` and `b` to be connected");

                // The shortest route is either `s -> a -> b` or `s -> b -> a`.
                // We can determine which by comparing `s -> a` with `s -> b`,
                // because `a -> b` and `b -> a` are the same distance.
                if self.graph[s_a] < self.graph[s_b] {
                    // Shortest route: s -> a -> b
                    let next = self.graph[a];
                    let last = self.graph[b];

                    let distance = self.graph[s_a] + self.graph[a_b];

                    Route {
                        stops: vec![start, next, last],
                        distance,
                    }
                } else {
                    // Shortest route: s -> b -> a
                    let next = self.graph[b];
                    let last = self.graph[a];

                    let distance = self.graph[s_b] + self.graph[a_b];

                    Route {
                        stops: vec![start, next, last],
                        distance,
                    }
                }
            }

            // For anything over length 3, we're in the recursive case.
            4.. => {
                // First, find the shortest route through all stops except `start`.
                let subgraph = self.subgraph(start);
                let subgraph_route = cache
                    .route_for(&subgraph)
                    .expect("cache to contain subgraph");

                // The shortest route that starts with `start`

                todo!()
            }

            0..=2 => unreachable!("the base case is a list with 3 stops"),
        }
    }
}

impl Hash for Locations<'_> {
    /// Hash a list of locations based on the stops it contains.
    #[inline]
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.stops().hash(state);
    }
}

impl PartialEq for Locations<'_> {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.stops().eq(other.stops())
    }
}

impl Eq for Locations<'_> {}

impl<'s> FromIterator<Distance<'s>> for Locations<'s> {
    fn from_iter<T: IntoIterator<Item = Distance<'s>>>(iter: T) -> Self {
        let mut graph = Graph::default();
        iter.into_iter()
            .for_each(|Distance { from, to, distance }| {
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

#[derive(Debug)]
struct Route<'s> {
    stops: Vec<&'s str>,
    distance: usize,
}

impl Route<'_> {
    const MAX: Route<'static> = Route {
        stops: vec![],
        distance: usize::MAX,
    };
}

impl Display for Route<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        #[allow(unstable_name_collisions, reason = "Once it errors, I'll change it")]
        self.stops
            .iter()
            .copied()
            .intersperse(" -> ")
            .try_for_each(|s| f.write_str(s))?;

        write!(f, " = {}", self.distance)
    }
}

impl PartialEq for Route<'_> {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.distance.eq(&other.distance)
    }
}

impl Eq for Route<'_> {}

impl PartialOrd for Route<'_> {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Route<'_> {
    #[inline]
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.distance.cmp(&other.distance)
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
        seq! { Distance {
            from: alpha1.context(StrContext::Label("from")),
            _: " to ",
            to: alpha1.context(StrContext::Label("to")),
            _: " = ",
            distance: digit1.context(StrContext::Label("distance")).parse_to()
        }}
        .parse(input.trim())
    }
}
