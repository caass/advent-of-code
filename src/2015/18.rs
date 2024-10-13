use std::str::FromStr;
use std::sync::OnceLock;

use eyre::{eyre, OptionExt, Report, Result};
use rayon::prelude::*;

use crate::common::grid::{Coordinate, Grid};
use crate::meta::Problem;

pub const PROBLEM: Problem = Problem::solved(
    &|input: &str| {
        let mut lights = input.parse::<LightSet<100>>()?;
        lights.play(100)?;
        Ok::<_, Report>(lights.num_lit())
    },
    &|input: &str| {
        let mut lights = input.parse::<LightSet<100>>()?;
        lights.notice_breakage();
        lights.play(100)?;
        Ok::<_, Report>(lights.num_lit())
    },
);

struct LightSet<const N: usize> {
    grid: Grid<N, Light>,
    is_broken: bool,
}

impl<const N: usize> LightSet<N> {
    fn notice_breakage(&mut self) {
        self.is_broken = true;

        self.grid[Coordinate { x: 0, y: 0 }].turn_on();
        self.grid[Coordinate { x: 0, y: 99 }].turn_on();
        self.grid[Coordinate { x: 99, y: 0 }].turn_on();
        self.grid[Coordinate { x: 99, y: 99 }].turn_on();
    }

    fn compute_next_frame(&self) {
        self.grid.par_range(..).for_each(|(coordinate, light)| {
            light.next.get_or_init(|| {
                if self.is_broken
                    && matches!(
                        coordinate,
                        Coordinate {
                            x: 0 | 99,
                            y: 0 | 99
                        }
                    )
                {
                    LightState::On
                } else {
                    let num_lit_neighbors = self
                        .grid
                        .neighbors(coordinate)
                        .filter(|light| light.is_on())
                        .count();
                    match (light.state, num_lit_neighbors) {
                        (LightState::On, 2 | 3) | (LightState::Off, 3) => LightState::On,
                        _ => LightState::Off,
                    }
                }
            });
        });
    }

    fn step(&mut self) -> Result<()> {
        self.compute_next_frame();
        self.grid.par_iter_mut().try_for_each(Light::try_step)
    }

    fn play(&mut self, steps: usize) -> Result<()> {
        for _ in 0..steps {
            self.step()?;
        }

        Ok(())
    }

    fn num_lit(&self) -> usize {
        self.grid.par_iter().filter(|light| light.is_on()).count()
    }
}

impl<const N: usize> FromStr for LightSet<N> {
    type Err = Report;

    fn from_str(s: &str) -> Result<Self> {
        s.trim()
            .lines()
            .flat_map(|line| line.trim().chars())
            .map(Light::try_from)
            .collect::<Result<_>>()
            .map(|grid| Self {
                grid,
                is_broken: false,
            })
    }
}

#[derive(Debug)]
struct Light {
    state: LightState,
    next: OnceLock<LightState>,
}

impl Light {
    fn turn_on(&mut self) {
        self.state = LightState::On;
    }

    fn is_on(&self) -> bool {
        matches!(self.state, LightState::On)
    }

    fn try_step(&mut self) -> Result<()> {
        let state = self
            .next
            .take()
            .ok_or_eyre("light's next state was never calculated")?;
        self.state = state;
        Ok(())
    }
}

impl TryFrom<char> for Light {
    type Error = Report;

    fn try_from(ch: char) -> Result<Self> {
        ch.try_into().map(|state| Light {
            state,
            next: OnceLock::new(),
        })
    }
}

#[derive(Debug, Clone, Copy)]
enum LightState {
    On,
    Off,
}

impl TryFrom<char> for LightState {
    type Error = Report;

    fn try_from(ch: char) -> Result<Self> {
        match ch {
            '#' => Ok(Self::On),
            '.' => Ok(Self::Off),
            _ => Err(eyre!(
                "Light can either be on ('#') or off ('.'), not '{}'",
                ch.escape_debug()
            )),
        }
    }
}
