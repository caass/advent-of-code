use std::ops::Index;

use crate::Part;
use crate::Solution;

use super::solution::ReturnValue;

#[repr(transparent)]
pub struct Problem([Option<&'static dyn Solution>; 2]);

impl Problem {
    #[inline]
    pub const fn part(&self, part: Part) -> Option<&'static dyn Solution> {
        let idx = part.as_u8() as usize - 1;
        self.0[idx]
    }

    pub fn parts(&self) -> impl Iterator<Item = (Part, &dyn Solution)> {
        Part::iter().filter_map(|part| self.part(part).map(|solution| (part, solution)))
    }

    #[allow(dead_code, reason = "Sometimes unused depending on solution progress")]
    #[inline]
    pub(crate) const fn unsolved() -> Self {
        Self([None, None])
    }

    #[inline]
    pub const fn partially_solved<F, R>(part_one: &'static F) -> Self
    where
        F: Fn(&str) -> R,
        R: ReturnValue,
    {
        Self([Some(part_one as &dyn Solution), None])
    }

    #[inline]
    pub const fn solved<F1, R1, F2, R2>(part_one: &'static F1, part_two: &'static F2) -> Self
    where
        F1: Fn(&str) -> R1,
        R1: ReturnValue,
        F2: Fn(&str) -> R2,
        R2: ReturnValue,
    {
        Self([
            Some(part_one as &dyn Solution),
            Some(part_two as &dyn Solution),
        ])
    }
}

impl Index<Part> for Problem {
    type Output = dyn Solution;

    #[inline]
    fn index(&self, part: Part) -> &Self::Output {
        self.part(part)
            .unwrap_or_else(move || panic!("Haven't solved part {part} yet"))
    }
}
