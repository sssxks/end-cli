clippy:
    # We are lenient with tests. By default, --lib --bins are included, so only add examples and benches.
    cargo clippy --workspace --examples --benches

test:
    cargo nextest run --workspace --show-progress none

done: clippy test

tokei:
    tokei crates web --exclude package-lock.json --exclude '*.toml'

archive:
    git archive --format=zip --output ../end2-head.zip HEAD

data:
    cd scripts/data_generator && uv run data-generator generate
