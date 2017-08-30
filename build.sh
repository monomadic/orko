#!/bin/sh

cargo build --release --all
cp target/release/pickle ~/.bin/
