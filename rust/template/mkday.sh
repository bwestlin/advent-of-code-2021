#!/bin/bash

DAY=$1
if [ -z "$DAY" ]
  then
    echo "No day supplied"
    exit 1
fi

DST="src/bin/day$DAY.rs"
cp -n template/day.rs "$DST"
echo "$DST created"
