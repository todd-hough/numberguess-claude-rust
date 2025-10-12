#!/usr/bin/env bash
set -euo pipefail

if [ "$#" -lt 1 ]; then
  echo "Usage: $0 <url> [timeout_seconds]" >&2
  exit 1
fi

URL="$1"
TIMEOUT="${2:-60}"
INTERVAL=2
ELAPSED=0

while [ "$ELAPSED" -lt "$TIMEOUT" ]; do
  if curl -fsS "$URL" >/dev/null 2>&1; then
    echo "✓ Service at $URL is ready"
    exit 0
  fi
  sleep "$INTERVAL"
  ELAPSED=$((ELAPSED + INTERVAL))
done

echo "Service at $URL did not become ready within ${TIMEOUT}s" >&2
exit 1
