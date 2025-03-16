use std::cmp::Ordering;
use std::fmt::Display;
use std::str::FromStr;

use eyre::{Result, bail, eyre};
use nohash_hasher::IntMap;
use winnow::ascii::{alpha1, digit1};
use winnow::combinator::{dispatch, fail, seq, terminated};
use winnow::prelude::*;

use aoc_meta::Problem;

pub const BALANCE_BOTS: Problem = Problem::solved(
    &|input| {
        let goal = Comparison::new(61, 17)?;
        let instructions = input.lines().map(Instruction::from_str);

        Factory::new(goal).with_instructions(instructions)?.run()
    },
    &|input| {
        let goal = [0, 1, 2];
        let instructions = input.lines().map(Instruction::from_str);

        Factory::new(goal).with_instructions(instructions)?.run()
    },
);

#[derive(Debug)]
struct Factory<G> {
    robots: IntMap<u8, Robot>,
    actions: Vec<Action>,
    outputs: IntMap<u8, u8>,
    goal: G,
}

impl<G: Goal> Factory<G> {
    fn new(goal: G) -> Self {
        Self {
            robots: IntMap::default(),
            actions: Vec::default(),
            outputs: IntMap::default(),
            goal,
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

    fn tick(&mut self) -> Result<Option<G::Output>> {
        for robot in &mut self.robots.values_mut() {
            robot.queue(&mut self.actions)?;
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
                Destination::Output(id) => {
                    if self.outputs.insert(id, chip).is_some() {
                        bail!("inserted microchip into already-full output")
                    }
                }
            }
        }

        Ok(self.goal.check(self))
    }

    fn run(&mut self) -> Result<G::Output> {
        loop {
            if let Some(output) = self.tick()? {
                return Ok(output);
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
    fn queue(&mut self, actions: &mut Vec<Action>) -> Result<()> {
        let Hands::Full(Comparison {
            high: hi_chip,
            low: lo_chip,
        }) = self.hands
        else {
            return Ok(());
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

        Ok(())
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

trait Goal {
    type Output: Display;

    fn check(&self, factory: &Factory<Self>) -> Option<Self::Output>
    where
        Self: Sized;
}

impl Goal for Comparison {
    type Output = u8;

    fn check(&self, factory: &Factory<Self>) -> Option<Self::Output> {
        factory.robots.iter().find_map(|(&id, robot)| {
            matches!(robot.hands, Hands::Full(comparison) if comparison == *self).then_some(id)
        })
    }
}

impl<T: AsRef<[u8]>> Goal for T {
    type Output = usize;

    fn check(&self, factory: &Factory<Self>) -> Option<Self::Output>
    where
        Self: Sized,
    {
        let this = self.as_ref();

        let mut product = 1;
        let mut num_outputs_left = this.len();

        for (id, &output) in &factory.outputs {
            if this.contains(id) {
                num_outputs_left -= 1;
                product *= usize::from(output);
            }
        }

        if num_outputs_left == 0 {
            Some(product)
        } else {
            None
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

fn parse_instruction(input: &mut &str) -> ModalResult<Instruction> {
    dispatch! {terminated(alpha1, ' ');
        "value" => parse_startup_instruction,
        "bot" => parse_bot_instruction,
        _ => fail
    }
    .parse_next(input)
}

fn parse_startup_instruction(input: &mut &str) -> ModalResult<Instruction> {
    seq! {
        Instruction::Startup {
            microchip: digit1.parse_to(),
            _: " goes to bot ",
            bot: digit1.parse_to()
        }
    }
    .parse_next(input)
}

fn parse_bot_instruction(input: &mut &str) -> ModalResult<Instruction> {
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

fn parse_destination(input: &mut &str) -> ModalResult<Destination> {
    dispatch! {terminated(alpha1, ' ');
        "bot" => digit1.parse_to().map(Destination::Bot),
        "output" => digit1.parse_to().map(Destination::Output),
        _ => fail
    }
    .parse_next(input)
}
