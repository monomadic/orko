#!/bin/sh

cargo build --release --all
cp target/release/orko ~/.bin/
