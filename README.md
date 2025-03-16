# Advent of Code

My solutions to [Eric Wastl](http://was.tl/)'s [Advent of Code](https://adventofcode.com/) :)

## Organization

This repo is a [Cargo workspace](https://doc.rust-lang.org/book/ch14-03-cargo-workspaces.html) containing multiple [crates](https://doc.rust-lang.org/book/ch07-01-packages-and-crates.html) that comprise my solutions. The workspace is organized as follows:

- [`aoc`](./aoc): top-level wrapper crate with a runnable binary.
- [`aoc-meta`](./aoc-meta): core types and traits for solving Advent of Code problems, such as [`Part`](aoc-meta/src/indices/part.rs) (Advent of Code problems tend to have two parts).
- [`aoc-common`](./aoc-common): types and traits common between multiple problems, such as [`TryFromStr`](aoc-common/src/from_str_ext.rs) (which allows parsing string slices into structs containing references to the original slice).
- [`aoc-2015`](./aoc-2015), [`aoc-2016`](./aoc-2016), [`aoc-2017`](./aoc-2017), [`aoc-2018`](./aoc-2018), [`aoc-2019`](./aoc-2019), [`aoc-2020`](./aoc-2020), [`aoc-2021`](./aoc-2021), [`aoc-2022`](./aoc-2022), [`aoc-2023`](./aoc-2023), and [`aoc-2024`](./aoc-2024): contain implementations of the Advent of Code problems from each respective year.

## Inputs & Descriptions

To comply with Eric's [rules about copying](https://adventofcode.com/about#faq_copying), I don't copy any text from the problems in my solutions or provide my puzzle inputs.

## Running

To run the `aoc` binary, run `just run <YEAR> <DAY> <PART>`. You can provide your puzzle input via stdin (default) or by providing a fourth argument containing the filepath of your input.
