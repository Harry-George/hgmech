#!/usr/bin/env bash
#
# Start everything needed to test multiplayer locally:
#   * the matchbox signaling server (crates/multiplayer_server) on :3536
#   * the Trunk dev server serving the WASM app on :8080
#
# Open two browser tabs at http://127.0.0.1:8080 — click "Host" in one (it shows
# a room id), then "Join" in the other and paste the id.
#
# Ctrl-C stops both.

set -euo pipefail
cd "$(dirname "$0")"

# The signaling URLs the client uses live in crates/ui/src/net.rs
# (SIGNAL_HTTP / SIGNAL_WS). They default to 127.0.0.1:3536 to match this.
SERVER_ADDR="${SERVER_ADDR:-127.0.0.1:3536}"

server_pid=""
cleanup() {
  echo ""
  echo "Shutting down…"
  [ -n "$server_pid" ] && kill "$server_pid" 2>/dev/null || true
  # Trunk is in the foreground; killing the process group covers any strays.
  jobs -p | xargs -r kill 2>/dev/null || true
}
trap cleanup EXIT INT TERM

echo "Starting matchbox signaling server on $SERVER_ADDR …"
HOST="$SERVER_ADDR" cargo run -p multiplayer_server &
server_pid=$!

# Give the server a moment to compile/boot before Trunk starts hammering it.
sleep 2
echo ""
echo "Starting Trunk dev server (http://127.0.0.1:8080) …"
echo "Open two tabs: Host in one, Join in the other with the room id."
echo ""

# Foreground; Ctrl-C here triggers the cleanup trap and stops the server too.
trunk serve --address 127.0.0.1 --port 8080
