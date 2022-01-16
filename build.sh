#!/bin/bash
set -e

cross build --release --target armv7-unknown-linux-gnueabihf
rsync -vlP target/armv7-unknown-linux-gnueabihf/release/roblib-server pi@$PI:~
