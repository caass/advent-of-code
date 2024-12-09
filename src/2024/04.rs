use itertools::Itertools;
use rayon::prelude::*;

use crate::meta::Problem;

pub const CERES_SEARCH: Problem = Problem::solved(&|input| input.count_xmasses(), &|input| {
    input.count_mases_in_x()
});

trait WordSearch {
    fn count_xmasses(&self) -> usize;

    fn count_mases_in_x(&self) -> usize;
}

impl WordSearch for str {
    fn count_xmasses(&self) -> usize {
        let ((horizontal, vertical), (slope_up, slope_down)) = rayon::join(
            || {
                rayon::join(
                    || count_horizontal_xmasses(self),
                    || count_vertical_xmasses(self),
                )
            },
            || {
                rayon::join(
                    || count_diagonal_slope_up_xmasses(self),
                    || count_diagonal_slope_down_xmasses(self),
                )
            },
        );

        horizontal + vertical + slope_up + slope_down
    }

    fn count_mases_in_x(&self) -> usize {
        self.lines()
            .tuple_windows()
            .par_bridge()
            .map(|(top_line, middle_line, bottom_line)| {
                top_line
                    .as_bytes()
                    .windows(3)
                    .zip(middle_line.as_bytes().windows(3))
                    .zip(bottom_line.as_bytes().windows(3))
                    .filter(|((top_window, middle_window), bottom_window)| {
                        if middle_window[1] != b'A' {
                            return false;
                        }

                        let slope_down = &[top_window[0], middle_window[1], bottom_window[2]];
                        let slope_up = &[top_window[2], middle_window[1], bottom_window[0]];

                        let slope_down_hits = slope_down == b"MAS" || slope_down == b"SAM";
                        let slope_up_hits = slope_up == b"MAS" || slope_up == b"SAM";

                        slope_down_hits && slope_up_hits
                    })
                    .count()
            })
            .sum()
    }
}

fn count_horizontal_xmasses(word_search: &str) -> usize {
    word_search
        .par_lines()
        .map(|line| {
            line.as_bytes()
                .windows(4)
                .filter(|slice| slice == b"XMAS" || slice == b"SAMX")
                .count()
        })
        .sum()
}

fn count_vertical_xmasses(word_search: &str) -> usize {
    word_search
        .lines()
        .tuple_windows()
        .par_bridge()
        .map(|(first_line, second_line, third_line, fourth_line)| {
            first_line
                .bytes()
                .zip(second_line.bytes())
                .zip(third_line.bytes())
                .zip(fourth_line.bytes())
                .filter(|(((first_byte, second_byte), third_byte), fourth_byte)| {
                    let bytes = [*first_byte, *second_byte, *third_byte, *fourth_byte];
                    &bytes == b"XMAS" || &bytes == b"SAMX"
                })
                .count()
        })
        .sum()
}

fn count_diagonal_slope_down_xmasses(word_search: &str) -> usize {
    word_search
        .lines()
        .tuple_windows()
        .par_bridge()
        .map(|(first_line, second_line, third_line, fourth_line)| {
            first_line
                .bytes()
                .zip(second_line.bytes().skip(1))
                .zip(third_line.bytes().skip(2))
                .zip(fourth_line.bytes().skip(3))
                .filter(|(((first_byte, second_byte), third_byte), fourth_byte)| {
                    let bytes = [*first_byte, *second_byte, *third_byte, *fourth_byte];
                    &bytes == b"XMAS" || &bytes == b"SAMX"
                })
                .count()
        })
        .sum()
}

fn count_diagonal_slope_up_xmasses(word_search: &str) -> usize {
    word_search
        .lines()
        .tuple_windows()
        .par_bridge()
        .map(|(first_line, second_line, third_line, fourth_line)| {
            first_line
                .bytes()
                .skip(3)
                .zip(second_line.bytes().skip(2))
                .zip(third_line.bytes().skip(1))
                .zip(fourth_line.bytes())
                .filter(|(((first_byte, second_byte), third_byte), fourth_byte)| {
                    let bytes = [*first_byte, *second_byte, *third_byte, *fourth_byte];
                    &bytes == b"XMAS" || &bytes == b"SAMX"
                })
                .count()
        })
        .sum()
}
