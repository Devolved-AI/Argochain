# Secure the WebSocket

### Secure a WS Port
A non-secure WS port can be converted to a secure WSS port by placing it behind an SSL-enabled proxy. This can be used to secure a boot node or secure a RPC server. The SSL-enabled apache2/nginx/other proxy server redirects requests to the internal ws and converts it to a secure (wss) connection. For this, you will need an SSL certificate for which you can use a service like lets-encrypt or self-signing.

### Installing a Proxy Server
There are a lot of different implementations of a WebSocket proxy, some of the more widely used are nginx and apache2, for which configuration examples are provided below.

### Nginx
First, install Nginx and Certbot:
```
apt install nginx
```
Create and enable the site:
```
cd /etc/nginx/sites-available
sudo cp default {SUB_DOMAIN}
sudo ln -s /etc/nginx/sites-available/{SUB_DOMAIN} /etc/nginx/sites-enabled/
```
Edit the site file:
```
vim {SUB_DOMAIN}
```
Change the root and server_name to get a file like this:
```
server {
    listen 80;
    listen [::]:80;

    root /var/www/{SUB_DOMAIN}/html;
    index index.html index.htm index.nginx-debian.html;

    server_name {SUB_DOMAIN};

    location / {
            try_files $uri $uri/ =404;
    }
}
```
###Generate SSL certificate
Issue the Certbot certificate:
```
sudo certbot certonly --nginx
```
Certbot will issue the SSL certificate into /etc/letsencrypt/live.
### Switch to https
Edit again the site file:
```
sudo nano {SUB_DOMAIN}
```
Delete the existing lines and set the content as below:
```
map $http_upgrade $connection_upgrade {
    default upgrade;
    '' close;
}

server {

    # SSL configuration
    #
    listen 443 ssl;
    listen [::]:443 ssl;

    root /var/www/{SUB_DOMAIN}/html;

    server_name {SUB_DOMAIN};
    ssl_certificate /etc/letsencrypt/live/{SUB_DOMAIN}/fullchain.pem; # managed by Certbot
    ssl_certificate_key /etc/letsencrypt/live/{SUB_DOMAIN}/privkey.pem; # managed by Certbot
    ssl_session_timeout 5m;
    ssl_protocols SSLv2 SSLv3 TLSv1 TLSv1.1 TLSv1.2;
    ssl_ciphers   HIGH:!aNULL:!MD5;
    ssl_prefer_server_ciphers on;

    location / {
        proxy_pass http://localhost:9944;
        proxy_pass_request_headers on;
        proxy_http_version 1.1;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection $connection_upgrade;
    }

}
```
Check and restart nginx:
```
sudo nginx -t
sudo systemctl restart nginx
```
### Usage
This is it, your arcive node is set and available from outside.

If you set a WS endpoint, you can explore the chain from the Polkadot.js portal using the format wss://{SUB_DOMAIN}

