use std::str::FromStr;

use aoc_meta::Problem;
use eyre::{OptionExt, Result, bail};
use itertools::Itertools;
use rayon::prelude::*;

pub const GRID_COMPUTING: Problem = Problem::partially_solved(&viable_pairs);

fn viable_pairs(input: &str) -> Result<usize> {
    let (_, after) = input
        .split_once("Use%\n")
        .ok_or_eyre("couldn't find \"Use%\\n\" in input")?;

    let nodes = after
        .par_lines()
        .map(PositionedNode::from_str)
        .collect::<Result<Vec<_>>>()?;

    let n = nodes
        .par_iter()
        .copied()
        .filter(|a| a.used > 0)
        .flat_map(|a| {
            nodes
                .par_iter()
                .copied()
                .filter(move |&b| !(a.x == b.x && a.y == b.y) && a.used <= b.available)
        })
        .count();

    Ok(n)
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

        let Some((before, after)) = line.split_at_checked(LINE_PREFIX.len()) else {
            bail!("line \"{line}\" was too short or missing prefix \"{LINE_PREFIX}\"")
        };

        if before != LINE_PREFIX {
            bail!("unexpected characters at start of line \"{line}\" (expected \"{LINE_PREFIX}\"")
        }

        let Some([coords, _size, used, available, _use_percent]) =
            after.split_ascii_whitespace().collect_array()
        else {
            bail!("failed to split line \"{line}\" into components")
        };

        let Some((x, y)) = coords.split_once("-") else {
            bail!("failed to split {coords} into x and y");
        };

        Ok(PositionedNode {
            x: x["x".len()..].parse()?,
            y: y["y".len()..].parse()?,
            used: used[..used.len() - "T".len()]
                .parse::<u16>()?
                .try_into()
                .unwrap_or(u8::MAX),
            available: available[..available.len() - "T".len()]
                .parse::<u16>()?
                .try_into()
                .unwrap_or(u8::MAX),
        })
    }
}
