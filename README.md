# UPDATE DEC 11, 2024

Compilation of this branch will resolve the **frame-metadata**, **wasm-unknown-unknown** error and the **getrandom** error.

However, the error that is present deals with the **secp256k1 elliptic curve.**

The error is below for your review.

```
(solc) pavondunbar@Pavons-MacBook-Pro Argochain % cargo update
warning: patch for the non root package will be ignored, specify patch at the workspace root:
package:   /Users/pavondunbar/Argochain/runtime/Cargo.toml
workspace: /Users/pavondunbar/Argochain/Cargo.toml
    Updating git repository `https://github.com/paritytech/frame-metadata`
    Updating git repository `https://github.com/bkchr/merkleized-metadata.git`
    Updating git repository `https://github.com/paritytech/substrate-secp256k1`
remote: Repository not found.
fatal: repository 'https://github.com/paritytech/substrate-secp256k1/' not found
error: failed to load source for dependency `secp256k1`

Caused by:
  Unable to update https://github.com/paritytech/substrate-secp256k1?branch=master

Caused by:
  failed to clone into: /Users/pavondunbar/.cargo/git/db/substrate-secp256k1-34d2b3e40e85fc4b

Caused by:
  process didn't exit successfully: `git fetch --force --update-head-ok 'https://github.com/paritytech/substrate-secp256k1' '+refs/heads/master:refs/remotes/origin/master'` (exit status: 128)
```

As part of the solution, a new hidden **.cargo** directory was created which has been pushed to this branch. The contents of this directory contains a **config.toml** file. 

If there is a fix, please push to a new branch.  Thanks!

Pavon


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
