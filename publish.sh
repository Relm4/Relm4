#!/usr/bin/env -S bash -euo pipefail

# Check the code
cargo update
cargo fmt --all -- --check
cargo clippy --all-targets -- --deny warnings
cargo clippy --features "all" -- --deny warnings
cargo clippy --examples -- --deny warnings
cargo test

# Publish and pass all arguments to cargo
cargo publish -p relm4-macros
cargo publish -p relm4-css
cargo publish -p relm4
cargo publish -p relm4-components
