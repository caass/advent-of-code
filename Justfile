set quiet

pubkey := env("AOC_INPUTS_PUBKEY")
secret := env("AOC_INPUTS_SECRET")

run *ARGS:
    RUSTFLAGS="-C target-cpu=native" cargo run --release -- {{ARGS}}

test *ARGS: decrypt-inputs
    cargo nextest run {{ARGS}}

encrypt-inputs: download-inputs
    tar cz ./tests/fixtures | rage -r {{pubkey}} > ./tests/fixtures.gz.age

decrypt-inputs:
    echo {{secret}} | rage -d -i - ./tests/fixtures.gz.age | tar xz ./tests/fixtures

download-inputs:
    ./scripts/download-inputs.sh
