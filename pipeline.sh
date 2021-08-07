#!/usr/bin/env bash


echo "---build"
cargo build || exit
echo
echo "---build --all-features"
cargo build --all-features || exit
echo
echo "---test"
cargo test || exit
echo
echo "---test --all-features"
cargo test || exit
echo
echo "---clippy --all-features --all-targets"
cargo clippy --all-features --all-targets || exit
echo
echo "---doc"
cargo doc || exit
echo
echo "---doc --all-features"
cargo doc || exit
