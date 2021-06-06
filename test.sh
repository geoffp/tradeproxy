#!/bin/bash

PORT=3137

if [[ "$2" == "--dev" ]]; then
    PORT=3138
fi

if [[ "$1" == "buy" || "$1" == "sell" ]]; then
    curl --header "Content-Type: application/json" \
         --request POST \
         --data "{\"action\": \"$1\", \"contracts\": 1}" \
         http://localhost:$PORT/trade
else
    echo "Need a buy or sell argument!"
fi
