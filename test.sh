#!/bin/sh
set -e

./check_all.py -d roblib default all async gpio roland camloc gpio-backend
./check_all.py -d roblib-server default all backend roland gpio camloc
./check_all.py -d roblib-client default all async roland gpio camloc tcp udp http ws
./check_all.py -de roblib-client default all async roland gpio camloc tcp udp http ws

[ -n "$GITHUB_ACTIONS" ] && echo "##[group]roblib-macro"
cargo clippy -p roblib-macro 2>&1
[ -n "$GITHUB_ACTIONS" ] && echo "##[endgroup]"

[ -n "$GITHUB_ACTIONS" ] && echo "##[group]test: roblib"
cargo test --all-features -p roblib 2>&1
[ -n "$GITHUB_ACTIONS" ] && echo "##[endgroup]"

[ -n "$GITHUB_ACTIONS" ] && echo "##[group]test: roblib-server"
cargo test --all-features -p roblib-server 2>&1
[ -n "$GITHUB_ACTIONS" ] && echo "##[endgroup]"

[ -n "$GITHUB_ACTIONS" ] && echo "##[group]test: roblib-client"
cargo test --all-features -p roblib-client 2>&1
[ -n "$GITHUB_ACTIONS" ] && echo "##[endgroup]"

exit 0
