use aoc_meta::Problem;

pub const AN_ELEPHANT_NAMED_JOSEPH: Problem =
    Problem::solved(&|input| input.parse().map(josephus), &|input| {
        input.parse().map(threesephus)
    });

fn josephus(num_elves: usize) -> usize {
    let prev_power_of_two = 1 << num_elves.ilog2();
    let remainder = num_elves - prev_power_of_two;
    2 * remainder + 1
}

fn threesephus(num_elves: usize) -> usize {
    let prev_power_of_three = 3_usize.pow(num_elves.ilog(3));
    let remainder = num_elves - prev_power_of_three;

    match remainder {
        0 => prev_power_of_three,
        _ if remainder <= prev_power_of_three => remainder,
        _ => 2 * (remainder - prev_power_of_three) + 1,
    }
}
