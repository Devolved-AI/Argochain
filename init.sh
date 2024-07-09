#!/bin/bash

# Function to print messages
print_message() {
    MESSAGE=$1
    echo "${MESSAGE}"
}

# Function to check the OS
check_os() {
    if [[ "$OSTYPE" == "linux-gnu"* ]]; then
        OS="linux"
    elif [[ "$OSTYPE" == "darwin"* ]]; then
        OS="macos"
    else
        print_message "Unsupported OS: $OSTYPE"
        exit 1
    fi
    print_message "Detected OS: $OS"
}

# Function to install a package if it's not already installed
install_if_not_installed() {
    PACKAGE=$1
    if [[ "$OS" == "linux" ]]; then
        if ! dpkg -l | grep -q "$PACKAGE"; then
            print_message "Installing $PACKAGE..."
            sudo apt-get install -y "$PACKAGE" > /dev/null 2>&1
        else
            print_message "$PACKAGE is already installed."
        fi
    elif [[ "$OS" == "macos" ]]; then
        if ! brew list -1 | grep -q "$PACKAGE"; then
            print_message "Installing $PACKAGE..."
            brew install "$PACKAGE" > /dev/null 2>&1
        else
            print_message "$PACKAGE is already installed."
        fi
    fi
}

# Function to install dependencies based on the OS
install_dependencies() {
    if [[ "$OS" == "linux" ]]; then
        print_message "Installing protobuf-compiler..."
        install_if_not_installed "protobuf-compiler"

        print_message "Installing build dependencies for Substrate..."
        dependencies=("build-essential" "clang" "libclang-dev" "curl" "libssl-dev" "llvm" "libudev-dev" "pkg-config" "zlib1g-dev" "git")
        for package in "${dependencies[@]}"; do
            install_if_not_installed "$package"
        done
    elif [[ "$OS" == "macos" ]]; then
        print_message "Installing protobuf..."
        brew install protobuf > /dev/null 2>&1

        print_message "Installing build dependencies for Substrate..."
        dependencies=("clang" "cmake" "pkg-config" "openssl" "llvm" "protobuf" "git")
        for package in "${dependencies[@]}"; do
            install_if_not_installed "$package"
        done
    fi
}

# Function to source the environment
source_environment() {
    if [[ "$OS" == "linux" ]]; then
        if [ -f "$HOME/.cargo/env" ]; then
            source "$HOME/.cargo/env"
            print_message "Sourced cargo environment for Linux."
        else
            print_message "Cargo environment file not found for Linux."
        fi
    elif [[ "$OS" == "macos" ]]; then
        if [ -f "/usr/local/bin/cargo/env" ]; then
            source "/usr/local/bin/cargo/env"
            print_message "Sourced cargo environment for macOS."
        else
            print_message "Cargo environment file not found for macOS."
        fi
    fi
}

# Function to install Python3
install_python3() {
    print_message "Installing Python3 and pip..."
    install_if_not_installed "python3"
    install_if_not_installed "python3-pip"
    print_message "Installing tqdm for Python..."
    pip3 install tqdm > /dev/null 2>&1
}

# Function to insert bootnodes into minervaRaw.json
insert_bootnodes() {
    python3 <<EOF
import re
import time
from tqdm import tqdm

def insert_bootnodes(original_file, bootnodes_file):
    try:
        with open(original_file, 'r') as file:
            original_content = file.read()

        with open(bootnodes_file, 'r') as file:
            bootnodes_content = file.read().strip()

        # Progress bar for processing the content
        for _ in tqdm(range(10), desc="🌟 Processing content", ncols=100, ascii=True, bar_format="{l_bar}{bar} | {n_fmt}/{total_fmt}"):
            time.sleep(0.1)  # Simulate work being done

        # Find the bootNodes section, clear its contents, and insert the new bootnodes content
        pattern = re.compile(r'("bootNodes"\\s*:\\s*\\[)[^\\]]*?(\\])', re.DOTALL)
        new_content = pattern.sub(r'\\1\n' + bootnodes_content + r'\\2', original_content)

        # Progress bar for writing the new content
        for _ in tqdm(range(10), desc="🌟 Writing new content", ncols=100, ascii=True, bar_format="{l_bar}{bar} | {n_fmt}/{total_fmt}"):
            time.sleep(0.1)  # Simulate work being done

        # Write the modified content back to the original file
        with open(original_file, 'w') as file:
            file.write(new_content)

        print(f"Successfully inserted bootnodes into {original_file}")

    except Exception as e:
        print(f"An error occurred: {e}")

def main():
    original_file = 'minervaRaw.json'  # Path to the original JSON file
    bootnodes_file = 'bootnodes.txt'   # Path to the bootnodes file

    # Progress bar for reading files
    for _ in tqdm(range(10), desc="📄 Reading files", ncols=100, ascii=True, bar_format="{l_bar}{bar} | {n_fmt}/{total_fmt}"):
        time.sleep(0.1)  # Simulate work being done

    insert_bootnodes(original_file, bootnodes_file)

if __name__ == "__main__":
    main()
EOF
}

# Main script execution
check_os
install_dependencies
source_environment
install_python3
insert_bootnodes

print_message "Installation and setup complete. Please restart your terminal."
