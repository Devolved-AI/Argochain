FROM ubuntu
# PolkaDot/ArgonChain Validator Node Dockerfile
# Author: BuzaG
# License: MIT
# Description: This Dockerfile sets up the ArgonChain Validator Node application with necessary configurations and environment variables.

# Usage:
# docker run -d -p 30333:30333 -p 9944:9944 -v /argochain-host-folder/:/argochain -e NODE_NAME=my-nodename argocd-validator-node

LABEL org.opencontainers.image.author="BuzaG"
LABEL org.opencontainers.image.description="Dockerfile for ArgonChain by BuzaG"
LABEL org.opencontainers.image.version="0.1"

# Set environment variables with default values
ENV NODE_NAME=my-nodename

# Create directories if they don't already exist
RUN mkdir -p /var/log/argochain /var/opt/argochain /argochain/log /argochain/opt

# Create symbolic links
RUN ln -s /var/log/argochain /argochain/log && \
    ln -s /var/opt/argochain /argochain/opt

EXPOSE 30333
EXPOSE 9944

WORKDIR /app

# Install dependecies
RUN apt update && apt upgrade -y
RUN apt install -y build-essential clang curl git libssl-dev llvm libudev-dev python3 python3-pip make protobuf-compiler python3-tqdm jq
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
ENV PATH="/root/.cargo/env:${PATH}"
RUN rustup default stable && \
    rustup update && \
    rustup update nightly && \
    rustup target add wasm32-unknown-unknown --toolchain nightly

# git clone https://github.com/devolved-ai/argochain
COPY . .

# Build
RUN cargo build --release

# Run scripts
RUN $SHELL ./update_bootnodes.sh

CMD ["/app/init-and-run.sh", "${NODE_NAME}"]
