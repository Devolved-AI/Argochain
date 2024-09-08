#!/bin/bash

# One-time initialization
if [ ! -f /argochain/.session_key ]; then
  echo ".session_key not found."
  echo "Starting rotate_keys_docker.sh"
  $SHELL ./rotate_keys_docker.sh ${NODE_NAME}
  echo "Creating .session_key file."
  # Tricky part: we have to start the server first to query the session key. So we put the process in the backgournd with some sleep and it tries to connect and get session keya few seconds later. We have to start the server anyways, so we do it outside of the if statement.
  sleep 10 && curl -s -H "Content-Type: application/json" --data '{"jsonrpc":"2.0", "method":"author_rotateKeys", "params":[], "id":1}' http://localhost:9944 | jq -r '.result' > /argochain/.session_key &
fi

# Start the main application
/app/target/release/argochain --chain minervaRaw.json --base-path /var/opt/argochain --port 30333 --rpc-port 9944 --telemetry-url "wss://telemetry.polkadot.io/submit/ 0" --name "${NODE_NAME}" --validator --rpc-methods Unsafe --unsafe-rpc-external --rpc-max-connections 100 --rpc-cors all
