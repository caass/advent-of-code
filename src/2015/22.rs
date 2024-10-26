use std::collections::{HashSet, VecDeque};
use std::hash::Hash;
use std::num::NonZeroU8;

use either::Either;
use eyre::{OptionExt, Result};
use fnv::FnvBuildHasher;
use rayon::prelude::*;

use crate::meta::Problem;

use self::boss::Boss;
use self::player::Player;
use self::spell::{Action, Effects, Instant, Kind, Spell};

/// <https://adventofcode.com/2015/day/22>
pub const WIZARD_SIMULATOR_20XX: Problem = Problem::solved(
    &|input| {
        let boss: Boss = input.parse()?;
        let player = Player::new();

        let game = Game::new(player, boss, false);
        game.least_mana()
    },
    &|input| {
        let boss: Boss = input.parse()?;
        let player = Player::new();

        let game = Game::new(player, boss, true);
        game.least_mana()
    },
);

struct Game {
    queue: VecDeque<GameState>,
    history: HashSet<GameState, FnvBuildHasher>,
}

impl Game {
    fn new(player: Player, boss: Boss, hard_mode: bool) -> Self {
        let mut queue = VecDeque::default();
        queue.push_back(GameState::new(player, boss, hard_mode));

        Game {
            queue,
            history: HashSet::default(),
        }
    }

