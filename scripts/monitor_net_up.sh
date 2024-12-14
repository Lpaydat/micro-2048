#!/bin/bash

# Program to monitor
program_to_run="linera net up"

# Pattern to match (using grep regex syntax)
pattern="REQUEST_APPLICATION - application_id: ([^,]+), requester_chain_id: ([^,]+), target_chain_id: ([^,]+)"

# Temporary file to track processed lines
processed_lines_file="/tmp/processed_lines.log"

# Ensure the processed lines file exists
touch "$processed_lines_file"

# Function to extract and execute CLI command
execute_command() {
    local line="$1"
    
    # Extract the IDs using regex with bash
    if [[ $line =~ application_id:\ ([^,]+),\ requester_chain_id:\ ([^,]+),\ target_chain_id:\ ([^,]+) ]]; then
        application_id="${BASH_REMATCH[1]}"
        requester_chain_id="${BASH_REMATCH[2]}"
        target_chain_id="${BASH_REMATCH[3]}"

        # Execute the CLI command
        echo "Executing: linera request-application $application_id --requester-chain-id $requester_chain_id --target-chain-id $target_chain_id"
        linera request-application "$application_id" --requester-chain-id "$requester_chain_id" --target-chain-id "$target_chain_id"
    else
        echo "Error: Unable to extract IDs from line: $line" >&2
    fi
}

# Monitor the program output, filter by pattern, and trigger script for unique lines
$program_to_run 2>&1 | grep --line-buffered "$pattern" | while IFS= read -r line; do
    # Check if the line has already been processed
    if ! grep -Fxq "$line" "$processed_lines_file"; then
        echo "$line" >> "$processed_lines_file"  # Mark the line as processed
        execute_command "$line"                   # Trigger the command
    else
        echo "Duplicate line ignored: $line"
    fi

done
