#!/bin/bash
curl --header "Content-Type: application/json" \
  --request POST \
  http://localhost:12999/reset

echo