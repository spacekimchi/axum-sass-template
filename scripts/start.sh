#! /usr/bin/env bash

./scripts/init_db.sh

./scripts/init_redis.sh

# cargo watch -x run
# --no-vcs-ignores tells watch to ignore filtering out files in version control systems (.gitignore)
# We include --bin to let it know what binary to run. Check what binarys are defined in the Cargo.toml file
cargo watch --no-vcs-ignores -x 'run --bin axum_sass_template'
