#!/bin/bash

# Use this file to startup the testing environment.

DATA_DIR="data"

# If docker-compose creates the data dir, it's owner will be root and the services won't
# be able to write anything to it.
if [ ! -d "$DATA_DIR" ]; then
  echo "Error: directory '$DATA_DIR' must exist and be owned by the current user."
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

source .env

# Because `docker compose up` assigns random ports to containers, the election config becomes invalid,
# if we restart the testing environment. For this reason we're running the containers manually with specified ports.
# Run `authority` containers.
for i in $(seq 1 $AUTHORITY_COUNT); do
    port=$(($AUTHORITY_PORT_RANGE_START + i - 1))
    docker-compose run -dp $port:$APP_PORT --name testing_authority_"$i" authority
done

# We need to wait for containers to be healthy, because `docker-compose up` exits when the
# containers have started, but the internal services might not have started yet, which means
# that the `authority-config.json` files might not have been created yet.
echo "Waiting for all authority containers to be healthy..."
until check_health; do
  echo "..."
  sleep 3
done

./aggregate-election-config.sh

# Run `node` containers.
for i in $(seq 1 $NODE_COUNT); do
    name="testing_node_$i"
    cp $DATA_DIR/election-config.json $DATA_DIR/$name/
    port=$(($NODE_PORT_RANGE_START + i - 1))
    docker-compose run -dp $port:$APP_PORT --name "$name" node
done
