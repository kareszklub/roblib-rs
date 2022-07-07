#!/bin/sh
set -e

name=${3:-test}

cross build --release --target $1 -p roland --example $name
rsync -vhP target/$1/release/examples/$name pi@$2:~
