#!/bin/bash

# Exit on error
set -e

# Check the code
cargo update
cargo fmt --all -- --check
cargo clippy --all-targets -- --deny warnings
cargo clippy --features "all" -- --deny warnings
cargo clippy --examples -- --deny warnings
cargo test

# Check if everything has been committed
if [ ! -z "$(git status --untracked-files=no --porcelain)" ]; then 
  echo "There are uncommitted changes -> exiting"
  exit 1
fi

# Temporarily move Cargo.toml from examples
# Otherwise cargo will not include the examples
# because it thinks the examples are another crate.
mv examples/Cargo.toml examples/_Cargo.toml

# Publish and pass all arguments to cargo
cargo publish -p relm4-macros "$@" --allow-dirty
cargo publish -p relm4 "$@" --allow-dirty
cargo publish -p relm4-components "$@" --allow-dirty

# Move Cargo.toml from examples into original position
mv examples/_Cargo.toml examples/Cargo.toml
