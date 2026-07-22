#!/usr/bin/env bash
# One-time bootstrap to create initial rmpl binary
# After this, rmpl can rebuild itself with: ./target/debug/rmpl build [debug|release]

set -e

mkdir -p target/debug
rustc src/bin/main.rs --edition 2021 -o target/debug/rmpl
echo "Bootstrap complete."
echo "rmpl can now rebuild itself: ./target/debug/rmpl build [debug|release]"
