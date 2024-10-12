#!/bin/bash
curl --header "Content-Type: application/json" \
  --request POST \
  --data '{"path": "./datasets/Dataset-context-NE_20231124_TT05-20240802_1158.zip"}' \
  http://localhost:12999/context-dataset

echo