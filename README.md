# Argochain 🏆

Welcome to Argochain! This document will guide you through the necessary steps to set up your development environment, build the code, and run a single node in a development environment.

## Requirements

Before you start, ensure your development environment is ready. Follow these steps:

1. **Prepare your development environment**: Make sure to set up your development environment by following the [official Substrate installation instructions](https://docs.substrate.io/install/).

2. **Clone the repository**:
   Use Git to clone the Argochain repository to your local machine:
   ```bash
   git clone https://github.com/mitun567/Argochain.git
   ```

3. **Source the cargo environment file**:
   Before proceeding, ensure your shell is configured correctly by sourcing the cargo environment file:
   ```bash
   source ~/.cargo/env
   ```

## Getting Started

To build and launch the Argochain node, follow these QuickStart steps:

### Build the Code

First, build the codebase using Cargo with the `--release` flag to compile the project in release mode for optimal performance:
```bash
cargo run --release
```

### Run the Single Node in a Development Environment

After building the project, you can start a single node running in a development environment using the following command:
```bash
./target/release/argochain --dev
```

## Argochain

Dive into the world of Argochain and start exploring the capabilities of your newly set-up node. If you encounter any issues or have questions, refer to our documentation or reach out to the community for support.