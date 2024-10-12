#!/bin/bash
curl --header "Content-Type: application/json" \
  --request POST \
  --data '{"period": "setup"}' \
  http://localhost:12999/init

echo