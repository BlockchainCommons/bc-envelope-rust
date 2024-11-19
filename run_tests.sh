#!/bin/bash

set -e

cargo test
cargo test --no-default-features
cargo test --no-default-features --features attachment
cargo test --no-default-features --features compress
cargo test --no-default-features --features encrypt
cargo test --no-default-features --features expression
cargo test --no-default-features --features known_value
cargo test --no-default-features --features proof
cargo test --no-default-features --features recipient
cargo test --no-default-features --features salt
cargo test --no-default-features --features signature
cargo test --no-default-features --features sskr
cargo test --no-default-features --features types
