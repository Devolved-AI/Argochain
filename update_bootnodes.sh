#!/bin/bash

# Function to print messages
print_message() {
    MESSAGE=$1
    echo "${MESSAGE}"
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
insert_bootnodes

print_message "Boot nodes insertion complete."
