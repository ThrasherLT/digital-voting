#!/bin/bash

# Use this file to startup the testing environment.

DATA_DIR="data/"

# If docker-compose creates the data dir, it's owner will be root and the services won't
# be able to write anything to it.
if [ ! -d "$DATA_DIR" ]; then
  echo "Error: Directory '$DATA_DIR' must exist and be owned by the current user."
  exit 1
fi

dir_owner=$(stat -c '%U' $DATA_DIR)
if [ "$dir_owner" != $(whoami) ]; then
  echo "Error: directory '$DATA_DIR' exists, but must be owned by the current user"
  exit 1
fi

check_health() {
  for container in $(docker-compose ps -q authority); do
    health_status=$(docker inspect --format '{{.State.Health.Status}}' $container)
    if [ "$health_status" != "healthy" ]; then
      return 1
    fi
  done
  return 0
}

docker-compose up -d authority

# We need to wait for containers to be healthy, because `docker-compose up` exits when the
# containers have started, but the internal services might not have started yet, which means
# that the `authority-config.json` files might not have been created yet.
echo "Waiting for all authority containers to be healthy..."
until check_health; do
  echo "..."
  sleep 2
done

./aggregate-blockchain-config.sh

docker-compose up -d node
