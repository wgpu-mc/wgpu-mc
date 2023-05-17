#!/bin/sh

echo "Cleaning with cargo fix"
cargo +nightly fix --allow-dirty --allow-staged --manifest-path rust/Cargo.toml
echo "Cleaning with clippy"
cargo +nightly clippy --fix --allow-dirty --allow-staged --manifest-path rust/Cargo.toml
