# PolkaDot/ArgonChain Validator Node Build Dockerfile
# Author: BuzaG
# License: Apache 2
# Description: This Dockerfile sets up the ArgonChain Validator Node application with necessary configurations and environment variables.

# Build image
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

WORKDIR /app
COPY . .

# Build
RUN cargo build --release

# Update minervaRaw.json
RUN chmod +x ./update_bootnodes.sh
RUN ./update_bootnodes.sh

# Create a prod image to run the app
FROM ubuntu AS prod
LABEL org.opencontainers.image.author="BuzaG"
LABEL org.opencontainers.image.description="ArgoChain Validator Node"
LABEL org.opencontainers.image.version="0.1"

RUN apt update && apt upgrade -y
RUN apt install -y curl jq

WORKDIR /app
COPY --from=build /app/target/release/argochain /app/target/release/argochain
COPY --from=build /app/minervaRaw.json /app/minervaRaw.json

# Prepare scripts
RUN mkdir -p /session
COPY /Docker/init-and-run.sh .
COPY /Docker/rotate_keys_docker.sh .
RUN chmod +x init-and-run.sh rotate_keys_docker.sh

EXPOSE 30333
EXPOSE 9944

CMD ["/app/init-and-run.sh", "${NODE_NAME}"]