#!/bin/bash

SESSION_KEY_FILE="/session/.session_key"
NODE_NAME="$1"

# One-time initialization
if [ ! -f "$SESSION_KEY_FILE" ] || [ ! -s "$SESSION_KEY_FILE" ]; then
  echo "Session key not found. Starting rotate_keys_docker.sh"
  ./rotate_keys_docker.sh

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
