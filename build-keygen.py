import subprocess
import sys
import shutil

def check_command(command):
    """Check if a command is available on the system."""
    if shutil.which(command) is None:
        print(f"Error: {command} is not installed or not found in PATH.")
        sys.exit(1)

def run_command(command, input_text=None, shell=False):
    """Run a system command and handle errors."""
    try:
        result = subprocess.run(command, input=input_text, text=True, capture_output=True, check=True, shell=shell)
        print(f"Successfully ran: {' '.join(command) if not shell else command}")
        return result.stdout.strip()
    except subprocess.CalledProcessError as e:
        print(f"Error running command: {' '.join(command) if not shell else command}: {e}")
        sys.exit(1)

def build_project():
    """Build the project in release mode."""
    print("Building the project in release mode...")
    run_command(["cargo", "build", "--release"])

def generate_keys(password, output_file):
    """Generate keys and save them to a file with schema labels."""
    print("Generating keys...")
    schemes = ["sr25519", "sr25519", "sr25519", "ed25519"]
    with open(output_file, "w") as f:
        for scheme in schemes:
            key_output = run_command(["./target/release/argochain", "key", "generate", "--scheme", scheme, "--password-interactive"], input_text=password + '\n')
            secret_seed_line = next((line for line in key_output.split('\n') if line.startswith("secret seed")), None)
            if secret_seed_line:
                f.write(f"{scheme}: {secret_seed_line}\n")

def insert_keys(password, keys_file):
    """Insert keys from the file, ensuring unique keys."""
    print("Inserting keys...")
    key_types = [
        ("sr25519", "babe"),
        ("ed25519", "gran"),
        ("sr25519", "imon"),
        ("sr25519", "audi")
    ]
    
    with open(keys_file, "r") as f:
        keys = f.read().splitlines()

    used_keys = set()
    for scheme, key_type in key_types:
        for line in keys:
            if line.startswith(f"{scheme}:"):
                key = next((part for part in line.split() if part.startswith("0x")), None)
                if key and key not in used_keys:
                    used_keys.add(key)
                    print(f"Inserting key with scheme {scheme} and key type {key_type}")
                    run_command(["./target/release/argochain", "key", "insert", "--scheme", scheme, "--password-interactive", "--key-type", key_type], input_text=password + '\n')
                    break

def main():
    # Check if necessary commands are available
    check_command("cargo")
    check_command("python3")

    password = "5683"
    output_file = "keys.txt"

    # Build the project
    build_project()

    # Generate keys and insert them, do this twice
    for _ in range(2):
        generate_keys(password, output_file)
        insert_keys(password, output_file)

if __name__ == "__main__":
    main()
