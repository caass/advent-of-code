use std::ops::Index;

use eyre::Report;

use crate::meta::Part;
use crate::meta::Solution;

#[repr(transparent)]
pub struct Problem([Option<&'static dyn Solution>; 2]);

impl Problem {
    #[inline(always)]
    pub const fn part(&self, part: Part) -> Option<&'static dyn Solution> {
        let idx = part.as_u8() as usize - 1;
        self.0[idx]
    }

    pub fn parts(&self) -> impl Iterator<Item = (Part, &dyn Solution)> {
        Part::iter().flat_map(|part| self.part(part).map(|solution| (part, solution)))
    }

    #[allow(dead_code, reason = "Sometimes unused depending on solution progress")]
    #[inline(always)]
    pub(crate) const fn unsolved() -> Self {
        Self([None, None])
    }

    #[allow(dead_code, reason = "Sometimes unused depending on solution progress")]
    #[inline(always)]
    pub(crate) const fn partially_solved<F, T, E>(part_one: &'static F) -> Self
    where
        F: Fn(&str) -> Result<T, E>,
        T: ToString,
        E: Into<Report>,
    {
        Self([Some(part_one as &dyn Solution), None])
    }

    #[allow(dead_code, reason = "Sometimes unused depending on solution progress")]
    #[inline(always)]
    pub(crate) const fn solved<F1, T1, E1, F2, T2, E2>(
        part_one: &'static F1,
        part_two: &'static F2,
    ) -> Self
    where
        F1: Fn(&str) -> Result<T1, E1>,
        T1: ToString,
        E1: Into<Report>,
        F2: Fn(&str) -> Result<T2, E2>,
        T2: ToString,
        E2: Into<Report>,
    {
        Self([
            Some(part_one as &dyn Solution),
            Some(part_two as &dyn Solution),
        ])
    }
}

impl Index<Part> for Problem {
    type Output = dyn Solution;

    #[inline(always)]
    fn index(&self, part: Part) -> &Self::Output {
        self.part(part)
            .unwrap_or_else(move || panic!("Haven't solved part {part} yet"))
    }
}
