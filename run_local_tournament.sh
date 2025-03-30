#!/bin/bash

cargo build --release -p quoridor-cli --quiet 

# Run with debug mode off and minimal output
QUORIDOR_DEBUG=0 ./target/release/quoridor-cli