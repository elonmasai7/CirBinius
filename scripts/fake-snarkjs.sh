#!/bin/sh
# Fake snarkjs for CI artifact generation
# Copies a fixture wtns to the output path ($5)
FIXTURE="tests/circuits/simple_mul.wtns"
if [ -f "$FIXTURE" ]; then
  cp "$FIXTURE" "$5"
fi
