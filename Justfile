pubkey := env("AOC_INPUTS_PUBKEY")
secret := env("AOC_INPUTS_SECRET")

run *ARGS:
    cargo run --release -- {{ARGS}}

test *ARGS:
    -RUSTFLAGS="-C target-cpu=native" cargo nextest run --verbose --no-fail-fast {{ARGS}}

encrypt-inputs:
    tar cvz ./tests/fixtures | rage -r {{pubkey}} > ./tests/fixtures.gz.age

decrypt-inputs:
    echo {{secret}} | rage -d -i - ./tests/fixtures.gz.age | tar xvz ./tests/fixtures
