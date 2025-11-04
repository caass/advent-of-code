use std::ops::{Index, IndexMut};
use std::str::FromStr;

use eyre::{OptionExt, Report, bail, eyre};

use aoc_meta::Problem;

/// <https://adventofcode.com/2015/day/23>
pub const OPENING_THE_TURING_LOCK: Problem = Problem::solved(
    &|input| {
        input.parse().map(|program: Program| {
            let mut computer = Computer::default();
            computer.run(&program);
            computer.b
        })
    },
    &|input| {
        input.parse().map(|program: Program| {
            let mut computer = Computer::new(1, 0);
            computer.run(&program);
            computer.b
        })
    },
);

#[derive(Default)]
struct Computer {
    a: usize,
    b: usize,
}

impl Computer {
    fn new(a: usize, b: usize) -> Computer {
        Computer { a, b }
    }
}

impl Index<Register> for Computer {
    type Output = usize;

    #[inline]
    fn index(&self, index: Register) -> &Self::Output {
        match index {
            Register::A => &self.a,
            Register::B => &self.b,
        }
    }
}

impl IndexMut<Register> for Computer {
    #[inline]
    fn index_mut(&mut self, index: Register) -> &mut Self::Output {
        match index {
            Register::A => &mut self.a,
            Register::B => &mut self.b,
        }
    }
}

impl Computer {
    fn run(&mut self, program: &Program) {
        let mut cursor = 0isize;
        while let Some(instruction) = cursor
            .try_into()
            .ok()
            .and_then(|idx: usize| program.instructions.get(idx))
            .copied()
        {
            let offset = self.execute(instruction);
            cursor += offset;
        }
    }

    fn execute(&mut self, instruction: Instruction) -> isize {
        match instruction {
            Instruction::Half(register) => self[register] /= 2,
            Instruction::Triple(register) => self[register] *= 3,
            Instruction::Increment(register) => self[register] += 1,
            Instruction::Jump { offset } => return offset.into(),
            Instruction::JumpIfEven { register, offset } => {
                if self[register].is_multiple_of(2) {
                    return offset.into();
                }
            }
            Instruction::JumpIfOne { register, offset } => {
                if self[register] == 1 {
                    return offset.into();
                }
            }
        };

        1
    }
}

#[derive(Debug, Clone, Copy)]
enum Register {
    A,
    B,
}

impl FromStr for Register {
    type Err = Report;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "a" => Ok(Self::A),
            "b" => Ok(Self::B),
            _ => Err(eyre!("Invalid register name {s}, expected one of 'a', 'b'")),
        }
    }
}

struct Program {
    instructions: Vec<Instruction>,
}

impl FromIterator<Instruction> for Program {
    #[inline]
    fn from_iter<T: IntoIterator<Item = Instruction>>(iter: T) -> Self {
        Self {
            instructions: Vec::from_iter(iter),
        }
    }
}

impl FromStr for Program {
    type Err = Report;

    #[inline]
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        s.trim()
            .lines()
            .map(str::trim)
            .map(Instruction::from_str)
            .collect()
    }
}

#[derive(Debug, Clone, Copy)]
enum Instruction {
    Half(Register),
    Triple(Register),
    Increment(Register),
    Jump { offset: i8 },
    JumpIfEven { register: Register, offset: i8 },
    JumpIfOne { register: Register, offset: i8 },
}

impl FromStr for Instruction {
    type Err = Report;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut parts = s.split([' ', ',']).filter(|s| !s.is_empty());
        let a = parts
            .next()
            .ok_or_eyre("Cannot parse instruction from empty input")?;
        let b = parts
            .next()
            .ok_or_eyre("Cannot parse instruction without target register/offset")?;
        let c = parts.next();

        debug_assert!(parts.next().is_none());

        match a {
            "hlf" => {
                if let Some(unexpected) = c {
                    bail!("Unexpected third term in `hlf` instruction: {unexpected}");
                }
                b.parse().map(Instruction::Half)
            }
            "tpl" => {
                if let Some(unexpected) = c {
                    bail!("Unexpected third term in `tpl` instruction: {unexpected}");
                }
                b.parse().map(Instruction::Triple)
            }
            "inc" => {
                if let Some(unexpected) = c {
                    bail!("Unexpected third term in `inc` instruction: {unexpected}");
                }
                b.parse().map(Instruction::Increment)
            }
            "jmp" => {
                if let Some(unexpected) = c {
                    bail!("Unexpected third term in `jmp` instruction: {unexpected}");
                }

                Ok(Instruction::Jump { offset: b.parse()? })
            }
            "jie" => {
                let register = b.parse()?;
                let offset = c
                    .ok_or_eyre("missing offset in `jie` instruction")?
                    .parse()?;
                Ok(Instruction::JumpIfEven { register, offset })
            }
            "jio" => {
                let register = b.parse()?;
                let offset = c
                    .ok_or_eyre("missing offset in `jio` instruction")?
                    .parse()?;
                Ok(Instruction::JumpIfOne { register, offset })
            }
            _ => bail!("Unknown instruction `{a}`"),
        }
    }
}