    fn least_mana(mut self) -> Result<usize> {
        while let Some(state) = self.queue.pop_front() {
            let Some(next_states) = state.next_turn() else {
                continue;
            };

            for next in next_states {
                if !self.history.contains(&next) {
                    self.history.insert(next);
                    self.queue.push_back(next);
                }
            }
        }

        self.history
            .par_iter()
            .filter(|state| state.is_victory())
            .map(|state| state.total_mana_spent)
            .min()
            .ok_or_eyre("couldn't beat boss")
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct GameState {
    player: Player,
    boss: Boss,
    effects: Effects,
    total_mana_spent: usize,
    is_player_turn: bool,
    hard_mode: bool,
}

impl GameState {
    fn new(player: Player, boss: Boss, hard_mode: bool) -> Self {
        Self {
            player,
            boss,
            effects: Effects::default(),
            total_mana_spent: 0,
            is_player_turn: true,
            hard_mode,
        }
    }

    #[inline]
    fn is_victory(&self) -> bool {
        self.boss.is_dead() && !self.player.is_dead()
    }

    #[inline]
    fn is_over(&self) -> bool {
        self.player.is_dead() || self.boss.is_dead()
    }

    #[inline]
    fn process_effects(&mut self) -> u8 {
        let mut armor = 0;
        for action in self.effects.current() {
            match action {
                Action::Shield => armor = action.value(),
                Action::Poison => self.boss.defend(action.value()),
                Action::Recharge => self.player.recharge(action.value()),
            }
        }

        armor
    }

    fn next_turn(&self) -> Option<impl Iterator<Item = GameState>> {
        if self.is_over() {
            return None;
        }

        let is_player_turn = self.is_player_turn;

        let mut this = *self;
        this.is_player_turn = !is_player_turn;
        if this.hard_mode && is_player_turn {
            this.player.defend(unsafe { NonZeroU8::new_unchecked(1) });
        }

        if this.is_over() {
            return Some(Either::Right(std::iter::once(this)));
        }

        let armor = this.process_effects();
        if this.is_over() {
            return Some(Either::Right(std::iter::once(this)));
        }

        Some(if is_player_turn {
            let base = this;
            let mut next_states = [None; 5];

            for (i, spell) in base
                .player
                .options()
                .into_iter()
                .enumerate()
                .filter_map(|(i, opt)| opt.map(move |spell| (i, spell)))
            {
                next_states[i] = base.try_cast(spell);
            }

            Either::Left(next_states.into_iter().flatten())
        } else {
            this.player.defend(this.boss.attack(armor));
            Either::Right(std::iter::once(this))
        })
    }

    fn try_cast(mut self, spell: Spell) -> Option<GameState> {
        self.player.cast(spell.cost);
        self.total_mana_spent = self.total_mana_spent.saturating_add(spell.cost.into());

        match spell.kind {
            Kind::Instant(Instant::Damage) => self.boss.defend(spell.value()),
            Kind::Instant(Instant::Drain) => {
                self.boss.defend(spell.value());
                self.player.heal(spell.value());
            }
            Kind::Effect { turns, action } if !self.effects.is_active(action) => {
                self.effects.cast(action, turns);
            }
            Kind::Effect { .. } => return None,
        };

        Some(self)
    }
}

mod boss {
    use std::num::NonZeroU8;
    use std::str::FromStr;

    use eyre::{OptionExt, Report, Result};

    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub(super) struct Boss {
        hp: u8,
        damage: u8,
    }

    impl Boss {
        /// Defend from a player's attack, taking damage.
        #[inline]
        pub(super) fn defend(&mut self, damage: u8) {
            self.hp = self.hp.saturating_sub(damage);
        }

        /// Attack, returning the amount of damage dealt to the player after taking armor into account.
        #[inline]
        pub(super) fn attack(self, armor: u8) -> NonZeroU8 {
            match self.damage.saturating_sub(armor) {
                0 => unsafe { NonZeroU8::new_unchecked(1) },
                n @ 1.. => unsafe { NonZeroU8::new_unchecked(n) },
            }
        }

        /// Returns `true` if the boss is dead.
        #[inline]
        pub(super) fn is_dead(self) -> bool {
            self.hp == 0
        }
    }

    impl FromStr for Boss {
        type Err = Report;

        fn from_str(s: &str) -> Result<Self> {
            let mut lines = s.trim().lines().map(str::trim);

            let hp_line = lines.next().ok_or_eyre("empty input")?;
            let hp = hp_line.trim_start_matches("Hit Points: ").parse()?;

            let damage_line = lines.next().ok_or_eyre("incomplete input")?;
            let damage = damage_line.trim_start_matches("Damage: ").parse()?;

            Ok(Boss { hp, damage })
        }
    }
}

mod player {
    use std::num::NonZeroU8;

    use super::spell::{Spell, SPELLS};

    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub(super) struct Player {
        hp: u8,
        mana: u16,
    }

    impl Player {
        /// Construct a new [`Player`] with the given HP and mana
        #[inline]
        pub(super) const fn new() -> Self {
            Self { hp: 50, mana: 500 }
        }

        /// Check if the player is dead
        #[inline]
        pub(super) fn is_dead(self) -> bool {
            self.mana < Spell::MAGIC_MISSILE.cost.into() || self.hp == 0
        }

        /// Defend from an enemey attack, taking damage equal to the given value
        #[inline]
        pub(super) fn defend(&mut self, damage: NonZeroU8) {
            self.hp = self.hp.saturating_sub(damage.get());
        }

        /// Return a list of all the spells this player can cast with their current mana.
        #[inline]
        pub(super) fn options(self) -> [Option<Spell>; 5] {
            std::array::from_fn(|i| (u16::from(SPELLS[i].cost) <= self.mana).then_some(SPELLS[i]))
        }

        /// Regain a given amount of mana
        #[inline]
        pub(super) fn recharge(&mut self, mana: u8) {
            self.mana = self.mana.saturating_add(mana.into());
        }

        /// Spend a given amount of mana
        #[inline]
        pub(super) fn cast(&mut self, mana: u8) {
            self.mana = self.mana.saturating_sub(mana.into());
        }

        /// Heal for a given amount of HP
        #[inline]
        pub(super) fn heal(&mut self, hp: u8) {
            self.hp = self.hp.saturating_add(hp);
        }
    }
}

mod spell {
    use std::fmt::{self, Display, Formatter};
    use std::hash::{Hash, Hasher};
    use std::num::NonZeroU8;

    use enum_map::{Enum, EnumMap};

    #[derive(Debug, Clone, Copy)]
    pub(super) struct Spell {
        pub(super) name: &'static str,
        pub(super) cost: u8,
        pub(super) kind: Kind,
    }

    impl Display for Spell {
        fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
            f.write_str(self.name)
        }
    }

    impl Hash for Spell {
        fn hash<H: Hasher>(&self, state: &mut H) {
            Hash::hash(self.name, state);
        }
    }

    pub const SPELLS: [Spell; 5] = [
        Spell::MAGIC_MISSILE,
        Spell::DRAIN,
        Spell::SHIELD,
        Spell::POISON,
        Spell::RECHARGE,
    ];

    impl Spell {
        pub const MAGIC_MISSILE: Spell = Spell {
            name: "Magic Missile",
            cost: 53,
            kind: Kind::Instant(Instant::Damage),
        };

        pub const DRAIN: Spell = Spell {
            name: "Drain",
            cost: 73,
            kind: Kind::Instant(Instant::Drain),
        };

        pub const SHIELD: Spell = Spell {
            name: "Shield",
            cost: 113,
            kind: Kind::Effect {
                turns: 6,
                action: Action::Shield,
            },
        };

        pub const POISON: Spell = Spell {
            name: "Poison",
            cost: 173,
            kind: Kind::Effect {
                turns: 6,
                action: Action::Poison,
            },
        };

        pub const RECHARGE: Spell = Spell {
            name: "Recharge",
            cost: 229,
            kind: Kind::Effect {
                turns: 5,
                action: Action::Recharge,
            },
        };

        #[inline]
        pub const fn value(&self) -> u8 {
            self.kind.value()
        }
    }

    #[derive(Debug, Clone, Copy)]
    pub enum Kind {
        Instant(Instant),
        Effect { turns: u8, action: Action },
    }

    impl Kind {
        #[inline]
        pub const fn value(self) -> u8 {
            match self {
                Kind::Instant(instant) => instant.value(),
                Kind::Effect { action, .. } => action.value(),
            }
        }
    }

    #[derive(Debug, Clone, Copy)]
    pub enum Instant {
        Damage,
        Drain,
    }

    impl Instant {
        #[inline]
        pub const fn value(self) -> u8 {
            match self {
                Instant::Damage => 4,
                Instant::Drain => 2,
            }
        }
    }

    #[derive(Debug, Clone, Copy, Enum)]
    pub enum Action {
        Shield,
        Poison,
        Recharge,
    }

    impl Action {
        #[inline]
        pub const fn value(self) -> u8 {
            match self {
                Action::Shield => 7,
                Action::Poison => 3,
                Action::Recharge => 101,
            }
        }
    }

    #[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub(super) struct Effects(EnumMap<Action, Option<NonZeroU8>>);

    impl Effects {
        /// Returns `true` if the given action is already in effect.
        #[inline]
        pub(super) fn is_active(self, action: Action) -> bool {
            self.0[action].is_some()
        }

        /// Cast the given spell action for the given number of turns.
        #[inline]
        pub(super) fn cast(&mut self, action: Action, turns: u8) {
            self.0[action] = NonZeroU8::new(turns);
        }

        /// Returns all actions currently in effect, decrementing the number of
        /// turns remaining on each by 1.
        pub(super) fn current(&mut self) -> impl Iterator<Item = Action> + use<'_> {
            self.0
                .iter_mut()
                .filter(|(_, turns_left)| turns_left.is_some())
                .map(|(action, turns_left)| {
                    *turns_left = turns_left.and_then(|n| NonZeroU8::new(n.get() - 1));

                    action
                })
        }
    }
}

