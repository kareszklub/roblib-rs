# See here for image contents: https://github.com/microsoft/vscode-dev-containers/tree/v0.236.0/containers/rust/.devcontainer/base.Dockerfile
ARG VARIANT="buster"
FROM mcr.microsoft.com/vscode/devcontainers/rust:0-${VARIANT}

USER vscode
RUN cargo install cargo-edit && cargo install cross
