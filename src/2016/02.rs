use std::fmt::{self, Display, Formatter};
use std::str::FromStr;

use eyre::{bail, eyre, Report, Result};

use crate::meta::Problem;

pub const BATHROOM_SECURITY: Problem = Problem::solved(
    &|input| InstructionSet::from(input).follow(true),
    &|input| InstructionSet::from(input).follow(false),
);

#[derive(Debug, Default)]
struct Code(Vec<Digit>);

impl Code {
    #[inline]
    fn push(&mut self, digit: Digit) {
        self.0.push(digit);
    }
}

impl FromIterator<Digit> for Code {
    fn from_iter<T: IntoIterator<Item = Digit>>(iter: T) -> Self {
        Self(Vec::from_iter(iter))
    }
}

impl Display for Code {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        self.0.iter().try_for_each(|digit| Display::fmt(digit, f))
    }
}

#[derive(Debug, Default, Clone, Copy)]
enum Digit {
    One,
    Two,
    Three,
    Four,
    #[default]
    Five,
    Six,
    Seven,
    Eight,
    Nine,

    // Physical buttons
    A,
    B,
    C,
    D,
}

impl Display for Digit {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        Display::fmt(
            &match self {
                Digit::One => '1',
                Digit::Two => '2',
                Digit::Three => '3',
                Digit::Four => '4',
                Digit::Five => '5',
                Digit::Six => '6',
                Digit::Seven => '7',
                Digit::Eight => '8',
                Digit::Nine => '9',
                Digit::A => 'A',
                Digit::B => 'B',
                Digit::C => 'C',
                Digit::D => 'D',
            },
            f,
        )
    }
}

impl Digit {
    fn mental_next(&self, moove: Move) -> Option<Self> {
        use Digit::*;
        use Move::*;

        // 1 2 3
        // 4 5 6
        // 7 8 9
        match (self, moove) {
            (One | Two | Three, Up)
            | (Seven | Eight | Nine, Down)
            | (One | Four | Seven, Left)
            | (Three | Six | Nine, Right) => None,

            (Four, Up) | (Two, Left) => Some(One),
            (Five, Up) | (One, Right) | (Three, Left) => Some(Two),
            (Six, Up) | (Two, Right) => Some(Three),

            (Seven, Up) | (One, Down) | (Five, Left) => Some(Four),
            (Eight, Up) | (Two, Down) | (Six, Left) | (Four, Right) => Some(Five),
            (Nine, Up) | (Three, Down) | (Five, Right) => Some(Six),

            (Four, Down) | (Eight, Left) => Some(Seven),
            (Five, Down) | (Nine, Left) | (Seven, Right) => Some(Eight),
            (Six, Down) | (Eight, Right) => Some(Nine),

            // Unreachable
            (A | B | C | D, _) => unreachable!("I didn't imagine there'd be these buttons..."),
        }
    }

    fn actual_next(&self, moove: Move) -> Option<Digit> {
        use Digit::*;
        use Move::*;

        //     1
        //   2 3 4
        // 5 6 7 8 9
        //   A B C
        //     D
        match (self, moove) {
            (One, Left | Right | Up)
            | (Two, Left | Up)
            | (Four, Up | Right)
            | (Five, Up | Down | Left)
            | (Nine, Up | Down | Right)
            | (A, Left | Down)
            | (C, Right | Down)
            | (D, Down | Left | Right) => None,

            (Three, Up) => Some(One),

            (Six, Up) | (Three, Left) => Some(Two),
            (Seven, Up) | (One, Down) | (Four, Left) | (Two, Right) => Some(Three),
            (Eight, Up) | (Three, Right) => Some(Four),

            (Six, Left) => Some(Five),
            (A, Up) | (Two, Down) | (Seven, Left) | (Five, Right) => Some(Six),
            (B, Up) | (Three, Down) | (Eight, Left) | (Six, Right) => Some(Seven),
            (C, Up) | (Four, Down) | (Nine, Left) | (Seven, Right) => Some(Eight),
            (Eight, Right) => Some(Nine),

            (Six, Down) | (B, Left) => Some(A),
            (D, Up) | (Seven, Down) | (C, Left) | (A, Right) => Some(B),
            (Eight, Down) | (B, Right) => Some(C),

            (B, Down) => Some(D),
        }
    }
}

#[derive(Debug)]
enum Move {
    Up,
    Down,
    Left,
    Right,
}

impl TryFrom<char> for Move {
    type Error = Report;

    fn try_from(ch: char) -> Result<Self> {
        match ch {
            'U' => Ok(Move::Up),
            'D' => Ok(Move::Down),
            'L' => Ok(Move::Left),
            'R' => Ok(Move::Right),
            _ => Err(eyre!(
                "Can only move (U)p, (D)own, (L)eft, and (R)ight -- not ({ch})"
            )),
        }
    }
}

impl FromStr for Move {
    type Err = Report;

    fn from_str(s: &str) -> Result<Self> {
        if s.len() != 1 {
            bail!("Instructions are 1 character long");
        };

        // Safety: we know the string has a length of 1
        let ch = unsafe { s.chars().next().unwrap_unchecked() };
        ch.try_into()
    }
}

struct MoveSet<'instructions>(&'instructions str);

impl MoveSet<'_> {
    fn next_digit(self, mut digit: Digit, imaginary: bool) -> Result<Digit> {
        for result in self {
            let moove = result?;
            if let Some(next) = if imaginary {
                digit.mental_next(moove)
            } else {
                digit.actual_next(moove)
            } {
                digit = next;
            }
        }

        Ok(digit)
    }
}

impl Iterator for MoveSet<'_> {
    type Item = Result<Move>;

    fn next(&mut self) -> Option<Self::Item> {
        let (next, rest) = self.0.split_at_checked(1)?;
        self.0 = rest;

        Some(next.parse())
    }
}

struct InstructionSet<'s>(std::str::Lines<'s>);

impl<'s> From<&'s str> for InstructionSet<'s> {
    fn from(s: &'s str) -> Self {
        Self(s.trim().lines())
    }
}

impl<'s> Iterator for InstructionSet<'s> {
    type Item = MoveSet<'s>;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next().map(MoveSet)
    }
}

impl InstructionSet<'_> {
    fn follow(self, imaginary: bool) -> Result<Code> {
        let mut digit = Digit::default();
        let mut code = Code::default();

        for instruction in self {
            digit = instruction.next_digit(digit, imaginary)?;
            code.push(digit);
        }

        Ok(code)
    }
}
