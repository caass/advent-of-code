use phf::phf_map;

use crate::types::ProblemSet;
use crate::util::mod_days;

mod_days!(1, 2, 3, 4, 5, 6);

pub const PROBLEMS: ProblemSet = ProblemSet(phf_map! {
    1u8 => NOT_QUITE_LISP,
    2u8 => I_WAS_TOLD_THERE_WOULD_BE_NO_MATH,
    3u8 => PERFECTLY_SPHERICAL_HOUSES_IN_A_VACUUM,
    4u8 => THE_IDEAL_STOCKING_STUFFER,
    5u8 => DOESNT_HE_HAVE_INTERN_ELVES_FOR_THIS,
    6u8 => PROBABLY_A_FIRE_HAZARD
});
