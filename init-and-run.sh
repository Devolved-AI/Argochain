#!/bin/bash

# One-time initialization
if [ ! -f /session_key/.session_key ]; then
  echo ".session_key not found."

  echo "Starting rotate_keys_docker.sh"
  ./rotate_keys_docker.sh ${NODE_NAME}

  # Tricky part: we have to start the server first to query the session key. So we put the process in the backgournd with some sleep and it tries to connect and get the session key a few seconds later. We have to start the server anyways, so we do it outside of the if statement.
    (
        sleep 10 && \
        result=$(curl -s -H "Content-Type: application/json" --data '{"jsonrpc":"2.0", "method":"author_rotateKeys", "params":[], "id":1}' http://localhost:9944 | jq -r '.result') && \
        if [ -n "$result" ]; then
            echo "$result" > /session_key/.session_key
            echo ".session_key file created."
        fi
    ) &
fi

# Start the main application
/app/target/release/argochain --chain minervaRaw.json --base-path /var/opt/argochain --port 30333 --rpc-port 9944 --telemetry-url "wss://telemetry.polkadot.io/submit/ 0" --name "${NODE_NAME}" --validator --rpc-methods Unsafe --unsafe-rpc-external --rpc-max-connections 100 --rpc-cors all
