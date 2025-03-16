export RUSTFLAGS := "-C target-cpu=native"

# MacOS uses BSD tar, which can generate warnings when untarring on Linux.
tar := if os() == "macos" { "gtar" } else { "tar" }
branch := trim(`git branch --show-current --no-column`)

[private]
default:
  @just --list

# Run the advent of code binary
run year day part input="-":
    cargo run --release --package aoc -- {{year}} {{day}} {{part}} {{input}}

# Test solutions
test *ARGS: decrypt-inputs
    cargo nextest run --all --no-tests=fail --cargo-profile=fast-test {{ARGS}}

# Download and encrypt puzzle inputs from https://adventofcode.com
get-inputs: download-inputs
    #!/usr/bin/env -S bash --posix
    set -euo pipefail

    if [[ -z "$AOC_INPUTS_PUBKEY" ]]; then
        printf "Need AOC_INPUTS_PUBKEY to be set to encrypt puzzle inputs.\n" && exit 1
    fi

    {{tar}} cz ./target/inputs | rage -r $AOC_INPUTS_PUBKEY > ./inputs.gz.age

[private]
decrypt-inputs:
    #!/usr/bin/env -S bash --posix
    set -euo pipefail

    if [[ -z "$AOC_INPUTS_SECRET" ]]; then
        printf "Need AOC_INPUTS_SECRET to be set to decrypt puzzle inputs.\n" && exit 1
    fi

    printenv AOC_INPUTS_SECRET | rage -d -i - ./inputs.gz.age | {{tar}} xz ./target/inputs

[private]
download-inputs:
    #!/usr/bin/env -S bash --posix
    set -euo pipefail

    cookies --version 2>/dev/null 1>&2 || \
        (printf "Install the \`cookies\` command from https://github.com/barnardb/cookies to continue.\n" >&2 && exit 1)

    SESSION_COOKIE="$(cookies https://adventofcode.com session)"
    UNPACKED_FIXTURES_PATH="./target/inputs"

    THIS_YEAR="$(date +%Y)"
    THIS_MONTH="$(date +%m)"
    TODAY="$(date +%d)"

    get_input() {
        year=$1
        day=$2

        sleep 0.5
        curl "https://adventofcode.com/$year/day/$day/input" \
            --cookie "session=${SESSION_COOKIE}" \
            -X "GET" \
            --fail-with-body \
            -s
    }

    rm -rf target/inputs/*

    if [ ! -d $UNPACKED_FIXTURES_PATH ]; then
        mkdir -p $UNPACKED_FIXTURES_PATH
    fi

    printf "Downloading inputs...\n"


    for year in `seq 2015 $(( $THIS_YEAR - 1 ))`; do
        printf '%s: ' $year
        year_path="${UNPACKED_FIXTURES_PATH}/$year"

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
        printf '%s: ' $THIS_YEAR
        year_path="${UNPACKED_FIXTURES_PATH}/$THIS_YEAR"

        if [ ! -d $year_path ]; then
            mkdir $year_path
        fi

        for day in `seq 1 $TODAY`; do
            printf '%s, ' $day

            two_digit_day=$day

            if [ "${#day}" -eq 1 ]; then
                two_digit_day="0$day"
            fi

            input_path="$year_path/$two_digit_day"

            get_input $THIS_YEAR $day > $input_path
        done

        printf 'Done!\n'
    fi
