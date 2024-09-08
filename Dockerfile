FROM ubuntu
# ------------------------------------------
# Usage:
#
# docker compose up -d
# Your session key will be at /argochain/session/.session_key
#
# ------------------------------------------

# PolkaDot/ArgonChain Validator Node Dockerfile
# Author: BuzaG
# License: Apache 2
# Description: This Dockerfile sets up the ArgonChain Validator Node application with necessary configurations and environment variables.

LABEL org.opencontainers.image.author="BuzaG"
LABEL org.opencontainers.image.description="Dockerfile for ArgonChain by BuzaG"
LABEL org.opencontainers.image.version="0.1"

# Create directories if they don't already exist
RUN mkdir -p /session

EXPOSE 30333
EXPOSE 9944

WORKDIR /app

# Install dependecies
RUN apt update && apt upgrade -y
RUN apt install -y build-essential clang curl git libssl-dev llvm libudev-dev python3 python3-pip make protobuf-compiler python3-tqdm jq
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
ENV PATH="/root/.cargo/bin:${PATH}"
RUN rustup default stable && \
    rustup update && \
    rustup update nightly && \
    rustup target add wasm32-unknown-unknown --toolchain nightly

# git clone https://github.com/devolved-ai/argochain
COPY . .

# Build
RUN cargo build --release

# Prepare scripts
COPY /Docker/init-and-run.sh .
COPY /Docker/rotate_keys_docker.sh .
RUN chmod +x update_bootnodes.sh init-and-run.sh rotate_keys_docker.sh

# Run scripts
RUN ./update_bootnodes.sh

CMD ["/app/init-and-run.sh", "${NODE_NAME}"]
