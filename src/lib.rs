use phf::phf_map;

pub(crate) mod common;
pub mod meta;

use meta::AdventOfCode;

#[path = "2015/mod.rs"]
mod _2015;

pub const AOC: AdventOfCode = AdventOfCode(phf_map! {
    2015u16 => _2015::PROBLEMS,
});
