## Argochain Testnet

# Requirements
Prepare your development environment [(Instructions)](https://docs.substrate.io/install/)

### Clone the Repo
```
git clone https://github.com/mitun567/Argochain.git
```
### Now Change the Directory
```
cd Argochain
```
### Don't forget to source the cargo env file
```
source ~/.cargo/env
```
# ⛳ Getting Started

Use this **QuickStart** command to build and launch the node:

```bash
cargo run --release -- --dev
```
By running the above command, all the necessary components will be pulled and the Argochain node will be started in development mode.

### List of all commands

The following command can be used to explore all parameters and subcommands:

```
./target/release/argochain -h
```
# 💻 <a name="dev-ecosystem">Development Ecosystem (Single Node)</a>

The provided `cargo run` command will launch a temporary node and its state will be discarded after you terminate the process. Use the following command to build the node without launching it:

```
cargo build --release
```

After the project has been built, you can see the binary in the location `./target/release/argochain`.

---
## purge-chain for a validator (If the node is running node and wants to run again.)
```
./target/release/argochain purge-chain --base-path /tmp/node01 --chain customSpecRaw.json
./target/release/argochain purge-chain --base-path /tmp/node02 --chain customSpecRaw.json
./target/release/argochain purge-chain --base-path /tmp/node03 --chain customSpecRaw.json
./target/release/argochain purge-chain --base-path /tmp/node04 --chain customSpecRaw.json
./target/release/argochain purge-chain --base-path /tmp/node05 --chain customSpecRaw.json
./target/release/argochain purge-chain --base-path /tmp/node06 --chain customSpecRaw.json
```
### Now delete the temp file
```
rm -rf /tmp/node01 node02 node03 node04 node05 node06
```
## Add the keys for Boot/RPC Node.
We need to add the keys to each validator node using the command:
```
./target/release/argochain key insert --base-path /tmp/node01 \
--chain ./customSpecRaw.json \
--scheme Sr25519 \
--suri 0x202641b917fbf0286b9042cf2b7b8669146b6b73607990bf99e177e4867228b0 \
--password-interactive \
--key-type babe
```
```
./target/release/argochain key insert --base-path /tmp/node01 \
--chain ./customSpecRaw.json \
--scheme Ed25519 \
--suri 0xc912ef91d7fcaf8cd73d11d637f74680ec0cbd32cba50bbb5e45a272b3cda43a \
--password-interactive \
--key-type gran
```
```
./target/release/argochain key insert --base-path /tmp/node01 \
--chain ./customSpecRaw.json \
--scheme Sr25519 \
--suri 0x6edca157aaf7764c9d5b1d9128fd05a1661372f91687085131d04646118cce7e \
--password-interactive \
--key-type imon
```
```
./target/release/argochain key insert --base-path /tmp/node01 \
--chain ./customSpecRaw.json \
--scheme Sr25519 \
--suri 0xe020a42d9084762730a768c1ba2254186770b77d39e82899206429b050f7cc19 \
--password-interactive \
--key-type audi
```
### Commands to start the Boot/RPC node
```
 nohup ./target/release/argochain \
--base-path /tmp/node01 \
--chain ./customSpecRaw.json \
--port 30333 \
--rpc-port 9944 \
--telemetry-url "wss://telemetry.polkadot.io/submit/ 0" \
--validator \
--rpc-methods Unsafe \
--rpc-max-connections 10000 \
--rpc-cors all \
--name MyNode01 &
```
### To find the keys 
```
cat nohup.out
```
KEY: 12D3KooWMaLkCFxbF4bQvi2R5bqWXtRyemdfnxp3EEsC19m3AT7A

## Add keys for Validator 2

### Port Forwarding: (Optional If we run on a different server)
```
ssh -i web-svr-kp-ohio.pem -L 9945:localhost:9945 -f -N USERNAME@Public-IP
```
We need to add the keys to each validator node using the command:
```
./target/release/argochain key insert --base-path /tmp/node02 \
--chain ./customSpecRaw.json \
--scheme Sr25519 \
--suri 0x7d37ea3331962daa1f6abb7e5743c4e3bd4cd5b976819151fac674dc884618d1 \
--password-interactive \
--key-type babe
```
```
./target/release/argochain key insert --base-path /tmp/node02 \
--chain ./customSpecRaw.json \
--scheme Ed25519 \
--suri 0xc8e2d366c7d28161b194f225d0278120812319c329fbfc9043ea517e0b2da129 \
--password-interactive \
--key-type gran
```
```
./target/release/argochain key insert --base-path /tmp/node02 \
--chain ./customSpecRaw.json \
--scheme Sr25519 \
--suri 0x685d8f16de5fb38ae202b7ae80ef72ae46a26d7f624bd1bb18d0fcfd7c06919a \
--password-interactive \
--key-type imon
```
```
./target/release/argochain key insert --base-path /tmp/node02 \
--chain ./customSpecRaw.json \
--scheme Sr25519 \
--suri 0x3feeda2930f647927e9f7c91091735581d3fd90150b936c7432885ac9aeb49a8 \
--password-interactive \
--key-type audi
```
### Commands to start the validator node
### For Local
```
nohup ./target/release/argochain \
--base-path /tmp/node02 \
--chain ./customSpecRaw.json \
--port 30334 \
--rpc-port 9946 \
--telemetry-url "wss://telemetry.polkadot.io/submit/ 0" \
--validator \
--rpc-methods Unsafe \
--name MyNode02 \
--bootnodes /ip4/127.0.0.1/tcp/30333/p2p/12D3KooWHFsxVGGgbcqNAybh27X73L6x33LpXtdAEumtpDfopnoE &
```
### For Global
```
 nohup ./target/release/argochain \
  --base-path /tmp/node02 \
  --chain customSpecRaw.json \
  --port 30334 \
  --rpc-port 9946 \
  --telemetry-url "wss://telemetry.polkadot.io/submit/ 0" \
  --validator \
  --rpc-methods Unsafe \
  --unsafe-rpc-external \
  --rpc-cors all \
  --name MyNode02 \
  --bootnodes /ip4/<IP-ADDRESS>/tcp/30333/p2p/12D3KooWGdzJLDdDtWTEubMpWmoqDKrpbXNru4FUo7eWNrsXgME8 &
```
### Add keys for Validator 3
### Port Forwarding: (Optional If we run on different servers)
```
ssh -i web-svr-kp-ohio.pem -L 9945:localhost:9945 -f -N USERNAME@Public-IP
```
We need to add the keys to each validator node using the command:
```
./target/release/argochain key insert --base-path /tmp/node03 \
--chain ./customSpecRaw.json \
--scheme Sr25519 \
--suri 0xdeea43096c89a250f2066e630930b6526190afed744e1084510863a18e26ea4f \
--password-interactive \
--key-type babe
```
```
./target/release/argochain key insert --base-path /tmp/node03 \
--chain ./customSpecRaw.json \
--scheme Ed25519 \
--suri 0xa9425b499cd82b5e01f23a85221090235921445ce6d60cac640717a8351f2701 \
--password-interactive \
--key-type gran
```
```
./target/release/argochain key insert --base-path /tmp/node03 \
--chain ./customSpecRaw.json \
--scheme Sr25519 \
--suri 0x27abdb874f0a6ad9a1478ad41f2d3ebb1b566dd5ebc71d9478c89b3bd9607647 \
--password-interactive \
--key-type imon
```
```
./target/release/argochain key insert --base-path /tmp/node03 \
--chain ./customSpecRaw.json \
--scheme Sr25519 \
--suri 0x1a12b0c6c5f5e59e38d6acf9e08da442d346322aba6a23dd3761be5002898ae1 \
--password-interactive \
--key-type audi
```
### Commands to start the validator node
### For Local
```
nohup ./target/release/argochain \
--base-path /tmp/node03 \
--chain ./customSpecRaw.json \
--port 30335 \
--rpc-port 9947 \
--telemetry-url "wss://telemetry.polkadot.io/submit/ 0" \
--validator \
--rpc-methods Unsafe \
--name MyNode03 \
--bootnodes /ip4/127.0.0.1/tcp/30333/p2p/12D3KooWHFsxVGGgbcqNAybh27X73L6x33LpXtdAEumtpDfopnoE &
```
### For Global
```
 nohup ./target/release/argochain \
  --base-path /tmp/node03 \
  --chain customSpecRaw.json \
  --port 30335 \
  --rpc-port 9947 \
  --telemetry-url "wss://telemetry.polkadot.io/submit/ 0" \
  --validator \
  --rpc-methods Unsafe \
  --unsafe-rpc-external \
  --rpc-cors all \
  --name MyNode03 \
  --bootnodes /ip4/<IP-ADDRESS>/tcp/30333/p2p/12D3KooWGdzJLDdDtWTEubMpWmoqDKrpbXNru4FUo7eWNrsXgME8 &
```
### Add keys for Validator 4
### Port Forwarding: (Optional If we run on a different server)
```
ssh -i web-svr-kp-ohio.pem -L 9945:localhost:9945 -f -N USERNAME@Public-IP
```
We need to add the keys to each validator node using the command:
```
./target/release/argochain key insert --base-path /tmp/node04 \
--chain ./customSpecRaw.json \
--scheme Sr25519 \
--suri 0xf5a6f9910fd4ad366b9cb891cee6865ec97a759470a1c547fae89c48b68c89e9 \
--password-interactive \
--key-type babe
```
```
./target/release/argochain key insert --base-path /tmp/node04 \
--chain ./customSpecRaw.json \
--scheme Ed25519 \
--suri 0xe506c3e47d9176834961a153d90fcffad885deaa0d064c2662bcda843c1842d0 \
--password-interactive \
--key-type gran
```
```
./target/release/argochain key insert --base-path /tmp/node04 \
--chain ./customSpecRaw.json \
--scheme Sr25519 \
--suri 0x931f66eab8803e07fb489466776e806ed56aab3e552a86631aed55ea008af484 \
--password-interactive \
--key-type imon
```
```
./target/release/argochain key insert --base-path /tmp/node04 \
--chain ./customSpecRaw.json \
--scheme Sr25519 \
--suri 0xa1dfcec096987c27da38289c12095faa3e8009d779d48d64eacb5f48eb6d14f5 \
--password-interactive \
--key-type audi
```
### Commands to start the validator node
### For Local
```
nohup ./target/release/argochain \
--base-path /tmp/node04 \
--chain ./customSpecRaw.json \
--port 30336 \
--rpc-port 9948 \
--telemetry-url "wss://telemetry.polkadot.io/submit/ 0" \
--validator \
--rpc-methods Unsafe \
--name MyNode04 \
--bootnodes /ip4/127.0.0.1/tcp/30333/p2p/12D3KooWHFsxVGGgbcqNAybh27X73L6x33LpXtdAEumtpDfopnoE &
```
### For Global
```
 nohup ./target/release/argochain \
--base-path /tmp/node04 \
--chain ./customSpecRaw.json \
--port 30336 \
--rpc-port 9948 \
--telemetry-url "wss://telemetry.polkadot.io/submit/ 0" \
--validator \
--rpc-methods Unsafe \
--unsafe-rpc-external \
--rpc-cors all \
--name MyNode04 \
--bootnodes /ip4/<IP-ADDRESS>/tcp/30333/p2p/12D3KooWGdzJLDdDtWTEubMpWmoqDKrpbXNru4FUo7eWNrsXgME8 &
```
### Add keys for Validator 5
### Port Forwarding: (Optional If we run on a different server)
```
ssh -i web-svr-kp-ohio.pem -L 9945:localhost:9945 -f -N USERNAME@Public-IP
```
We need to add the keys to each validator node using the command:
```
./target/release/argochain key insert --base-path /tmp/node05 \
--chain ./customSpecRaw.json \
--scheme Sr25519 \
--suri 0xbc5322aa44ee46a3190ba84d928b54eb502f4974cc45c3e86c06319e58b0b57c \
--password-interactive \
--key-type babe
```
```
./target/release/argochain key insert --base-path /tmp/node05 \
--chain ./customSpecRaw.json \
--scheme Ed25519 \
--suri 0x2873b45705a4320900e1e3782e4b45a7e8ea99676a216fff0329a06ffb4b7bf1 \
--password-interactive \
--key-type gran
```
```
./target/release/argochain key insert --base-path /tmp/node05 \
--chain ./customSpecRaw.json \
--scheme Sr25519 \
--suri 0x400e2d598ba2c25dc919ab6d67911ff173022c8002671a43936ab88139540b4d \
--password-interactive \
--key-type imon
```
```
./target/release/argochain key insert --base-path /tmp/node05 \
--chain ./customSpecRaw.json \
--scheme Sr25519 \
--suri 0xb6c1ae7492636c82796cc8ab25e467e0df2cc30aceff0bef83894e6a2d2ca1b9 \
--password-interactive \
--key-type audi
```
## Commands to start the validators
## For Local
```
nohup ./target/release/argochain \
--base-path /tmp/node05 \
--chain ./customSpecRaw.json \
--port 30337 \
--rpc-port 9949 \
--telemetry-url "wss://telemetry.polkadot.io/submit/ 0" \
--validator \
--rpc-methods Unsafe \
--name MyNode05 \
--bootnodes /ip4/127.0.0.1/tcp/30333/p2p/12D3KooWLSZ47SEDCwEvRRRg3efqhfUoYQocwusqbyRX68AGm2Wz &
```
## For Global
```
nohup ./target/release/argochain \
--base-path /tmp/node05 \
--chain ./customSpecRaw.json \
--port 30337 \
--rpc-port 9949 \
--telemetry-url "wss://telemetry.polkadot.io/submit/ 0" \
--validator \
--rpc-methods Unsafe \
--unsafe-rpc-external \
--rpc-cors all \
--name MyNode05 \
--bootnodes /ip4/<IP-ADDRESS>/tcp/30333/p2p/12D3KooWLSZ47SEDCwEvRRRg3efqhfUoYQocwusqbyRX68AGm2Wz &
```
### Add keys for Validator 6
### Port Forwarding: (Optional If we run on a different server)
```
ssh -i web-svr-kp-ohio.pem -L 9945:localhost:9945 -f -N USERNAME@Public-IP
```
We need to add the keys to each validator node using the command:
```
./target/release/argochain key insert --base-path /tmp/node06 \
--chain ./customSpecRaw.json \
--scheme Sr25519 \
--suri 0x2872ffa33b1119ade850da082e91e729761e3f109168da9167164f499ed3238d \
--password-interactive \
--key-type babe
```
```
./target/release/argochain key insert --base-path /tmp/node06 \
--chain ./customSpecRaw.json \
--scheme Ed25519 \
--suri 0x0f193d05c2361df2dcc39b1f0f703c1ed2abf05c7c2f9f3a811e66396e6addea \
--password-interactive \
--key-type gran
```
```
./target/release/argochain key insert --base-path /tmp/node06 \
--chain ./customSpecRaw.json \
--scheme Sr25519 \
--suri 0x4597a9cc172226d9ca0e9cdf2acb4299affa1dbdecf86c6245d12f4394354b3b \
--password-interactive \
--key-type imon
```
```
./target/release/argochain key insert --base-path /tmp/node06 \
--chain ./customSpecRaw.json \
--scheme Sr25519 \
--suri 0xca66544e769de5e3ab49194b8233674dae1171c6d413101ebf40e583932258df \
--password-interactive \
--key-type audi
```
## Commands to start the validators
## For Local
```
nohup ./target/release/argochain \
--base-path /tmp/node06 \
--chain ./customSpecRaw.json \
--port 30338 \
--rpc-port 9950 \
--telemetry-url "wss://telemetry.polkadot.io/submit/ 0" \
--validator \
--rpc-methods Unsafe \
--name MyNode06 \
--bootnodes /ip4/127.0.0.1/tcp/30333/p2p/12D3KooWLSZ47SEDCwEvRRRg3efqhfUoYQocwusqbyRX68AGm2Wz &
```
## For Global
```
nohup ./target/release/argochain \
--base-path /tmp/node06 \
--chain ./customSpecRaw.json \
--port 30338 \
--rpc-port 9950 \
--telemetry-url "wss://telemetry.polkadot.io/submit/ 0" \
--validator \
--rpc-methods Unsafe \
--unsafe-rpc-external \
--rpc-cors all \
--name MyNode06 \
--bootnodes /ip4/<IP-ADDRESS>/tcp/30333/p2p/12D3KooWLSZ47SEDCwEvRRRg3efqhfUoYQocwusqbyRX68AGm2Wz &
```