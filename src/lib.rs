pub(crate) mod common;
pub mod meta;

use meta::{AdventOfCode, Year};

#[path = "2015/mod.rs"]
mod year2015;

pub const AOC: AdventOfCode = AdventOfCode::new().with_year(Year::Fifteen, year2015::PROBLEMS);
