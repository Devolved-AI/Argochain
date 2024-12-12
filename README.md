# Dec 12, 2024 UPDATES

This branch has the following updates to the **node/Cargo.toml** and **runtime/Cargo.toml** files
- The addition of the following to the ***dependencies*** section
    - zstd = "0.13.2"
    - secp256k1 = "0.30.0"
    - secp256k1-sys = "0.9.2"

In addition, the Emscripten SDK may need to be installed in the project root of Argochain to accommodate the issues dealing with C, Clang, and WASM that that scep256k1 elliptic curve needs to operate along with other crates.

You can install and activate the Emscripten SDK by running the following:

```
git clone https://github.com/emscripten-core/emsdk.git
cd emsdk
./emsdk install latest
./emsdk activate latest
source ./emsdk_env.sh
emcc --version
```

You should see a version number print out in your terminal.

However, after running `cargo build --release` after cleaning and updating the project, the following error still persists and needs to be fixed

`error: failed to run custom build command for 'secp256k1-sys v0.9.2'`


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
