# Introducing the ArgoChain-SDK: Your Gateway to the ArgoChain Testnet
![photo_2024-04-29 13 29 45](https://github.com/Devolved-AI/Argochain/assets/96510238/9989a2c0-dbdf-4baa-b8fc-54e3c75f7445)
------------------
We are thrilled to release the ArgoChain-SDK, specifically designed for developers eager to explore and innovate within the ArgoChain ecosystem. This toolkit facilitates the development, testing, and deployment of decentralized applications, providing you a robust platform on our testnet.

Here's the updated guide with instructions on how to check the time until the next era:

---

# Adding a New Validator

### Step 1: Install Rust

#### On Linux:

1. **Install Dependencies**:
   ```bash
   sudo apt install build-essential git clang curl libssl-dev protobuf-compiler
   ```
2. **Install Rust**:
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   source $HOME/.cargo/env
   rustup default stable
   rustup update
   rustup target add wasm32-unknown-unknown --toolchain nightly
   ```

#### On Windows:

1. **Enable WSL**:
   ```bash
   wsl --install
   ```
2. **Install Dependencies and Rust**:
   ```bash
   sudo apt update
   sudo apt install git clang curl libssl-dev llvm libudev-dev make protobuf-compiler
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   source ~/.cargo/env
   rustup default stable
   rustup update
   rustup target add wasm32-unknown-unknown --toolchain nightly
   ```

### Step 2: Setup Key Pairs

1. **Install Subkey**:
   ```bash
   cargo install subkey --force --locked
   ```

2. **Clone and Build ArgoChain**:
   ```bash
   git clone --branch tokenomics https://github.com/Devolved-AI/Argochain.git
   cd Argochain
   cargo build --release
   ```

### Step 3: Generate Keys

**For BABE (Block Production):**
```bash
subkey generate --scheme Sr25519 --password-interactive
./target/release/argochain key insert --base-path /tmp/node05 --chain ./customSpecRaw.json --scheme Sr25519 --suri <Secret Seed> --password-interactive --key-type babe
```

**For GRANDPA (Finality Gadget):**
```bash
subkey generate --scheme Ed25519 --password-interactive
./target/release/argochain key insert --base-path /tmp/node05 --chain ./customSpecRaw.json --scheme Ed25519 --suri <Secret Seed> --password-interactive --key-type gran
```

**For IM Online (Heartbeat Mechanism):**
```bash
subkey generate --scheme Sr25519 --password-interactive
./target/release/argochain key insert --base-path /tmp/node05 --chain ./customSpecRaw.json --scheme Sr25519 --suri <Secret Seed> --password-interactive --key-type imon
```

**For Authority Discovery (Network Discovery):**
```bash
subkey generate --scheme Sr25519 --password-interactive
./target/release/argochain key insert --base-path /tmp/node05 --chain ./customSpecRaw.json --scheme Sr25519 --suri <Secret Seed> --password-interactive --key-type audi
```

### Step 4: Run Validator Node

Start the validator node with the following command:
```bash
nohup ./target/release/argochain --base-path /tmp/<path> --chain customSpecRaw.json --port <port> --rpc-port <rpc port> --telemetry-url "wss://telemetry.polkadot.io/submit/ 0" --name <name> --validator --rpc-methods Unsafe --unsafe-rpc-external --rpc-max-connections 15000 --rpc-cors all &
```

### Step 5: View Logs

To view the logs of your running validator node, use the following command:
```bash
tail -f nohup.out
```
This will display the log output in real-time, allowing you to monitor the node's activity.

### Step 6: Check Time Until Next Era

1. **Access Polkadot.js Apps**: Go to [Polkadot.js Apps](https://polkadot.js.org/apps/).
2. **Navigate to Staking**: Click on "Network" in the top menu, then select "Staking".
3. **Check Era Progress**: On the staking page, you will see the current era and a progress bar indicating how much time is left until the next era.

### Step 7: Staking

1. **Access ArgoScan Explorer**: Go to [ArgoScan Explorer](https://explorer.argoscan.net/).
2. **Navigate to Staking**: Go to Network -> Staking -> Accounts.
3. **Get Rotate Keys**: Run the following command to rotate keys:
   ```bash
   curl -H "Content-Type: application/json" --data '{"jsonrpc":"2.0", "method":"author_rotateKeys", "params":[], "id":1}' http://localhost:<rpc port>
   ```
4. **Bond and Validate**: Follow the bonding process to start validating.

Wait for an era (approximately 1 hour) to begin participating in block validation.

---

This guide provides detailed steps for installing Rust, setting up key pairs, running a validator node, viewing logs, checking the time until the next era, and staking for new validators on the ArgoChain. For more information, refer to the original [documentation](https://github.com/mitun567/Docs/blob/main/Add_Validator.md).
