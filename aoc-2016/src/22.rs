use std::str::FromStr;

use aoc_meta::Problem;
use eyre::{OptionExt, Result, bail};
use itertools::Itertools;
use rayon::prelude::*;

pub const GRID_COMPUTING: Problem = Problem::partially_solved(&viable_pairs);

fn viable_pairs(input: &str) -> Result<usize> {
    let nodes = parse_nodes(input)?;

    Ok(nodes
        .par_iter()
        .copied()
        .filter(|a| a.used > 0)
        .flat_map(|a| {
            nodes
                .par_iter()
                .copied()
                .filter(move |&b| !(a.x == b.x && a.y == b.y) && a.used <= b.available)
        })
        .count())
}

#[derive(Debug, Clone, Copy)]
struct PositionedNode {
    x: u8,
    y: u8,
    used: u8,
    available: u8,
}

impl FromStr for PositionedNode {
    type Err = eyre::Report;

    fn from_str(line: &str) -> Result<PositionedNode> {
        const LINE_PREFIX: &str = "/dev/grid/node-";
        const TERABYTE_SUFFIX_LEN: usize = "T".len();
        const COORD_PREFIX_LEN: usize = const {
            if "x".len() != "y".len() {
                panic!("mismatched prefix len between 'x' and 'y'")
            } else {
                "x".len()
            }
        };

        let trimmed = line.trim_start_matches(LINE_PREFIX);
        if trimmed == line {
            bail!("couldn't find prefix \"{LINE_PREFIX}\" in line \"{line}\"")
        }

        let Some([coords, _size, used, available, _use_percent]) =
            trimmed.split_ascii_whitespace().collect_array()
        else {
            bail!("failed to split line \"{line}\" into components")
        };

        let Some((x, y)) = coords.split_once("-") else {
            bail!("failed to split {coords} into x and y");
        };

        Ok(PositionedNode {
            x: x[COORD_PREFIX_LEN..].parse()?,
            y: y[COORD_PREFIX_LEN..].parse()?,
            used: used[..used.len() - TERABYTE_SUFFIX_LEN]
                .parse::<u16>()?
                .try_into()
                .unwrap_or(u8::MAX),
            available: available[..available.len() - TERABYTE_SUFFIX_LEN]
                .parse::<u16>()?
                .try_into()
                .unwrap_or(u8::MAX),
        })
    }
}

fn parse_nodes(input: &str) -> Result<Vec<PositionedNode>> {
    let (_, after) = input
        .split_once("Use%\n")
        .ok_or_eyre("couldn't find \"Use%\\n\" in input")?;

    after.par_lines().map(PositionedNode::from_str).collect()
}
