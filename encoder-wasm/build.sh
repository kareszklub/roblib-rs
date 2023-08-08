#!/bin/sh
set -e

podman build -t wasm .
podman run --rm -it --init -v "$(realpath ..):/src" wasm bash -c "cd /src/encoder-wasm && wasm-pack build -s kareszklub --release"
