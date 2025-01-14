#!/bin/bash

# This used to generate the blockchain config file from the authority config files.

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

candidates='[{"name": "Ricky", "id": 0}, {"name": "Randy", "id": 1}]'

blockchain_config=$(jq -n \
  --argjson candidates "$candidates" \
  --argjson authorities "$authorities" \
  '{candidates: $candidates, authorities: $authorities}')

echo "$blockchain_config" > data/blockchain-config.json
