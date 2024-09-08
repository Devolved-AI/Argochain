# Argochain validator install with docker compose

This installer is for docker compose based on this guieline: https://devolved-ai.gitbook.io/argochain-validator-guide

Make sure, ***docker is installed***.

Clone and start the install process:

```
git clone https://github.com/Devolved-AI/Argochain.git & cd Argochain
```

```
NODE_NAME=my-node-name docker-compose up
```

This builds the image and starts everything. It typically takes between 10-20 minutes and takes 15-20 GB disk space to build the image and start the sync process. After the build is complete if you restart the host, the service will come up again automatically. The sync process ***can take hours***!

It creates a folder `$HOME/argochain-data` and it will sync THE WHOLE BLOCKCHAIN in this folder.

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

# Maintenance (later)
If you would like to **rotate the keys**, delete the sesion key and restart the compose.

```
rm $HOME/argochain-data/session/.session_key && docker compose restart
```

This triggers the execution of rotate_keys.sh inside the container and generates new session_key. Don't forget to re-add it here: https://explorer.argoscan.net/#/staking/actions under "Change session keys".