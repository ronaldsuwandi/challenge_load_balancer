#!/bin/bash

# Start the servers in the background
python3 -m http.server 8081 --directory script/server1 &
server1_pid=$!
python3 -m http.server 8082 --directory script/server2 &
server2_pid=$!
python3 -m http.server 8083 --directory script/server3 &
server3_pid=$!

# Function to clean up background processes
cleanup() {
  echo "Shutting down servers..."
  kill $server1_pid $server2_pid $server3_pid
}

# Trap SIGINT (Ctrl+C) and SIGTERM (termination signal) to run the cleanup function
trap cleanup SIGINT SIGTERM

# Wait for the background processes to complete
wait $server1_pid
wait $server2_pid
wait $server3_pid
