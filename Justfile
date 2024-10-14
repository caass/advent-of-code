set quiet

pubkey := env("AOC_INPUTS_PUBKEY")
secret := env("AOC_INPUTS_SECRET")

run *ARGS:
    RUSTFLAGS="-C target-cpu=native" cargo run --release -- {{ARGS}}

test *ARGS: decrypt-inputs
    -RUSTFLAGS="-C target-cpu=native" cargo nextest run --verbose --no-fail-fast {{ARGS}}

encrypt-inputs:
    tar cz ./tests/fixtures | rage -r {{pubkey}} > ./tests/fixtures.gz.age

decrypt-inputs:
    echo {{secret}} | rage -d -i - ./tests/fixtures.gz.age | tar xz ./tests/fixtures

check-wasm *ARGS:
    cargo clippy --target=wasm32-unknown-unknown

test-wasm *ARGS:
    cargo nextest run --verbose --no-fail-fast --target=wasm32-unknown-unknown {{ARGS}}
