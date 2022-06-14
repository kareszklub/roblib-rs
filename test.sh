#!/bin/sh
set -e

cross build --release --target $1 -p roland --example test
rsync -vlP target/$1/release/examples/test pi@$2:~
