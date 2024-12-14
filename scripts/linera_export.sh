#!/bin/bash

# Find the first directory in /tmp/ starting with .tmp
TMP_DIR=$(find /tmp/ -maxdepth 1 -type d -name ".tmp*" | head -n 1)

# Check if a directory was found
if [ -z "$TMP_DIR" ]; then
    echo "No .tmp directory found in /tmp/"
    exit 1
fi

# Extract the full path of the directory
TMP_DIR_PATH=$(realpath "$TMP_DIR")

# Set the environment variables
echo "export LINERA_WALLET=${TMP_DIR_PATH}/wallet_0.json"
echo "export LINERA_STORAGE=rocksdb:${TMP_DIR_PATH}/client_0.db"