// #[derive(Clone, Debug, Eq, Hash, PartialEq)]
// struct State {
//     hp: i32,
//     mp: i32,
//     bhp: i32,
//     totalmp: i32,
//     shield: i32,
//     poison: i32,
//     recharge: i32,
//     bdmg: i32,
// }

// impl State {
//     fn new(bhp: i32, bdmg: i32) -> Self {
//         State {
//             hp: 50,
//             mp: 500,
//             bhp,
//             totalmp: 0,
//             shield: 0,
//             poison: 0,
//             recharge: 0,
//             bdmg,
//         }
//     }

//     fn expand(&self) -> Vec<(State, &'static str)> {
//         let mut res = Vec::new();
//         let mut state = self.clone();

//         // Apply effects at the start of the turn
//         if state.recharge > 0 {
//             state.mp += 101;
//             state.recharge -= 1;
//         }
//         if state.shield > 0 {
//             state.shield -= 1;
//         }
//         if state.poison > 0 {
//             state.bhp -= 3;
//             state.poison -= 1;
//         }
//         if state.bhp <= 0 {
//             state.bhp = 0;
//             return vec![(state, "")];
//         }

//         // Spells: missile
//         if state.mp >= 53 {
//             let mut new_state = state.clone();
//             new_state.mp -= 53;
//             new_state.bhp -= 4;
//             new_state.totalmp += 53;
//             res.push((new_state, "missile"));
//         }

