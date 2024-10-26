use std::str::FromStr;

use eyre::{bail, eyre, OptionExt, Report, Result};
use itertools::Itertools;
use rayon::prelude::*;

use crate::meta::Problem;

/// <https://adventofcode.com/2015/day/21>
pub const RPG_SIMULATOR_20XX: Problem = Problem::solved(
    &|input: &str| {
        let player = Character::PLAYER;
        let boss = input.parse()?;
        let loadout = player
            .best_loadout(boss)
            .ok_or_eyre("impossible to beat boss")?;

        Ok::<_, Report>(loadout.cost())
    },
    &|input: &str| {
        let player = Character::PLAYER;
        let boss = input.parse()?;
        let loadout = player
            .worst_loadout(boss)
            .ok_or_eyre("impossible to lose to boss")?;

        Ok::<_, Report>(loadout.cost())
    },
);

#[derive(Debug)]
struct Loadout {
    weapon: Weapon,
    armor: Option<Armor>,
    rings: [Option<Ring>; 2],
}

impl Loadout {
    fn cost(&self) -> u16 {
        u16::from(self.weapon.cost)
            + self.armor.as_ref().map_or(0, |armor| u16::from(armor.cost))
            + self
                .rings
                .iter()
                .flatten()
                .map(|ring| u16::from(ring.cost))
                .sum::<u16>()
    }
}

#[derive(Debug, Clone, Copy)]
struct Character {
    hp: u8,
    armor: u8,
    damage: u8,
}

impl Character {
    const PLAYER: Character = Character {
        hp: 100,
        armor: 0,
        damage: 0,
    };

    fn damage_to(self, enemy: Character) -> u8 {
        if self.damage <= enemy.armor {
            1
        } else {
            self.damage - enemy.armor
        }
    }

    fn best_loadout(self, enemy: Character) -> Option<Loadout> {
        loadouts()
            .filter(|loadout| self.with_loadout(loadout).wins_against(enemy))
            .min_by_key(Loadout::cost)
    }

    fn worst_loadout(self, enemy: Character) -> Option<Loadout> {
        loadouts()
            .filter(|loadout| self.with_loadout(loadout).loses_to(enemy))
            .max_by_key(Loadout::cost)
    }

    /// Returns `true` if `self` loses a fight against `enemy`
    #[inline]
    fn loses_to(self, enemy: Character) -> bool {
        !self.wins_against(enemy)
    }

    /// Returns `true` if `self` wins a fight against `enemy`
    fn wins_against(self, enemy: Character) -> bool {
        // this is like a linear equation:
        //
        // let f(t) = the damage dealt by the opponent
        // let t = the round #, where 1 is after the first round, 2 is after the second round, etc
        // let d = the damage dealt my the opponent
        // let a = the armor worn by `self`
        // f(t) = t * max(d - a, 1)
        //
        // therefore you can compute the damage dealt by an opponent after a certain number of rounds (ignoring that you or they may die)
        // by computing f(t) for some t.
        //
        // to compute the round where you'll die, you just find the value of t where f(t) <= self.hp
        // by calculating for the other character, you can find when they'll die.
        //
        // The winner is whoever has a higher `t` when f(t) <= self.hp, where ties go to the player.

        #[inline]
        fn rounds_survived(protagonist: Character, antagonist: Character) -> u8 {
            (0..=u8::MAX)
                .into_par_iter()
                .by_exponential_blocks()
                .find_first(|&round| {
                    // t * max(d - a, 1) - self.hp <= 0
                    round as usize * antagonist.damage_to(protagonist) as usize
                        >= protagonist.hp as usize
                })
                .unwrap_or_else(|| panic!("protagonist survived > 255 rounds against antagonist"))
        }

        let (we_survive, they_survive) = rayon::join(
            || rounds_survived(self, enemy),
            || rounds_survived(enemy, self),
        );

        we_survive >= they_survive
    }

    fn with_loadout(mut self, loadout: &Loadout) -> Self {
        let Loadout {
            weapon,
            armor,
            rings,
        } = loadout;

        self.damage += weapon.damage;
        if let Some(Armor { defense, .. }) = armor {
            self.armor += defense;
        }

        rings
            .iter()
            .flatten()
            .for_each(|Ring { buff, .. }| match buff {
                RingBuff::Damage(damage) => self.damage += damage,
                RingBuff::Defense(defense) => self.armor += defense,
            });

        self
    }
}

