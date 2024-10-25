export RUSTFLAGS := "-C target-cpu=native"

# MacOS uses BSD tar, which can generate warnings when untarring on Linux.
tar := if os() == "macos" { "gtar" } else { "tar" }
branch := trim(`git branch --show-current --no-column`)

[private]
default:
  @just --list

# Run the advent of code binary
run year day part: decrypt-inputs
    cargo run --release -- {{year}} {{day}} {{part}} tests/fixtures/{{year}}/{{day}}

# Check for outdated dependencies
outdated *ARGS:
    cargo outdated

# Run unit & integration tests
test *ARGS: decrypt-inputs
    cargo nextest run --no-tests=fail --cargo-profile=fast-test {{ARGS}}

# Benchmark against inputs in `tests/fixtures.gz.age`
bench *ARGS: decrypt-inputs
    cargo bench --bench bench -- {{ARGS}}

# Run benchmarks under `bencher` i.e. on CI
bencher testbed="adhoc" *ARGS='': decrypt-inputs
    bencher run \
    --project adventofcode \
    --branch {{branch}} \
    --threshold-measure latency \
    --threshold-test t_test \
    --threshold-max-sample-size 64 \
    --threshold-upper-boundary 0.99 \
    --thresholds-reset \
    --err \
    --adapter rust_criterion \
    --testbed {{testbed}} \
    "cargo bench --color always {{ARGS}}" || exit 0

# Download and encrypt puzzle inputs from https://adventofcode.com
get-inputs: download-inputs
    #!/usr/bin/env -S bash --posix
    set -euo pipefail

    if [[ -z "$AOC_INPUTS_PUBKEY" ]]; then
        printf "Need AOC_INPUTS_PUBKEY to be set to encrypt puzzle inputs.\n" && exit 1
    fi

    {{tar}} cz ./tests/fixtures | rage -r $AOC_INPUTS_PUBKEY > ./tests/fixtures.gz.age

# Clean `target/` and `tests/fixtures/`
clean: clean-inputs
    cargo clean

[private]
decrypt-inputs:
    #!/usr/bin/env -S bash --posix
    set -euo pipefail

    if [[ -z "$AOC_INPUTS_SECRET" ]]; then
        printf "Need AOC_INPUTS_SECRET to be set to decrypt puzzle inputs.\n" && exit 1
    fi

    printenv AOC_INPUTS_SECRET | rage -d -i - ./tests/fixtures.gz.age | {{tar}} xz ./tests/fixtures

[private]
clean-inputs:
    rm -rf tests/fixtures

[private]
download-inputs: clean-inputs
    #!/usr/bin/env -S bash --posix
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