//         // Spells: drain
//         if state.mp >= 73 {
//             let mut new_state = state.clone();
//             new_state.mp -= 73;
//             new_state.hp += 2;
//             new_state.bhp -= 2;
//             new_state.totalmp += 73;
//             res.push((new_state, "drain"));
//         }

//         // Conditional Spells
//         // Shield
//         if state.shield == 0 && state.mp >= 113 {
//             let mut new_state = state.clone();
//             new_state.mp -= 113;
//             new_state.totalmp += 113;
//             new_state.shield = 6;
//             res.push((new_state, "shield"));
//         }

//         // Poison
//         if state.poison == 0 && state.mp >= 173 {
//             let mut new_state = state.clone();
//             new_state.mp -= 173;
//             new_state.totalmp += 173;
//             new_state.poison = 6;
//             res.push((new_state, "poison"));
//         }

//         // Recharge
//         if state.recharge == 0 && state.mp >= 229 {
//             let mut new_state = state.clone();
//             new_state.mp -= 229;
//             new_state.totalmp += 229;
//             new_state.recharge = 5;
//             res.push((new_state, "recharge"));
//         }

//         // Filtering
//         res.retain(|(state, _)| state.mp >= 0);

//         for (new_state, spell) in &mut res {
//             if new_state.poison > 0 {
//                 new_state.bhp -= 3;
//                 new_state.poison -= 1;
//             }
//             if new_state.shield > 0 {
//                 new_state.shield -= 1;
//             }
//             if new_state.recharge > 0 {
//                 new_state.mp += 101;
//                 new_state.recharge -= 1;
//             }
//             new_state.hp -= new_state.bdmg - if new_state.shield > 0 { 7 } else { 0 };
//         }

//         res.retain(|(state, _)| state.hp > 0);
//         res
//     }
// }

// fn main() {
//     let inp = "13\n8";

//     let bhp: i32 = inp.lines().next().unwrap().parse().unwrap();
//     let bdmg: i32 = inp.lines().nth(1).unwrap().parse().unwrap();

//     let initial_state = State::new(bhp, bdmg);

//     let mut queue: VecDeque<State> = VecDeque::new();
//     let mut visited: HashSet<State> = HashSet::new();
//     let mut comefrom: HashMap<State, (State, &'static str)> = HashMap::new();

//     queue.push_back(initial_state.clone());
//     visited.insert(initial_state.clone());

//     while let Some(current) = queue.pop_front() {
//         if current.bhp <= 0 {
//             let mut path = Vec::new();
//             let mut step = current.clone();

//             while let Some((prev_state, spell)) = comefrom.get(&step) {
//                 path.push(spell);
//                 step = prev_state.clone();
//             }

//             path.reverse();
//             println!("{:?}", path);
//             println!("Total Mana: {}", current.totalmp);
//             return;
//         }

//         for (next_state, spell) in current.expand() {
//             if !visited.contains(&next_state) {
//                 queue.push_back(next_state.clone());
//                 visited.insert(next_state.clone());
//                 comefrom.insert(next_state.clone(), (current.clone(), spell));
//             }
//         }
//     }

//     println!("impossible");
// }
