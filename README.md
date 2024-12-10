# Advent of Code

My solutions to [Eric Wastl](http://was.tl/)'s [Advent of Code](https://adventofcode.com/) :)

## Organization

This repo is a single rust crate, `advent-of-code`, and is organized mostly according to those conventions; my solutions are located in `src` and organized by year and day, so you can find my solution to e.g. [day 9 of 2015](https://adventofcode.com/2015/day/9) in [`src/2015/09.rs`](./src/2015/09.rs). Each solution is tested in [`tests/integration.rs`](./tests/integration.rs) (via some write-only `macro_rules!` magic) and additionally there's some unit tests for certain parts of certain days that had more complex internal logic; those tests are colocated in the source files alongside the logic that they're testing.

The [`meta`](./src/meta) subfolder in [`src`](./src) contains things that are more structural to advent of code, rather than anything related to a specific problem. The [`common`](./src/common) subfoleder contains types and traits that are useful for more than one problem, e.g. 2015's days [6](./src/2015/06.rs) and [18](./src/2015/18.rs).

## Inputs & Descriptions

To comply with Eric's [rules about copying](https://adventofcode.com/about#faq_copying), I don't copy any text from the problems in my solutions or provide my puzzle inputs.

## Running

The crate also has a binary, which you can run with `just run <YEAR> <DAY> <PART> < path/to/input`. To see all available commands, run `just` with no arguments.
