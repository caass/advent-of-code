use std::{
    fmt::{self, Display, Formatter, Write},
    str::FromStr,
};

use eyre::{bail, eyre, Report, Result};
use rayon::prelude::*;
use winnow::{
    ascii::dec_uint,
    combinator::{alt, preceded, separated_pair},
    prelude::*,
};

use crate::meta::Problem;

pub const TWO_FACTOR_AUTHENTICATION: Problem = Problem::solved(
    &|input| {
        let mut screen: Screen<50, 6> = Screen::default();

        for line in input.lines() {
            let instruction = line.parse()?;
            screen.apply(&instruction);
        }

        Ok::<_, Report>(screen.lit_pixels())
    },
    &|input| {
        let mut screen: Screen<50, 6> = Screen::default();

        for line in input.lines() {
            let instruction = line.parse()?;
            screen.apply(&instruction);
        }

        screen.message()
    },
);

#[derive(Debug)]
struct Screen<const W: usize, const H: usize>([[bool; H]; W]);

impl<const W: usize, const H: usize> Display for Screen<W, H> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        for y in 0..H {
            for x in 0..W {
                f.write_char(if self.0[x][y] { '#' } else { '.' })?;
            }

            if y < H - 1 {
                writeln!(f)?;
            }
        }

        Ok(())
    }
}

impl<const W: usize, const H: usize> Default for Screen<W, H> {
    fn default() -> Self {
        Self([[false; H]; W])
    }
}

impl<const W: usize, const H: usize> Screen<W, H> {
    fn apply(&mut self, instruction: &Instruction) {
        match *instruction {
            Instruction::Rect { width, height } => self.rect(width, height),
            Instruction::RotateRow { y, rotation } => self.rotate_row(y, rotation),
            Instruction::RotateCol { x, rotation } => self.rotate_col(x, rotation),
        }
    }

    fn lit_pixels(&self) -> usize {
        self.0
            .par_iter()
            .flatten()
            .filter(|is_lit| **is_lit)
            .count()
    }

    fn rect(&mut self, width: usize, height: usize) {
        self.0[..width]
            .iter_mut()
            .for_each(|col| col[..height].fill(true));
    }

    fn rotate_col(&mut self, x: usize, rotation: usize) {
        let signed_h: isize = H
            .try_into()
            .expect("screen to be small enough to fit in `isize::MAX`");
        let signed_rot: isize = rotation
            .try_into()
            .expect("rotation to be small enough to fit in `isize::MAX`");

        let new_col: [bool; H] = std::array::from_fn(|new_y| {
            // Safety: `new_y` is strictly less than `H`, which we know fits in `isize` thanks to `signed_h`
            let signed_new_y: isize = unsafe { new_y.try_into().unwrap_unchecked() };
            let signed_old_y = (signed_new_y - signed_rot).rem_euclid(signed_h);

            // Safety: the result of `.rem_euclid` is always positive.
            let old_y: usize = unsafe { signed_old_y.try_into().unwrap_unchecked() };

            self.0[x][old_y]
        });

        self.0[x] = new_col;
    }

    fn rotate_row(&mut self, y: usize, rotation: usize) {
        let signed_w: isize = W
            .try_into()
            .expect("screen to be small enough to fit in `isize::MAX`");
        let signed_rot: isize = rotation
            .try_into()
            .expect("rotation to be small enough to fit in `isize::MAX`");

        let new_row: [bool; W] = std::array::from_fn(|new_x| {
            // Safety: `i` is strictly less than `W`, which we know fits in `isize` thanks to `signed_w`
            let signed_new_x: isize = unsafe { new_x.try_into().unwrap_unchecked() };
            let signed_old_x = (signed_new_x - signed_rot).rem_euclid(signed_w);

            // Safety: the result of `.rem_euclid` is always positive.
            let old_x: usize = unsafe { signed_old_x.try_into().unwrap_unchecked() };

            self.0[old_x][y]
        });

        for (old_value, new_value) in self.0.iter_mut().map(|col| &mut col[y]).zip(new_row) {
            *old_value = new_value;
        }
    }
}

impl<const W: usize, const H: usize> From<[[bool; H]; W]> for Screen<W, H> {
    fn from(inner: [[bool; H]; W]) -> Self {
        Self(inner)
    }
}

impl<const W: usize> Screen<W, 6> {
    fn message(&self) -> Result<String> {
        let chunks = self.0.chunks_exact(5);
        if !chunks.remainder().is_empty() {
            bail!("Grid width isn't a multiple of 5, cannot read message!");
        }

        let mut message = String::with_capacity(chunks.len());
        for chunk in chunks {
            // Safety: the chunks are created with length of exactly 5
            let grid: [[bool; 6]; 5] = unsafe { chunk.try_into().unwrap_unchecked() };
            let letter = Screen::from(grid).letter()?;
            message.push(letter);
        }

        Ok(message)
    }
}

