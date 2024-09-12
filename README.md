# EVM Mainnet v2 Upgrade
## Tested and Validated on September 12, 2024

```
HIGHLIGHT:  This branch is compatible with the latest version of Rust, v1.81.0, released September 5, 2024. 

Check the Rust Changelog for details: https://releases.rs
```

This branch is the most recent branch of the Argochain EVM mainnet with both cross swap functionality (evmToSubstrate & substrateToEvm) working as expected.  

To build this branch with the new release above, follow these steps:

**1. Make sure Rust is updated to version 1.81.0, the most stable version:**

```
rustup install 1.81.0
rustup default 1.81.0
rustup override set 1.81.0
```

**2. Check to see if the newest version of Rust is installed on your system.**
```
rustc --version
```

If all goes well, your output should be 

```
rustc 1.81.0 (eeb90cda1 2024-09-04)
```

**3. Install the WASM binaries**

Next, install the WASM binaries. You will need these to build the chain with the new version.

```
rustup target add wasm32-unknown-unknown
cargo install wasm-bindgen-cli
```

**4. Verify the WASM binary was installed**

```
rustup target list --installed
```

Your output should be similar to the below:

```
aarch64-apple-darwin
wasm32-unknown-unknown
```

**5. Update Cargo and Build the Chain in Dev Mode**

```
cargo update
cargo run --release -- --dev
```

That's it. The chain should build and start in development mode for you to test


# Argochain-SDK
![photo_2024-04-29 13 29 45](https://github.com/Devolved-AI/Argochain/assets/96510238/9989a2c0-dbdf-4baa-b8fc-54e3c75f7445)
------------------
Building and compiling the node is a one-time, but very resource intensive process and we recommend the following hardware specifications:

Recommended Specifications\
CPU: 16 cores \
RAM: 32 GB\
Storage: 1TB SSD\
Network: 1 Gbps Internet Connection

Please click [here](https://devolved-ai.gitbook.io/argochain-validator-guide) to go right to the Validator Guide and get started.

[Argochain Validator Guide](https://devolved-ai.gitbook.io/argochain-validator-guide)\
[Argoscan Explorer](https://explorer.argoscan.net)\
[https://devolvedai.com](https://devolvedai.com)

![Argochain's GitHub Stats](https://github-readme-stats.vercel.app/api/pin/?username=devolved-ai&repo=argochain&show_owner=true&theme=radical)


![Profile Views](https://komarev.com/ghpvc/?username=devolved-ai&repo=argochain)
