#!/bin/sh
set -e

cross build --release -p roblib-server --target $1
rsync -vhP target/$1/release/roblib-server pi@$2:~
