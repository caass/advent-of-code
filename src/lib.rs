pub(crate) mod common;
pub mod meta;
#[cfg(target_family = "wasm")]
mod wasm;

use meta::AOC;
#[cfg(target_family = "wasm")]
pub use wasm::solve;

AOC!([2015, 2016]);
