#!/bin/bash

# Function to print messages
print_message() {
    MESSAGE=$1
    echo "${MESSAGE}"
}

# Function to create and activate a virtual environment
create_and_activate_venv() {
    VENV_DIR=".venv"

    # Check if the virtual environment already exists
    if [ ! -d "$VENV_DIR" ]; then
        echo "Creating virtual environment in ${VENV_DIR}..."
        python3 -m venv "$VENV_DIR"
    fi

    # Activate the virtual environment
    echo "Activating virtual environment..."
    source "${VENV_DIR}/bin/activate"

    # Upgrade pip within the virtual environment
    pip install --upgrade pip
}

# Function to install tqdm using pip
install_tqdm() {
    # Attempt to install tqdm
    pip install tqdm

    # Check if tqdm was successfully installed
    if python3 -m pip show tqdm &> /dev/null; then
        echo "tqdm is successfully installed."
        return 0
    else
        echo "tqdm installation failed. Falling back to ASCII progress bar."
        return 1
    fi
}

# Function to insert bootnodes into minervaRaw.json
insert_bootnodes() {
    python3 <<EOF
import re
import time

try:
    from tqdm import tqdm

    def progress_bar(iterable, desc):
        return tqdm(iterable, desc=desc, ncols=100, ascii=True, bar_format="{l_bar}{bar} | {n_fmt}/{total_fmt}")

except ImportError:
    # Fallback ASCII progress bar
    def progress_bar(iterable, desc):
        total = len(iterable)
        print(f"{desc} [{'#' * total}]")
        for i, item in enumerate(iterable):
            yield item
            print(f"\r{desc} [{'#'*(i+1)}{'.'*(total-i-1)}] | {i+1}/{total}", end='')
        print()

def insert_bootnodes(original_file, bootnodes_file):
    try:
        with open(original_file, 'r') as file:
            original_content = file.read()

        with open(bootnodes_file, 'r') as file:
            bootnodes_content = file.read().strip()

        # Progress bar for processing the content
        for _ in progress_bar(range(10), desc="🌟 Processing content"):
            time.sleep(0.1)  # Simulate work being done

        # Find the bootNodes section, clear its contents, and insert the new bootnodes content
        pattern = re.compile(r'("bootNodes"\\s*:\\s*\\[)[^\\]]*(\\])', re.DOTALL)
        new_content = pattern.sub(r'\\1\n' + bootnodes_content + r'\\2', original_content)

        # Progress bar for writing the new content
        for _ in progress_bar(range(10), desc="🌟 Writing new content"):
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
    for _ in progress_bar(range(10), desc="📄 Reading files"):
        time.sleep(0.1)  # Simulate work being done

    insert_bootnodes(original_file, bootnodes_file)

if __name__ == "__main__":
    main()
EOF
}

# Main script execution
create_and_activate_venv
install_tqdm && insert_bootnodes || insert_bootnodes

print_message "Boot nodes insertion complete."
