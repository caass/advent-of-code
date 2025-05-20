use std::{ops, str::FromStr};

use either::Either;
use eyre::{Report, eyre};
use rayon::prelude::*;
use winnow::{
    ascii::{alpha1, dec_int},
    combinator::{alt, dispatch, empty, fail, separated_pair, terminated},
    prelude::*,
    token::any,
};

use aoc_meta::Problem;

pub const LEONARDOS_MONORAIL: Problem = Problem::solved(
    &|input| parse_and_solve(Computer::default(), input),
    &|input| {
        parse_and_solve(
            Computer {
                c: Register { value: 1 },
                ..Default::default()
            },
            input,
        )
    },
);

fn parse_and_solve(mut computer: Computer, input: &str) -> Result<isize, Report> {
    let instructions = input
        .par_lines()
        .map(Instruction::from_str)
        .collect::<Result<Vec<_>, _>>()?;

    computer.execute(&instructions);

    Ok(computer.a.value)
}

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Hash, Default)]
struct Computer {
    instruction_pointer: usize,

    a: Register,
    b: Register,
    c: Register,
    d: Register,
}

impl ops::Index<RegisterName> for Computer {
    type Output = Register;

    fn index(&self, index: RegisterName) -> &Self::Output {
        match index {
            RegisterName::A => &self.a,
            RegisterName::B => &self.b,
            RegisterName::C => &self.c,
            RegisterName::D => &self.d,
        }
    }
}

impl ops::IndexMut<RegisterName> for Computer {
    fn index_mut(&mut self, index: RegisterName) -> &mut Self::Output {
        match index {
            RegisterName::A => &mut self.a,
            RegisterName::B => &mut self.b,
            RegisterName::C => &mut self.c,
            RegisterName::D => &mut self.d,
        }
    }
}

impl Computer {
    fn execute(&mut self, instructions: &[Instruction]) {
        while self.instruction_pointer < instructions.len() {
            instructions[self.instruction_pointer].execute(self)
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Hash, Default)]
struct Register {
    value: isize,
}

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Hash)]
enum RegisterName {
    A,
    B,
    C,
    D,
}

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Hash)]
enum Instruction {
    Copy {
        source: Either<RegisterName, isize>,
        target: RegisterName,
    },
    Increment {
        target: RegisterName,
    },
    Decrement {
        target: RegisterName,
    },
    JumpIfNotZero {
        condition: Either<RegisterName, isize>,
        distance: Either<RegisterName, isize>,
    },
}

impl Instruction {
    fn execute(self, computer: &mut Computer) {
        match self {
            Instruction::Copy { source, target } => {
                computer[target] = match source {
                    Either::Left(register) => computer[register],
                    Either::Right(value) => Register { value },
                }
            }
            Instruction::Increment { target } => computer[target].value += 1,
            Instruction::Decrement { target } => computer[target].value -= 1,
            Instruction::JumpIfNotZero {
                condition,
                distance,
            } => {
                let should_jump = match condition {
                    Either::Left(register) => computer[register].value,
                    Either::Right(value) => value,
                } != 0;

                if should_jump {
                    let distance_value = match distance {
                        Either::Left(register) => computer[register].value,
                        Either::Right(value) => value,
                    };

                    let op = if distance_value < 0 {
                        ops::SubAssign::sub_assign
                    } else {
                        ops::AddAssign::add_assign
                    };

                    return op(
                        &mut computer.instruction_pointer,
                        distance_value.unsigned_abs(),
                    );
                }
            }
        }

        computer.instruction_pointer += 1;
    }
}

impl FromStr for Instruction {
    type Err = Report;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        fn parse_register_name(input: &mut &str) -> ModalResult<RegisterName> {
            dispatch! {any;
              'a' => empty.value(RegisterName::A),
              'b' => empty.value(RegisterName::B),
              'c' => empty.value(RegisterName::C),
              'd' => empty.value(RegisterName::D),
              _ => fail
            }
            .parse_next(input)
        }

        fn parse_either(input: &mut &str) -> ModalResult<Either<RegisterName, isize>> {
            alt((
                parse_register_name.map(Either::Left),
                dec_int.map(Either::Right),
            ))
            .parse_next(input)
        }

        fn parse_instruction(input: &mut &str) -> ModalResult<Instruction> {
            dispatch!{terminated(alpha1, ' ');
                "cpy" => separated_pair(parse_either, ' ', parse_register_name).map(|(source, target)| Instruction::Copy { source, target }),
                "inc" => parse_register_name.map(|target| Instruction::Increment { target }),
                "dec" => parse_register_name.map(|target| Instruction::Decrement { target }),
                "jnz" => separated_pair(parse_either, ' ', parse_either).map(|(condition, distance)| Instruction::JumpIfNotZero { condition, distance }),
                _ => fail,
            }.parse_next(input)
        }

        parse_instruction.parse(s).map_err(|e| eyre!("{e}"))
    }
}
