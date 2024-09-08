#!/bin/bash

# Define the file name
FILE="/session/NODE_NAME"
SESSION_KEY_FILE="/session/.session_key"

# Define the default value
DEFAULT_NODE="mynode"

# Check if a parameter is provided for NODE_NAME
if [ -n "$1" ]; then
  # Write the parameter to the file and set NODE_NAME to the parameter
  echo "$1" > "$FILE"
  NODE_NAME="$1"
else
  # No parameter provided, check if the file exists
  if [ -f "$FILE" ]; then
    # Read the first word from the file
    NODE_NAME=$(head -n 1 "$FILE" | cut -d ' ' -f 1)
  else
    # File does not exist, use the default value
    NODE_NAME="$DEFAULT_NODE"
  fi
fi

# One-time initialization
if [ ! -f "$SESSION_KEY_FILE" ] || [ ! -s "$SESSION_KEY_FILE" ]; then
  echo "Session key not found. Starting rotate_keys_docker.sh"
  ./rotate_keys_docker.sh ${NODE_NAME}

  # Tricky part: we have to start the server first to query the session key. So we put the process in the backgournd with some sleep and it tries to connect and get the session key a few seconds later. We have to start the server anyways, so we do it outside of the if statement.
    (
        sleep 10 && \
        result=$(curl -s -H "Content-Type: application/json" --data '{"jsonrpc":"2.0", "method":"author_rotateKeys", "params":[], "id":1}' http://localhost:9944 | jq -r '.result') && \
        if [ -n "$result" ]; then
            echo "$result" > $SESSION_KEY_FILE
            echo "$SESSION_KEY_FILE created."
        fi
    ) &
fi

# Start the main application
/app/target/release/argochain --chain minervaRaw.json --base-path /var/opt/argochain --port 30333 --rpc-port 9944 --telemetry-url "wss://telemetry.polkadot.io/submit/ 0" --name "${NODE_NAME}" --validator --rpc-methods Unsafe --unsafe-rpc-external --rpc-max-connections 100 --rpc-cors all
