use std::cmp::Ordering;
use std::str::FromStr;

use eyre::{bail, eyre, Result};
use nohash_hasher::IntMap;
use winnow::ascii::{alpha1, digit1};
use winnow::combinator::{dispatch, fail, seq, terminated};
use winnow::prelude::*;

use crate::meta::Problem;

pub const BALANCE_BOTS: Problem = Problem::partially_solved(&|input| {
    Factory::new(Comparison::new(61, 17)?)
        .with_instructions(input.lines().map(Instruction::from_str))?
        .run()
});

#[derive(Debug)]
struct Factory {
    robots: IntMap<u8, Robot>,
    actions: Vec<Action>,
    outputs: IntMap<u8, Vec<u8>>,
    target: Comparison,
}

impl Factory {
    fn new(target: Comparison) -> Self {
        Self {
            robots: IntMap::default(),
            actions: Vec::default(),
            outputs: IntMap::default(),
            target,
        }
    }

    fn with_instructions(
        &mut self,
        instructions: impl IntoIterator<Item = Result<Instruction>>,
    ) -> Result<&mut Self> {
        for instruction in instructions {
            match instruction? {
                Instruction::Startup { microchip, bot } => {
                    self.robots.entry(bot).or_default().take(microchip)
                }
                Instruction::Comparison { bot, high, low } => self
                    .robots
                    .entry(bot)
                    .or_default()
                    .target(Targets { low, high }),
            }?;
        }

        Ok(self)
    }
}

impl Factory {
    fn tick(&mut self) -> Result<Option<u8>> {
        let mut comparator = None;
        for (&id, robot) in &mut self.robots {
            if robot.queue(&mut self.actions, self.target)? {
                comparator = Some(id);
            }
        }

        if self.actions.is_empty() {
            bail!("action queue was empty");
        }

        for Action { chip, to } in self.actions.drain(..) {
            match to {
                Destination::Bot(id) => {
                    if let Some(bot) = self.robots.get_mut(&id) {
                        bot.take(chip)?;
                    }
                }
                Destination::Output(id) => self.outputs.entry(id).or_default().push(chip),
            }
        }

        Ok(comparator)
    }

    fn run(&mut self) -> Result<u8> {
        loop {
            if let Some(id) = self.tick()? {
                return Ok(id);
            }
        }
    }
}

#[derive(Debug, Default)]
struct Robot {
    targets: Option<Targets>,
    hands: Hands,
}

impl Robot {
    fn queue(&mut self, actions: &mut Vec<Action>, target: Comparison) -> Result<bool> {
        let Hands::Full(Comparison {
            high: hi_chip,
            low: lo_chip,
        }) = self.hands
        else {
            return Ok(false);
        };

        self.hands = Hands::Empty;
        let Some(Targets {
            low: lo_dest,
            high: hi_dest,
        }) = self.targets
        else {
            bail!("bot's hands were full, but had no targets for chips");
        };

        actions.push(Action {
            chip: hi_chip,
            to: hi_dest,
        });
        actions.push(Action {
            chip: lo_chip,
            to: lo_dest,
        });

        Ok(target.high == hi_chip && target.low == lo_chip)
    }

    fn take(&mut self, a: u8) -> Result<()> {
        self.hands = match self.hands {
            Hands::Empty => Hands::PartiallyFull(a),
            Hands::PartiallyFull(b) => Hands::Full(Comparison::new(a, b)?),
            Hands::Full(_) => bail!("tried to hold a third microchip"),
        };

        Ok(())
    }

    fn target(&mut self, targets: Targets) -> Result<()> {
        if self.targets.is_some() {
            bail!("Already had targets set for bot");
        }

        self.targets = Some(targets);

        Ok(())
    }
}

#[derive(Debug, Default)]
enum Hands {
    #[default]
    Empty,
    PartiallyFull(u8),
    Full(Comparison),
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
struct Targets {
    low: Destination,
    high: Destination,
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
struct Action {
    chip: u8,
    to: Destination,
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
enum Destination {
    Bot(u8),
    Output(u8),
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
struct Comparison {
    high: u8,
    low: u8,
}

impl Comparison {
    fn new(a: u8, b: u8) -> Result<Self> {
        match a.cmp(&b) {
            Ordering::Less => Ok(Self { high: b, low: a }),
            Ordering::Equal => Err(eyre!("Got handed two of the same microchip")),
            Ordering::Greater => Ok(Self { high: a, low: b }),
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
enum Instruction {
    Startup {
        microchip: u8,
        bot: u8,
    },
    Comparison {
        bot: u8,
        high: Destination,
        low: Destination,
    },
}

impl FromStr for Instruction {
    type Err = eyre::Report;

    fn from_str(s: &str) -> Result<Self> {
        parse_instruction.parse(s).map_err(|e| eyre!("{e}"))
    }
}

fn parse_instruction(input: &mut &str) -> PResult<Instruction> {
    dispatch! {terminated(alpha1, ' ');
        "value" => parse_startup_instruction,
        "bot" => parse_bot_instruction,
        _ => fail
    }
    .parse_next(input)
}

fn parse_startup_instruction(input: &mut &str) -> PResult<Instruction> {
    seq! {
        Instruction::Startup {
            microchip: digit1.parse_to(),
            _: " goes to bot ",
            bot: digit1.parse_to()
        }
    }
    .parse_next(input)
}

fn parse_bot_instruction(input: &mut &str) -> PResult<Instruction> {
    seq! {
        Instruction::Comparison {
            bot: digit1.parse_to(),
            _: " gives low to ",
            low: parse_destination,
            _: " and high to ",
            high: parse_destination
        }
    }
    .parse_next(input)
}

fn parse_destination(input: &mut &str) -> PResult<Destination> {
    dispatch! {terminated(alpha1, ' ');
        "bot" => digit1.parse_to().map(Destination::Bot),
        "output" => digit1.parse_to().map(Destination::Output),
        _ => fail
    }
    .parse_next(input)
}
