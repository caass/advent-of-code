use aoc_meta::Problem;

pub const AN_ELEPHANT_NAMED_JOSEPH: Problem =
    Problem::partially_solved(&|input| input.parse().map(josephus));

fn josephus(num_elves: usize) -> usize {
    let prev_power_of_two = 1 << num_elves.ilog2();
    let remainder = num_elves - prev_power_of_two;
    2 * remainder + 1
}
