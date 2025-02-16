#!/bin/bash

# This is the script which is used to initialize the containers.

DATA_DIR="data/"

# Docker does not expose the service name as an environment variable.
# So we have to retrieve it through the docker.socket API.
export NAME=\
$(curl -X GET --unix-socket /var/run/docker.sock -s "http://v1.43/containers/$HOSTNAME/json" \
| jq -r '.Name' \
| sed "s/['\"/\\/]//g") # Sanitize output from extra symbols like '/' or '\"'

# Same for the host port to which the container's `APP_PORT` port is bound to.
export PORT=\
$(curl -X GET --unix-socket /var/run/docker.sock -s "http://v1.43/containers/$HOSTNAME/json" \
| jq -r --arg app_port "$APP_PORT" '.NetworkSettings.Ports[$app_port + "/tcp"][0].HostPort' \
| sed "s/['\"/\\/]//g")

# Save port number to file.
mkdir -p /$DATA_DIR/$NAME
echo "$PORT" > $DATA_DIR/$NAME/port

# Run the authority application.
/exec/debug/${APP} --data-path $DATA_DIR/$NAME
