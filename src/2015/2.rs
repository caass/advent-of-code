use std::str::FromStr;

use eyre::{bail, OptionExt, Report};
use rayon::prelude::*;

use crate::types::Problem;

pub const I_WAS_TOLD_THERE_WOULD_BE_NO_MATH: Problem = Problem {
    part_1: Some(|input| part_1(input).to_string()),
    part_2: Some(|input| part_2(input).to_string()),
};

struct Dimensions([usize; 3]);

impl Dimensions {
    /// Returns the amount of wrapping paper needed to wrap a present with dimensions `self`:
    /// enough to cover all six sides plus extra equal to the area of the smallest side.
    #[inline(always)]
    fn wrapping_paper(self) -> usize {
        let Dimensions([l, w, h]) = self;

        let a = l * w;
        let b = w * h;
        let c = h * l;

        2 * a + 2 * b + 2 * c + a.min(b).min(c)
    }

    /// Returns the amount of ribbon needed to tie off a present with dimensions `self`:
    /// enough to wrap around the smallest size, plus bow-material equal to the volume of the present.
    #[inline(always)]
    fn ribbon(self) -> usize {
        let Dimensions([l, w, h]) = self;

        let volume = l * w * h;

        let a = 2 * (l + w);
        let b = 2 * (w + h);
        let c = 2 * (h + l);
        let smallest_side = a.min(b).min(c);

        smallest_side + volume
    }
}

impl FromStr for Dimensions {
    type Err = Report;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut split = s.split('x');
        let a = split.next().ok_or_eyre("No length")?.parse()?;
        let b = split.next().ok_or_eyre("No width")?.parse()?;
        let c = split.next().ok_or_eyre("No height")?.parse()?;
        if let Some(other) = split.next() {
            bail!("Unexpected extra split: {other}");
        };

        Ok(Self([a, b, c]))
    }
}

/// The elves are running low on wrapping paper, and so they need to submit an order for more.
/// They have a list of the dimensions (length `l`, width `w`, and height `h`) of each present, and only want to order
/// exactly as much as they need.
///
/// Fortunately, every present is a box (a perfect _right rectangular prism_), which makes calculating the required
/// wrapping paper for each gift a little easier: find the surface area of the box, which is `2*l*w` + `2*w*h` + `2*h*l`.
/// The elves also need a little extra paper for each present: the area of the smallest side.
///
/// For example:
/// - A present with dimensions `2x3x4` requires `2*6 + 2*12 + 2*8 = 52` square feet of wrapping paper plus `6` square
///   feet of slack, for a total of `58` square feet.
/// - A present with dimensions `1x1x10` requires `2*1 + 2*10 + 2*10 = 42` square feet of wrapping paper plus `1`
///   square foot of slack, for a total of `43` square feet.
///
/// All numbers in the elves' list are in feet. How many **total square feet of wrapping paper** should they order?
fn part_1(input: &str) -> usize {
    input
        .par_lines()
        .map(|line| line.parse::<Dimensions>().unwrap())
        .map(Dimensions::wrapping_paper)
        .sum()
}

/// The elves are also running low on ribbon.
/// Ribbon is all the same width, so they only have to worry about the length they need to order,
/// which they would again like to be exact.
///
/// The ribbon required to wrap a present is the shortest distance around its sides,
/// or the smallest perimeter of any one face. Each present also requires a bow made out of ribbon as well;
/// the feet of ribbon required for the perfect bow is equal to the cubic feet of volume of the present.
/// Don't ask how they tie the bow, though; they'll never tell.
///
/// For example:
/// - A present with dimensions 2x3x4 requires 2+2+3+3 = 10 feet of ribbon to wrap the present plus 2*3*4 = 24 feet of ribbon for the bow, for a total of 34 feet.
/// - A present with dimensions 1x1x10 requires 1+1+1+1 = 4 feet of ribbon to wrap the present plus 1*1*10 = 10 feet of ribbon for the bow, for a total of 14 feet.
///
/// How many total **feet of ribbon** should they order?
fn part_2(input: &str) -> usize {
    input
        .par_lines()
        .map(|line| line.parse::<Dimensions>().unwrap())
        .map(Dimensions::ribbon)
        .sum()
}
