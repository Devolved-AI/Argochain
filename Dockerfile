# ------------------------------------------
# Usage:
#
# docker compose up -d
# Your session key will be at /argochain/session/.session_key
#
# ------------------------------------------

# PolkaDot/ArgonChain Validator Node Build Dockerfile
# Author: BuzaG
# License: Apache 2
# Description: This Dockerfile sets up the ArgonChain Validator Node application with necessary configurations and environment variables.
# TODO: create a multi-stage minimal rust build

FROM ubuntu AS build

# Install dependecies
RUN apt update && apt upgrade -y
RUN apt install -y build-essential clang curl git libssl-dev llvm libudev-dev python3 python3-pip make protobuf-compiler python3-tqdm jq
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
ENV PATH="/root/.cargo/bin:${PATH}"
RUN rustup default stable && \
    rustup update && \
    rustup update nightly && \
    rustup target add wasm32-unknown-unknown --toolchain nightly

# Build
WORKDIR /app
COPY . .
RUN cargo build --release

# Update minervaRaw.json
RUN chmod +x ./update_bootnodes.sh
RUN ./update_bootnodes.sh

FROM ubuntu AS prod-stage
LABEL org.opencontainers.image.author="BuzaG"
LABEL org.opencontainers.image.description="Dockerfile for ArgonChain by BuzaG"
LABEL org.opencontainers.image.version="0.1"

RUN apt update && apt upgrade -y
RUN apt install -y curl git python3-tqdm jq

WORKDIR /app
COPY --from=build /app/target/release/argochain /app/target/release/argochain
COPY --from=build /app/minervaRaw.json /app/minervaRaw.json

# Create directories if they don't already exist
RUN mkdir -p /session

EXPOSE 30333
EXPOSE 9944

# Prepare scripts
COPY /Docker/init-and-run.sh .
COPY /Docker/rotate_keys_docker.sh .
RUN chmod +x init-and-run.sh rotate_keys_docker.sh

CMD ["/app/init-and-run.sh", "${NODE_NAME}"]