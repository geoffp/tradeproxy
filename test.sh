#!/bin/sh

curl --header "Content-Type: application/json" \
     --request POST \
     --data '{"action": "buy", "contracts": 1}' \
     http://localhost:3137/trade
