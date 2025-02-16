#!/bin/bash

# This used to generate the election config file from the authority config files.

# Traverse the `data/` dir and extract the ports and public keys of all the authorities.
authorities=$(jq -n '[]')
for dir in $(find ./data/ -type d -iname "*authority*"); do
    PUBKEY=$(jq -r ".pk" "$dir/authority-config.json")
    PORT=$(cat "$dir/port")

    new_authority=$(jq -n \
      --arg addr "http://${HOSTNAME}:${PORT}" \
      --arg authority_key "$PUBKEY" \
      '{addr: $addr, authority_key: $authority_key}')
    authorities=$(jq --argjson new_authority "$new_authority" '. += [$new_authority]' <<< "$authorities")
done

# Traverse `data/` dir and extract ports of all the blockchain nodes. 
# Because we aggregate election config before we start the nodes, we need to
# manually calculate the ports, since the data folders for the nodes may be empty
# at this point.
source .env
nodes=$(jq -n '[]')
for i in $(seq 1 $NODE_COUNT); do
    port=$(($NODE_PORT_RANGE_START + i - 1))
    new_node=$(jq -n --arg addr "http://${HOSTNAME}:${port}" '$addr')
    nodes=$(jq --argjson new_node "$new_node" '. += [$new_node]' <<< "$nodes")
done

# These values can be simply hardcoded for now.
name="Trailer Park Supervisor CA, Nova Scotia, Dartmouth, 2025"
beginning="2025-01-01T00:00:00Z"
ending="2025-06-03T12:59:59Z"
candidates='[{"name": "Ricky", "id": 0}, {"name": "Randy", "id": 1}]'

# Gather all the separate json info into one election config json.
election_config=$(jq -n \
  --arg name "$name" \
  --arg beginning "$beginning" \
  --arg ending "$ending" \
  --argjson nodes "$nodes" \
  --argjson candidates "$candidates" \
  --argjson authorities "$authorities" \
  '{name: $name, beginning: $beginning, ending: $ending, nodes: $nodes, candidates: $candidates, authorities: $authorities}')

echo "$election_config" > data/election-config.json
