# Argochain validator install with docker compose

Make sure, ***docker is installed***.

Clone and start the install process:

```
git clone https://github.com/Devolved-AI/Argochain.git && cd Argochain
```

```
NODE_NAME=my-node-name docker compose up -d
```

This builds the image and starts everything. It typically takes between 10-20 minutes and takes 15-20 GB disk space to build the image and start the sync process. After the build is complete, if you restart the host, the service will come up automatically.

It creates a folder `$HOME/argochain-data` and will sync THE ENTIRE BLOCKCHAIN into this folder. The sync process ***can take hours***!

# View logs
You can check the progress here:

```
docker compose logs -f --tail=50 agc-validator
```

# Get your session key
Your session key is here, it takes a few seconds until it appears:

```
cat $HOME/argochain-data/session/.session_key
```

Use it here: https://explorer.argoscan.net/#/staking/actions

Based on this guieline: https://devolved-ai.gitbook.io/argochain-validator-guide

---

# Maintenance/Rotate keys
If you would like to **rotate the keys**, delete the sesion key and restart the compose.

```
sudo rm $HOME/argochain-data/session/.session_key && docker compose down && NODE_NAME=my-node-name docker compose up -d
```

This triggers the execution of rotate_keys.sh inside the container and generates new session_key. Don't forget to re-add it here: https://explorer.argoscan.net/#/staking/actions under "Change session keys".