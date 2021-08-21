#!/usr/bin/env bash


echo "---build"
cargo build --workspace --no-default-features || exit
echo
echo "---build --all-features"
cargo build --workspace --all-features || exit
echo
echo "---test"
cargo test --workspace --no-default-features || exit
echo
echo "---test --all-features"
cargo test --workspace --all-features || exit
echo
echo "---clippy --all-features --all-targets"
cargo clippy --workspace --all-features --all-targets || exit
echo
echo "---doc"
cargo doc --workspace --no-default-features || exit
echo
echo "---doc --all-features"
cargo doc --workspace --all-features || exit
