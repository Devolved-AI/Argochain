#!/bin/bash
# Same script as rotate_keys.sh but with default values and without any interaction:
# Default base path: /var/log/argochain
# Default chain specification file: minervaRaw.json
# Validator name: <argument1>

if [ "$#" -ne 1 ]; then
  echo "Usage: $0 <validator-name>"
  exit 1
fi

# Function to prompt user for input with a custom message
prompt_for_input() {
    local prompt_message="$1"
    local input_variable
    read -p "$prompt_message" input_variable
    echo "$input_variable"
}

# Default base path
base_path=/var/log/argochain

# Default chain specification file
chain_spec=minervaRaw.json

# Set validator name (First argument in command line)
name=$1

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
    sudo ./target/release/argochain key insert --base-path "$base_path" --chain "$chain_spec" --scheme "$scheme" --suri "$secret_phrase" --key-type "$key_type"

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
./target/release/argochain --chain $chain_spec --name $name --validator --base-path $base_path
