#!/bin/sh
set -e

command -v docker >/dev/null && docker=docker
command -v podman >/dev/null && docker=podman

if [ -z "$docker" ]; then
    echo "neither docker or podman found"
    exit 1
fi

echo "using $docker"

target="$1"
command="${2:-build}"
# features="--features=$3"
#
# [ -z "$3" ] && features="--all-features"

profile="${3:-roland}"

if [ -z "$target" ]; then
    # echo "Usage: $0 <target> [command=build] [features=alll]"
    echo "Usage: $0 <target> [command=build] [profile=roland]"
    exit 1
fi

# ran twice to make sure the script exits on fail
./features.py ubuntu "$profile" >/dev/null

eval $(./features.py ubuntu "$profile" 2>/dev/null)

cmd="cargo $command --release --target=$target --features=$f"

echo $cmd

$docker run --rm -it --init -v $(pwd):/src docker.io/vbeni/rust-arm-musl $cmd

