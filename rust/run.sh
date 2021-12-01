#!/bin/bash

DAY=$1
if [ -z "$DAY" ]; then
  echo "No day supplied"
  exit 1
fi

FEATURES=""
INPUT=""
if [ "$2" == "timeit" ]; then
  FEATURES="$2"
  INPUT="${@:3}"
elif [ "$2" == "print" ]; then
  FEATURES="$2"
  INPUT="${@:3}"
else
  INPUT="${@:2}"
fi

if [ -z "$INPUT" ]; then
  INPUT="../input/day$DAY"
fi

RUST_BACKTRACE=1 cargo run --features="$FEATURES" --bin "day$DAY" --release "$INPUT"
