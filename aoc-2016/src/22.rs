use std::str::FromStr;

use aoc_meta::Problem;
use eyre::{OptionExt, Report, Result, bail};
use itertools::Itertools;
use rayon::prelude::*;

pub const GRID_COMPUTING: Problem = Problem::solved(
    &|input| input.parse().map(StorageCluster::viable_pairs),
    &|input| input.parse().map(StorageCluster::minimum_steps),
);

impl StorageCluster {
    fn viable_pairs(self) -> usize {
        self.nodes
            .par_iter()
            .copied()
            .filter(|a| a.used > 0)
            .flat_map(|a| {
                self.nodes
                    .par_iter()
                    .copied()
                    .filter(move |&b| !(a.x == b.x && a.y == b.y) && a.used <= b.available)
            })
            .count()
    }

    fn minimum_steps(self) -> u16 {
        let empty = self.nodes[self.empty_idx];
        let target = self.nodes[self.target_idx];

        u16::from(empty.x + empty.y + target.x - 1) + u16::from(5 * target.x - 4)
    }
}

#[derive(Debug)]
struct StorageCluster {
    target_idx: usize,
    empty_idx: usize,
    nodes: Vec<PositionedNode>,
}

impl FromStr for StorageCluster {
    type Err = Report;

    fn from_str(s: &str) -> Result<Self> {
        let (_, after) = s
            .split_once("Use%\n")
            .ok_or_eyre("couldn't find \"Use%\\n\" in input")?;

        let mut max_x = 0;
        let mut empty_idx = 0;
        let mut found_empty = false;

        let nodes = after
            .lines()
            .map(PositionedNode::from_str)
            .enumerate()
            .inspect(|(i, res)| {
                let Ok(node) = res else {
                    return;
                };

                if node.used == 0 {
                    found_empty = true;
                    empty_idx = *i;
                }

                max_x = max_x.max(node.x)
            })
            .map(|(_, res)| res)
            .collect::<Result<Vec<PositionedNode>>>()?;

        if !found_empty {
            bail!("cannot move target data")
        }

        let target_idx = nodes
            .iter()
            .position(|node| node.x == max_x && node.y == 0)
            .ok_or_eyre("no target data")?;

        Ok(Self {
            target_idx,
            empty_idx,
            nodes,
        })
    }
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
