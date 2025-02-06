#!/bin/bash

if [ $# -eq 0 ]; then
    echo "Usage: $0 <total-number-of-wallets>"
    exit 1
fi

LOG_FILE="faucet_bot.log"
first_hash=""
current_count=0

# Check for existing log file
if [ -f "$LOG_FILE" ]; then
    # Read first wallet address from log
    first_hash=$(grep -m 1 "First wallet address:" "$LOG_FILE" | awk '{print $NF}')
    # Get current iteration count from actual wallet entries
    current_count=$(grep -c '^[0-9]\+ [0-9a-f]\{64\}$' "$LOG_FILE")
fi

total_wallets=$1
start_iter=$((current_count + 1))

if [ $start_iter -gt $total_wallets ]; then
    echo "Already have $current_count wallets. Nothing to do."
    exit 0
fi

for ((i=start_iter; i<=$total_wallets; i++)); do
    echo "Running iteration $i/$total_wallets"
    
    # Execute command and capture output
    output=$(linera wallet init --with-new-chain --faucet https://faucet.testnet-archimedes.linera.net 2>&1)
    
    # Extract hash
    hash=$(echo "$output" | grep -E '^[0-9a-f]{64}$' | head -n 1)
    
    if [ -z "$hash" ]; then
        echo "Error: Failed to capture hash in iteration $i" | tee -a "$LOG_FILE"
        exit 1
    fi
    
    # Store first wallet address if creating initial wallet
    if [ $i -eq 1 ]; then
        first_hash="$hash"
        echo "First wallet address: $first_hash" | tee -a "$LOG_FILE"
    fi
    
    # Perform transfer for subsequent wallets
    if [ $i -gt 1 ]; then
        echo "Transferring 99.99 from $hash to $first_hash" | tee -a "$LOG_FILE"
        
        # Execute transfer command
        transfer_output=$(linera transfer --from "$hash" --to "$first_hash" 99.99 2>&1)
        transfer_exit_code=$?
        
        if [ $transfer_exit_code -ne 0 ]; then
            echo "Error: Transfer failed in iteration $i" | tee -a "$LOG_FILE"
            echo "Transfer output: $transfer_output" | tee -a "$LOG_FILE"
            exit 1
        fi
        
        echo "Transfer completed successfully" | tee -a "$LOG_FILE"
    fi
    
    # Move config directory
    config_dir="$HOME/.config/linera"
    new_dir="$HOME/.config/linera-$i"
    
    if [ -d "$config_dir" ]; then
        mv "$config_dir" "$new_dir"
        echo "Moved config to $new_dir" | tee -a "$LOG_FILE"
    else
        echo "Error: Config directory not found in iteration $i" | tee -a "$LOG_FILE"
        exit 1
    fi
    
    # Log results with index
    echo "$i $hash" >> "$LOG_FILE"
done

echo "Completed $total_wallets wallets with transfers. Results logged to $LOG_FILE"
