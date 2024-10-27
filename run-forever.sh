#!/bin/bash
FILE=./target/release/SNTPing
if ! test -f "$FILE"; then
  
    echo "Could not find a build, make sure you run 'cargo build --release'!"
    exit
fi

until ./target/release/SNTPing; do
    echo "
Oh NOOOO, the pinger crashed with exit code $?.
Respawning in a 10 seconds!" >&2
    sleep 10
done