impl Screen<5, 6> {
    fn letter(&self) -> Result<char> {
        macro_rules! LETTERS {
            ($($letter:ident),+) => {$(
                const $letter: &str = include_str!(concat!(
                    env!("CARGO_MANIFEST_DIR"),
                    ::pathsep::path_separator!(),
                    "fixtures",
                    ::pathsep::path_separator!(),
                    "2016",
                    ::pathsep::path_separator!(),
                    "08",
                    ::pathsep::path_separator!(),
                    stringify!($letter)
                ));
            )+};
        }

        LETTERS!(B, C, E, F, I, J, K, L, O, R, U, Y, Z);

        match self.to_string().as_str() {
            B => Ok('B'),
            C => Ok('C'),
            E => Ok('E'),
            F => Ok('F'),
            I => Ok('I'),
            J => Ok('J'),
            K => Ok('K'),
            L => Ok('L'),
            O => Ok('O'),
            R => Ok('R'),
            U => Ok('U'),
            Y => Ok('Y'),
            Z => Ok('Z'),
            other => bail!("Don't know what letter this is:\n\n{other}\n"),
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
enum Instruction {
    Rect { width: usize, height: usize },
    RotateRow { y: usize, rotation: usize },
    RotateCol { x: usize, rotation: usize },
}

impl FromStr for Instruction {
    type Err = Report;

    fn from_str(s: &str) -> Result<Self> {
        alt((parse_rect, parse_rotate_col, parse_rotate_row))
            .parse(s)
            .map_err(|_| eyre!("Invalid instruction: {s}"))
    }
}

fn parse_rotate_row(input: &mut &str) -> PResult<Instruction> {
    (
        preceded("rotate row y=", dec_uint),
        preceded(" by ", dec_uint),
    )
        .map(|(y, rotation)| Instruction::RotateRow { y, rotation })
        .parse_next(input)
}

fn parse_rotate_col(input: &mut &str) -> PResult<Instruction> {
    (
        preceded("rotate column x=", dec_uint),
        preceded(" by ", dec_uint),
    )
        .map(|(x, rotation)| Instruction::RotateCol { x, rotation })
        .parse_next(input)
}

fn parse_rect(input: &mut &str) -> PResult<Instruction> {
    preceded(
        "rect ",
        separated_pair(dec_uint, "x", dec_uint)
            .map(|(width, height)| Instruction::Rect { width, height }),
    )
    .parse_next(input)
}

#[cfg(test)]
mod test {
    use itertools::Itertools;

    use super::*;

    #[test]
    fn test_parse_rect() {
        let parsed = parse_rect.parse("rect 3x2").unwrap();
        assert_eq!(
            parsed,
            Instruction::Rect {
                width: 3,
                height: 2
            }
        );
    }

    #[test]
    fn test_parse_rotate_col() {
        let parsed = parse_rotate_col.parse("rotate column x=2 by 1").unwrap();
        assert_eq!(parsed, Instruction::RotateCol { x: 2, rotation: 1 });
    }

    #[test]
    fn test_parse_rotate_row() {
        let parsed = parse_rotate_row.parse("rotate row y=0 by 4").unwrap();
        assert_eq!(parsed, Instruction::RotateRow { y: 0, rotation: 4 });
    }

    #[test]
    fn formatting() {
        let screen = Screen([
            [false, true, false],
            [true, false, true],
            [false, true, false],
            [false, false, false],
            [true, false, false],
            [false, false, false],
            [true, false, false],
        ]);

        let expected: String = Itertools::intersperse(
            ".#..#.#
             #.#....
             .#....."
                .trim()
                .lines()
                .map(str::trim),
            "\n",
        )
        .collect();

        assert_eq!(screen.to_string(), expected);
    }

    #[test]
    fn example_1() {
        let mut screen: Screen<7, 3> = Screen::default();

        assert_eq!(
            screen.to_string(),
            Itertools::intersperse(
                ".......
                 .......
                 ......."
                    .trim()
                    .lines()
                    .map(str::trim),
                "\n",
            )
            .collect::<String>()
        );

        let rect_3_2 = "rect 3x2".parse().unwrap();
        screen.apply(&rect_3_2);

        assert_eq!(
            screen.to_string(),
            Itertools::intersperse(
                "###....
                 ###....
                 ......."
                    .trim()
                    .lines()
                    .map(str::trim),
                "\n",
            )
            .collect::<String>()
        );

        let rot_col_1_1 = "rotate column x=1 by 1".parse().unwrap();
        screen.apply(&rot_col_1_1);

        assert_eq!(
            screen.to_string(),
            Itertools::intersperse(
                "#.#....
                 ###....
                 .#....."
                    .trim()
                    .lines()
                    .map(str::trim),
                "\n",
            )
            .collect::<String>()
        );

        let rot_row_0_4 = "rotate row y=0 by 4".parse().unwrap();
        screen.apply(&rot_row_0_4);

        assert_eq!(
            screen.to_string(),
            Itertools::intersperse(
                "....#.#
                 ###....
                 .#....."
                    .trim()
                    .lines()
                    .map(str::trim),
                "\n",
            )
            .collect::<String>()
        );

        let rot_col_1_1 = "rotate column x=1 by 1".parse().unwrap();
        screen.apply(&rot_col_1_1);

        assert_eq!(
            screen.to_string(),
            Itertools::intersperse(
                ".#..#.#
                 #.#....
                 .#....."
                    .trim()
                    .lines()
                    .map(str::trim),
                "\n",
            )
            .collect::<String>()
        );
    }
}
