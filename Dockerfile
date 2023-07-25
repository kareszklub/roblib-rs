FROM rust:bookworm

WORKDIR /src

RUN export DEBIAN_FRONTEND=noninteractive && set -ex && \
    dpkg --add-architecture armhf && \
    dpkg --add-architecture arm64 && \
    apt update -y && \
    apt install -y musl-dev musl-dev:armhf musl-dev:arm64 && \
    rustup target add x86_64-unknown-linux-musl armv7-unknown-linux-musleabihf aarch64-unknown-linux-musl && \
    mkdir -vp ~/.cargo && \
    echo '[target.armv7-unknown-linux-musleabihf]' >> $CARGO_HOME/config.toml && \
    echo 'linker = "arm-linux-musleabihf-gcc"' >> $CARGO_HOME/config.toml && \
    echo "[target.aarch64-unknown-linux-musl]" >> $CARGO_HOME/config.toml && \
    echo 'linker = "aarch64-linux-musl-gcc"' >> $CARGO_HOME/config.toml && \
    rm -rf /var/lib/apt/lists/* && \
    true
