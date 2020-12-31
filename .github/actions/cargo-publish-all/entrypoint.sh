#!/bin/sh
rustup update
cargo-publish-all --token $1 --yes --verbose