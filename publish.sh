#!/usr/bin/env bash

set -x

cd "${0%/*}"

cd workspace/proc_macros_impl
cargo publish
sleep 5s
cd ../proc_macros
cargo publish
sleep 5s
cd ../..
cargo publish
sleep 5s