impl FromStr for Character {
    type Err = Report;

    fn from_str(s: &str) -> Result<Self> {
        let mut hp = None;
        let mut armor = None;
        let mut damage = None;

        s.trim().lines().try_for_each(|line| {
            let (stat, value_str) = line
                .trim()
                .split_once(": ")
                .ok_or_else(|| eyre!("Error parsing line {line}: no `:`"))?;
            let value = value_str.parse()?;

            match stat {
                "Hit Points" if hp.is_none() => hp = Some(value),
                "Armor" if armor.is_none() => armor = Some(value),
                "Damage" if damage.is_none() => damage = Some(value),
                "Hit Points" | "Armor" | "Damage" => {
                    bail!("Duplicate value for \"{stat}\" in input")
                }
                other => bail!("Unknown stat \"{other}\" in input"),
            };

            Ok(())
        })?;

        let hp = hp.ok_or_eyre("No value set for Hit Points")?;
        let armor = armor.ok_or_eyre("No value set for Armor")?;
        let damage = damage.ok_or_eyre("No value set for Damage")?;

        Ok(Self { hp, armor, damage })
    }
}

#[derive(Debug, Clone, Copy)]
struct Weapon {
    cost: u8,
    damage: u8,
}

#[derive(Debug, Clone, Copy)]
struct Armor {
    cost: u8,
    defense: u8,
}

#[derive(Debug, Clone, Copy)]
struct Ring {
    cost: u8,
    buff: RingBuff,
}

#[derive(Debug, Clone, Copy)]
enum RingBuff {
    Damage(u8),
    Defense(u8),
}

fn loadouts() -> impl ParallelIterator<Item = Loadout> {
    WEAPONS
        .par_iter()
        .copied()
        .flat_map(|weapon| {
            ARMOR
                .par_iter()
                .copied()
                .map(Some)
                .chain([None])
                .map(move |armor| (weapon, armor))
        })
        .flat_map_iter(|(weapon, armor)| {
            RINGS
                .iter()
                .copied()
                .map(Some)
                .chain([None, None])
                .tuple_combinations()
                .map(move |(ring1, ring2)| Loadout {
                    weapon,
                    armor,
                    rings: [ring1, ring2],
                })
        })
}

const WEAPONS: [Weapon; 5] = [DAGGER, SHORT_SWORD, WAR_HAMMER, LONG_SWORD, GREAT_AXE];
const ARMOR: [Armor; 5] = [LEATHER, CHAIN_MAIL, SPLINT_MAIL, BANDED_MAIL, PLATE_MAIL];
const RINGS: [Ring; 6] = [
    DAMAGE_1, DAMAGE_2, DAMAGE_3, DEFENSE_1, DEFENSE_2, DEFENSE_3,
];

const DAGGER: Weapon = Weapon { cost: 8, damage: 4 };
const SHORT_SWORD: Weapon = Weapon {
    cost: 10,
    damage: 5,
};
const WAR_HAMMER: Weapon = Weapon {
    cost: 25,
    damage: 6,
};
const LONG_SWORD: Weapon = Weapon {
    cost: 40,
    damage: 7,
};
const GREAT_AXE: Weapon = Weapon {
    cost: 74,
    damage: 8,
};

const LEATHER: Armor = Armor {
    cost: 13,
    defense: 1,
};
const CHAIN_MAIL: Armor = Armor {
    cost: 31,
    defense: 2,
};
const SPLINT_MAIL: Armor = Armor {
    cost: 53,
    defense: 3,
};
const BANDED_MAIL: Armor = Armor {
    cost: 75,
    defense: 4,
};
const PLATE_MAIL: Armor = Armor {
    cost: 102,
    defense: 5,
};

const DAMAGE_1: Ring = Ring {
    cost: 25,
    buff: RingBuff::Damage(1),
};
const DAMAGE_2: Ring = Ring {
    cost: 50,
    buff: RingBuff::Damage(2),
};
const DAMAGE_3: Ring = Ring {
    cost: 100,
    buff: RingBuff::Damage(3),
};

const DEFENSE_1: Ring = Ring {
    cost: 20,
    buff: RingBuff::Defense(1),
};
const DEFENSE_2: Ring = Ring {
    cost: 40,
    buff: RingBuff::Defense(2),
};
const DEFENSE_3: Ring = Ring {
    cost: 80,
    buff: RingBuff::Defense(3),
};
