#!/bin/bash

if [[ "$1" == "buy" || "$1" == "sell" ]]; then
    curl --header "Content-Type: application/json" \
         --request POST \
         --data "{\"action\": \"$1\", \"contracts\": 1}" \
         http://localhost:3137/trade
else
    echo "Need a buy or sell argument!"
fi
