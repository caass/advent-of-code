#!/usr/bin/env sh

set -euo pipefail

cookies --version 2>/dev/null 1>&2 || \
    (printf "Install the \`cookies\` command from https://github.com/barnardb/cookies to continue.\n" >&2 && exit 1)

SESSION_COOKIE="$(cookies https://adventofcode.com session)"
FIXTURES_PATH="./tests/fixtures"

get_input() {
    year=$1
    day=$2

    curl "https://adventofcode.com/$year/day/$day/input" \
        --cookie "session=${SESSION_COOKIE}" \
        -X "GET" \
        --fail-with-body \
        -s
}

if [ ! -d $FIXTURES_PATH ]; then
    mkdir $FIXTURES_PATH
fi

printf "Downloading inputs...\n"

for year in `seq 2015 2023`; do
    printf '%s: ' $year
    year_path="${FIXTURES_PATH}/$year"

    if [ ! -d $year_path ]; then
        mkdir $year_path
    fi

    for day in `seq 1 25`; do
        printf '%s, ' $day

        two_digit_day=$day

        if [ "${#day}" -eq 1 ]; then
            two_digit_day="0$day"
        fi

        input_path="$year_path/$two_digit_day"

        get_input $year $day > $input_path
    done

    printf 'Done!\n'
done

printf 'Done!\n'
