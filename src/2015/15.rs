use std::{
    collections::{HashMap, HashSet},
    hash::{Hash, Hasher},
    ops::{Add, AddAssign},
    sync::OnceLock,
};

use eyre::{eyre, OptionExt, Report, Result};
use fnv::FnvBuildHasher;
use itertools::Itertools;
use rayon::prelude::*;
use winnow::{
    ascii::{alpha1, dec_int},
    combinator::{eof, seq},
    error::ContextError,
    Parser,
};

use crate::types::{problem, Problem};

pub const SCIENCE_FOR_HUNGRY_PEOPLE: Problem = problem!(best_cookie);

fn best_cookie(input: &str) -> Result<usize> {
    let kitchen = input
        .lines()
        .map(|line| line.try_into())
        .collect::<Result<Kitchen, _>>()?;
    let cookie = kitchen.bake()?;
    Ok(cookie.score())
}

struct Kitchen<'s> {
    ingredients: HashSet<Ingredient<'s>, FnvBuildHasher>,
}

impl<'s> FromIterator<Ingredient<'s>> for Kitchen<'s> {
    fn from_iter<T: IntoIterator<Item = Ingredient<'s>>>(iter: T) -> Self {
        Self {
            ingredients: HashSet::from_iter(iter),
        }
    }
}

impl<'s> Kitchen<'s> {
    fn bake(&self) -> Result<Cookie<'s>> {
        // bake `self.ingredients.len() choose 100 with replacement` cookies
        self.ingredients
            .iter()
            .copied()
            .combinations_with_replacement(100)
            .par_bridge()
            .map(|tablespoons| {
                debug_assert_eq!(tablespoons.len(), 100);
                tablespoons.into_iter().collect()
            })
            .max_by_key(|cookie: &Cookie| cookie.score())
            .ok_or_eyre("baked with 0 ingredients")
    }
}

struct Cookie<'s> {
    ingredients: HashMap<Ingredient<'s>, u8, FnvBuildHasher>,
    score: OnceLock<usize>,
}

impl<'s> FromIterator<Ingredient<'s>> for Cookie<'s> {
    fn from_iter<T: IntoIterator<Item = Ingredient<'s>>>(iter: T) -> Self {
        let mut map: HashMap<Ingredient<'s>, u8, FnvBuildHasher> = HashMap::default();
        for ingredient in iter {
            map.entry(ingredient).or_default().add_assign(1);
        }

        Self {
            ingredients: map,
            score: OnceLock::new(),
        }
    }
}

impl Cookie<'_> {
    fn score(&self) -> usize {
        *self.score.get_or_init(|| {
            self.ingredients
                .iter()
                .map(|(ingredient, n)| ingredient.tbsp(*n))
                .reduce(|a, b| a + b)
                .map(|q| q.score())
                .unwrap_or_default()
        })
    }
}

#[derive(Debug, Clone, Copy)]
struct Ingredient<'s> {
    name: &'s str,
    qualities: Qualities,
}

impl Ingredient<'_> {
    #[inline(always)]
    fn tbsp(&self, tbsp: u8) -> Qualities {
        let tbsp: isize = tbsp.into();
        let Ingredient {
            qualities:
                Qualities {
                    capacity,
                    durability,
                    flavor,
                    texture,
                    calories,
                },
            ..
        } = *self;

        Qualities {
            capacity: capacity * tbsp,
            durability: durability * tbsp,
            flavor: flavor * tbsp,
            texture: texture * tbsp,
            calories: calories * tbsp,
        }
    }
}

impl PartialEq for Ingredient<'_> {
    fn eq(&self, other: &Self) -> bool {
        self.name.eq(other.name)
    }
}

impl Eq for Ingredient<'_> {}

impl Hash for Ingredient<'_> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.name.hash(state);
    }
}

#[derive(Debug, Clone, Copy)]
struct Qualities {
    capacity: isize,
    durability: isize,
    flavor: isize,
    texture: isize,
    calories: isize,
}

impl Qualities {
    fn score(&self) -> usize {
        let sum = self.capacity + self.durability + self.flavor + self.texture;
        sum.try_into().unwrap_or_default()
    }
}

impl Add for Qualities {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Qualities {
            capacity: self.capacity + rhs.capacity,
            durability: self.durability + rhs.durability,
            flavor: self.flavor + rhs.flavor,
            texture: self.texture + rhs.texture,
            calories: self.calories + rhs.calories,
        }
    }
}

impl AddAssign for Qualities {
    fn add_assign(&mut self, rhs: Self) {
        *self = *self + rhs;
    }
}

impl<'s> TryFrom<&'s str> for Ingredient<'s> {
    type Error = Report;

    fn try_from(value: &'s str) -> Result<Self> {
        seq! {Ingredient {
            name: alpha1::<_, ContextError>,
            qualities: seq!{Qualities {
                _: ": capacity ",
                capacity: dec_int,
                _: ", durability ",
                durability: dec_int,
                _: ", flavor ",
                flavor: dec_int,
                _: ", texture ",
                texture: dec_int,
                _: ", calories ",
                calories: dec_int,
            }},
            _: eof
        }}
        .parse(value)
        .map_err(|e| eyre!("{e}"))
    }
}
