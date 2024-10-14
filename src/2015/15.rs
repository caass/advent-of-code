use std::{
    collections::{HashMap, HashSet},
    fmt::{self, Display, Formatter},
    hash::{Hash, Hasher},
    ops::{Add, AddAssign},
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

use crate::meta::Problem;

pub const SCIENCE_FOR_HUNGRY_PEOPLE: Problem =
    Problem::solved(&|input| best_cookie(input, None), &|input| {
        best_cookie(input, Some(500))
    });

const NUM_TABLESPOONS: usize = 100;

fn best_cookie(input: &str, calorie_restriction: Option<usize>) -> Result<usize> {
    let kitchen = Kitchen::from_input(input)?;

    let cookie = kitchen.best_cookie(calorie_restriction)?;
    Ok(cookie.score())
}

#[derive(Debug, PartialEq)]
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
    fn from_input(input: &'s str) -> Result<Self> {
        input.lines().map(|line| line.trim().try_into()).collect()
    }

    fn best_cookie(&self, calorie_restriction: Option<usize>) -> Result<Cookie<'s>> {
        self.ingredients
            .iter()
            .copied()
            .combinations_with_replacement(NUM_TABLESPOONS)
            .par_bridge()
            .map(Cookie::bake)
            .filter(|cookie| {
                if let Some(cal_limit) = calorie_restriction {
                    cookie.calories() == cal_limit
                } else {
                    true
                }
            })
            .max_by_key(Cookie::score)
            .ok_or_eyre("didn't bake any cookies")
    }
}

#[derive(Debug)]
struct Cookie<'s> {
    ingredients: HashMap<Ingredient<'s>, u8, FnvBuildHasher>,
    qualities: Qualities,
}

impl Display for Cookie<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        self.ingredients
            .iter()
            .try_for_each(|(Ingredient { name, .. }, n)| writeln!(f, "- {n} tbsp {name}"))
    }
}

impl<'s> Cookie<'s> {
    fn bake<I: IntoIterator<Item = Ingredient<'s>>>(ingredients: I) -> Self {
        Self::from_iter(ingredients)
    }
}

impl<'s> FromIterator<Ingredient<'s>> for Cookie<'s> {
    fn from_iter<T: IntoIterator<Item = Ingredient<'s>>>(iter: T) -> Self {
        let mut ingredients: HashMap<Ingredient<'s>, u8, FnvBuildHasher> = HashMap::default();
        for ingredient in iter {
            ingredients.entry(ingredient).or_default().add_assign(1);
        }

        let qualities = ingredients
            .iter()
            .map(|(ingredient, n)| ingredient.tbsp(*n))
            .reduce(|a, b| a + b)
            .unwrap_or_default();

        Self {
            ingredients,
            qualities,
        }
    }
}

impl Cookie<'_> {
    fn score(&self) -> usize {
        self.qualities.score()
    }

    fn calories(&self) -> usize {
        self.qualities.calories.try_into().unwrap_or_default()
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

#[derive(Debug, Clone, Copy, PartialEq, Default)]
struct Qualities {
    capacity: isize,
    durability: isize,
    flavor: isize,
    texture: isize,
    calories: isize,
}

impl Qualities {
    fn score(&self) -> usize {
        if [self.capacity, self.durability, self.flavor, self.texture]
            .iter()
            .any(|quality| (isize::MIN..0).contains(quality))
        {
            0
        } else {
            (self.capacity * self.durability * self.flavor * self.texture)
                .try_into()
                .unwrap_or_default()
        }
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

#[test]
fn example() {
    let butterscotch = Ingredient {
        name: "Butterscotch",
        qualities: Qualities {
            capacity: -1,
            durability: -2,
            flavor: 6,
            texture: 3,
            calories: 8,
        },
    };
    let cinnamon = Ingredient {
        name: "Cinnamon",
        qualities: Qualities {
            capacity: 2,
            durability: 3,
            flavor: -2,
            texture: -1,
            calories: 3,
        },
    };

    let mut ingredients = HashSet::with_hasher(FnvBuildHasher::default());
    ingredients.insert(butterscotch);
    ingredients.insert(cinnamon);

    let expected_kitchen = Kitchen { ingredients };
    let actual_kitchen = Kitchen::from_input(
        "Butterscotch: capacity -1, durability -2, flavor 6, texture 3, calories 8
         Cinnamon: capacity 2, durability 3, flavor -2, texture -1, calories 3",
    )
    .unwrap();

    assert_eq!(actual_kitchen, expected_kitchen);

    assert_eq!(
        butterscotch.tbsp(44),
        Qualities {
            #[allow(clippy::neg_multiply)]
            capacity: 44 * -1,
            durability: 44 * -2,
            flavor: 44 * 6,
            texture: 44 * 3,
            calories: 44 * 8
        }
    );

    assert_eq!(
        cinnamon.tbsp(56),
        Qualities {
            capacity: 56 * 2,
            durability: 56 * 3,
            flavor: 56 * -2,
            #[allow(clippy::neg_multiply)]
            texture: 56 * -1,
            calories: 3 * 56
        }
    );

    let sum = butterscotch.tbsp(44) + cinnamon.tbsp(56);
    assert_eq!(
        sum,
        Qualities {
            capacity: 68,
            durability: 80,
            flavor: 152,
            texture: 76,
            calories: 520
        }
    );

    let expected_score = sum.score();
    assert_eq!(expected_score, 62842880);

    let actual_score = actual_kitchen.best_cookie(None).unwrap().score();
    assert_eq!(actual_score, expected_score);
}
