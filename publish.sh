#!/usr/bin/env bash

set -x
set -e

cd "${0%/*}"

cd workspace/proc_macros_impl
cargo publish
cd ../proc_macros
cargo publish
cd ../..
cargo publish
