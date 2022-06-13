#!/bin/sh
set -e

cross build --release -p roblib-server --target $1
rsync -vlP target/$1/release/roblib-server pi@$2:~
