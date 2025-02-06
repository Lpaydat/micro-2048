#!/bin/bash

# Program to monitor
program_to_run="linera service"

# Updated pattern to match HELLO_WORLD with numeric value
pattern="HELLO_WORLD - value ([0-9]+)"

# Temporary file to track processed lines
processed_lines_file="/tmp/processed_lines.log"

# Ensure the processed lines file exists
touch "$processed_lines_file"

# Function to extract and execute CLI command
execute_command() {
    local line="$1"
    
    # Extract numeric value using regex
    if [[ $line =~ HELLO_WORLD\ -\ value\ ([0-9]+) ]]; then
        local value="${BASH_REMATCH[1]}"
        
        # Create file with the value
        echo "Creating file with value: $value"
        echo "$value" > "hello_world_value.txt"
    else
        echo "Error: Unable to extract numeric value from line: $line" >&2
    fi
}

# Monitor the service output and handle startup
{
    # Start the service in the background
    $program_to_run 2>&1 &
    
    # Capture the PID to monitor
    service_pid=$!
    
    # Wait for service to start up
    sleep 2
    
    # Now run net up command
    linera net up --initial-amount 18446744073709551615
    
} | grep --line-buffered "$pattern" | while IFS= read -r line; do
    # Check if the line has already been processed
    if ! grep -Fxq "$line" "$processed_lines_file"; then
        echo "$line" >> "$processed_lines_file"  # Mark the line as processed
        execute_command "$line"                   # Trigger the command
    else
        echo "Duplicate line ignored: $line"
    fi
done

# Cleanup background process on exit
trap "kill $service_pid 2> /dev/null" EXIT
