set quiet

pubkey := env("AOC_INPUTS_PUBKEY")
secret := env("AOC_INPUTS_SECRET")

[private]
default:
  just --list

run *ARGS:
    RUSTFLAGS="-C target-cpu=native" cargo run --release -- {{ARGS}}

test *ARGS: decrypt-inputs
    cargo nextest run {{ARGS}}

get-inputs: download-inputs
    tar cz ./tests/fixtures | rage -r {{pubkey}} > ./tests/fixtures.gz.age

[private]
decrypt-inputs:
    echo {{secret}} | rage -d -i - ./tests/fixtures.gz.age | tar xz ./tests/fixtures

[private]
download-inputs:
    #!/usr/bin/env sh

    set -euo pipefail

    cookies --version 2>/dev/null 1>&2 || \
        (printf "Install the \`cookies\` command from https://github.com/barnardb/cookies to continue.\n" >&2 && exit 1)

    SESSION_COOKIE="$(cookies https://adventofcode.com session)"
    FIXTURES_PATH="./tests/fixtures"

    THIS_YEAR="$(date +%Y)"
    THIS_MONTH="$(date +%m)"
    TODAY="$(date +%d)"

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

    for year in `seq 2015 $(( $THIS_YEAR - 1 ))`; do
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

    if [ $THIS_MONTH -eq 12 ]; then
        for day in `seq 1 $TODAY`; do
            printf '%s, ' $day

            two_digit_day=$day

            if [ "${#day}" -eq 1 ]; then
                two_digit_day="0$day"
            fi

            input_path="$year_path/$two_digit_day"

            get_input $year $day > $input_path
        done
    fi
