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
        echo "Error purging database. Attempting manual cleanup..."
        if [ -d "$base_path/chains/argochain/db" ]; then
            sudo rm -rf "$base_path/chains/argochain/db"
            echo "Manual database cleanup completed."
        else
            echo "Database directory not found. Skipping manual cleanup."
        fi
    fi
}

# Purge the database first
purge_database

# Start the node in the background and save logs
log_file="$base_path/node.log"
echo "Starting the node..."
nohup ./target/release/argochain \
    --chain minervaRaw.json \
    --base-path "$base_path" \
    --port 30333 \
    --rpc-port 9944 \
    --telemetry-url "wss://telemetry.polkadot.io/submit/ 0" \
    --name "$name" \
    --validator \
    --rpc-methods Unsafe \
    --unsafe-rpc-external \
    --rpc-max-connections 100 \
    --rpc-cors all > "$log_file" 2>&1 &

# Enhanced SUCCESSFUL message
if [ $? -eq 0 ]; then
    echo -e "\033[1;32m" # Green color
    echo "=============================================================="
    echo "     🎉🎉🎉      OPERATION SUCCESSFUL!       🎉🎉🎉"
    echo "=============================================================="
    echo -e "\033[0m" # Reset color

    echo -e "\033[1;33m✅ Your Argochain node is now running successfully! ✅\033[0m"
    echo -e "\033[1;34m🌟 Logs are being written to:\033[0m \033[1;37m$log_file\033[0m"
    echo -e "\033[1;34m📝 To monitor logs in real-time, use:\033[0m"
    echo -e "\033[1;36m    tail -f $log_file\033[0m"
    echo -e "\033[1;34m🚀 Ready to validate blocks and explore Argochain! 🚀\033[0m"
else
    echo -e "\033[1;31mFailed to start the node. Check the logs at:\033[0m $log_file"
    exit 1
fi
