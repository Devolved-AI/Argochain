#!/bin/bash

# Function to prompt user for input with a custom message
prompt_for_input() {
    local prompt_message="$1"
    local input_variable
    read -p "$prompt_message" input_variable
    echo "$input_variable"
}

# Prompt user for inputs
base_path=$(prompt_for_input "Enter the base path for the node (e.g., /var/log/argochain): ")
chain_spec=$(prompt_for_input "Enter the chain specification file (e.g., minervaRaw.json): ")
name=$(prompt_for_input "Give your node a unique name (e.g., BestValidatorEver): ")

# Ensure inputs are absolute paths
chain_spec=$(realpath "$chain_spec")
base_path=$(realpath "$base_path")

# Ensure required tools and binaries exist
if ! command -v jq &> /dev/null; then
    echo "jq is not installed. Please install it and try again."
    exit 1
fi

if [ ! -f "./target/release/argochain" ]; then
    echo "argochain binary not found. Please build it and try again."
    exit 1
fi

# Ensure base path exists
if [ ! -d "$base_path" ]; then
    echo "Base path does not exist. Creating it now..."
    mkdir -p "$base_path"
fi

# Set proper permissions for the base path
sudo chown -R $(whoami) "$base_path"

# Function to purge the database
purge_database() {
    echo "Purging database to prevent stale state conflicts..."
    ./target/release/argochain purge-chain --chain "$chain_spec" --base-path "$base_path" -y
    if [ $? -eq 0 ]; then
        echo "Database purged successfully."
    else
        echo "Error purging database. Exiting."
        exit 1
    fi
}

# Function to generate and insert a key
generate_and_insert_key() {
    local key_type="$1"
    local scheme="$2"

    echo "Generating $key_type key..."
    key_output=$(./target/release/argochain key generate --scheme "$scheme" --output-type json)

    # Extract secret phrase and public key
    secret_phrase=$(echo "$key_output" | jq -r '.secretPhrase')
    public_key=$(echo "$key_output" | jq -r '.publicKey')

    echo "Inserting $key_type key..."
    ./target/release/argochain key insert --base-path "$base_path" --chain "$chain_spec" --scheme "$scheme" --suri "$secret_phrase" --key-type "$key_type"

    if [ $? -eq 0 ]; then
        echo "$key_type key inserted successfully. Public key: $public_key"
    else
        echo "Error inserting $key_type key. Exiting."
        exit 1
    fi
}

# Function to rotate keys for a specific type
rotate_key() {
    local key_type="$1"
    local scheme="$2"

    echo "Rotating $key_type key..."
    generate_and_insert_key "$key_type" "$scheme"
    echo "$key_type key rotated successfully."
}

# Purge the database first
purge_database

# Rotate keys for all consensus mechanisms
rotate_key "babe" "Sr25519"
rotate_key "gran" "Ed25519"
rotate_key "audi" "Sr25519"
rotate_key "imon" "Sr25519"

# Start the node with injected keys
echo "Starting the node with the new validator keys..."
./target/release/argochain --chain "$chain_spec" --name "$name" --validator --base-path "$base_path"
