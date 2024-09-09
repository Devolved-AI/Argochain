#!/bin/bash

# Same script as ../rotate_keys.sh but with default values and without any interaction.

# Defaults
base_path=/var/log/argochain
chain_spec=minervaRaw.json
DEFAULT_NODE_NAME="default_node_name"
# Set validator name from first argument or fall back to default if null or not set.
name="${1:-$DEFAULT_NODE_NAME}"

if [ "$#" -ne 1 ]; then
  echo "Usage: $0 <node-name>"
  echo "Falling back to default: $DEFAULT_NODE_NAME"
fi

# Function to generate key and insert into node
generate_and_insert_key() {
    local key_type="$1"
    local scheme="$2"
    local base_path="$3"
    local chain_spec="$4"
    local name="$5"

    echo "Generating $key_type key..."
    key_output=$(./target/release/argochain key generate --scheme "$scheme" --output-type json)

    # Extract secret phrase and public key from key output
    secret_phrase=$(echo "$key_output" | jq -r '.secretPhrase')
    public_key=$(echo "$key_output" | jq -r '.publicKey')

    echo "Inserting $key_type key..."
    ./target/release/argochain key insert --base-path "$base_path" --chain "$chain_spec" --scheme "$scheme" --suri "$secret_phrase" --key-type "$key_type"

    if [ $? -eq 0 ]; then
        echo "$key_type key inserted. Public key: $public_key"
    else
        echo "Error inserting $key_type key."
    fi
}

# Function to rotate keys
rotate_key() {
    local key_type="$1"
    local scheme="$2"
    local base_path="$3"
    local chain_spec="$4"
    local name="$5"

    echo "Rotating $key_type key..."
    generate_and_insert_key "$key_type" "$scheme" "$base_path" "$chain_spec"
    echo "$key_type key rotated."
}

# Rotate keys for all consensus mechanisms
rotate_key "babe" "Sr25519" "$base_path" "$chain_spec"
rotate_key "gran" "Ed25519" "$base_path" "$chain_spec"
rotate_key "audi" "Sr25519" "$base_path" "$chain_spec"
rotate_key "imon" "Sr25519" "$base_path" "$chain_spec"

echo "Node has now been injected with new validator keys."
