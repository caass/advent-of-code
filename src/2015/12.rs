use rayon::prelude::*;
use serde_json::Value;

use crate::meta::Problem;

/// <https://adventofcode.com/2015/day/12>
pub const JS_ABACUS_FRAMEWORK_IO: Problem = Problem::solved(
    &|input| serde_json::from_str(input).map(|root| sum_numbers(root, false)),
    &|input| serde_json::from_str(input).map(|root| sum_numbers(root, true)),
);

fn sum_numbers(value: Value, ignore_red: bool) -> i64 {
    match value {
        Value::Null | Value::Bool(_) | Value::String(_) => 0,
        Value::Number(number) => number.as_i64().expect("number to be an integer"),
        Value::Array(vec) => vec
            .into_par_iter()
            .map(|val| sum_numbers(val, ignore_red))
            .sum(),
        Value::Object(map) if ignore_red && map.values().any(|val| val == "red") => 0,
        Value::Object(map) => map
            .into_iter()
            .par_bridge()
            .map(|(_, value)| sum_numbers(value, ignore_red))
            .sum(),
    }
}
