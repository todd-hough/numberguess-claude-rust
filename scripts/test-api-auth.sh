#!/usr/bin/env bash
set -euo pipefail

# Test script to verify programmatic API authentication works
# This script:
# 1. Gets an access token from Keycloak using Direct Access Grants
# 2. Calls the API to create a new game with the Bearer token
# 3. Verifies oauth2-proxy validates the token and adds X-Forwarded-* headers

# Colors for output
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo "========================================"
echo "Testing Programmatic API Authentication"
echo "========================================"
echo ""

# Configuration
KEYCLOAK_URL="http://localhost:8090"
OAUTH2_PROXY_URL="http://localhost:8080"
CLIENT_ID="test-client"
CLIENT_SECRET="test-secret-do-not-use-in-production"
USERNAME="admin@local.test"
PASSWORD="password"

# Step 1: Get access token from Keycloak
echo "Step 1: Getting access token from Keycloak..."
TOKEN_RESPONSE=$(curl -s -X POST \
  "${KEYCLOAK_URL}/realms/numberguess/protocol/openid-connect/token" \
  -H "Content-Type: application/x-www-form-urlencoded" \
  -d "grant_type=password" \
  -d "client_id=${CLIENT_ID}" \
  -d "client_secret=${CLIENT_SECRET}" \
  -d "username=${USERNAME}" \
  -d "password=${PASSWORD}")

# Check if we got a token
if echo "$TOKEN_RESPONSE" | grep -q "access_token"; then
  ACCESS_TOKEN=$(echo "$TOKEN_RESPONSE" | jq -r '.access_token')
  echo -e "${GREEN}✓${NC} Successfully obtained access token"
  echo "Token (first 50 chars): ${ACCESS_TOKEN:0:50}..."
else
  echo -e "${RED}✗${NC} Failed to get access token"
  echo "Response: $TOKEN_RESPONSE"
  exit 1
fi

echo ""

# Step 2: Call API to create a game
echo "Step 2: Creating a game via API with Bearer token..."
API_RESPONSE=$(curl -s -w "\nHTTP_STATUS:%{http_code}" \
  -X POST "${OAUTH2_PROXY_URL}/api/games" \
  -H "Authorization: Bearer ${ACCESS_TOKEN}" \
  -H "Content-Type: application/json" \
  -d '{
    "min": 1,
    "max": 100,
    "max_guesses": "10"
  }')

# Extract HTTP status and body
HTTP_BODY=$(echo "$API_RESPONSE" | sed -e 's/HTTP_STATUS\:.*//g')
HTTP_STATUS=$(echo "$API_RESPONSE" | tr -d '\n' | sed -e 's/.*HTTP_STATUS://')

echo "HTTP Status: $HTTP_STATUS"
echo "Response Body: $HTTP_BODY"
echo ""

# Step 3: Verify response
if [ "$HTTP_STATUS" = "200" ]; then
  echo -e "${GREEN}✓${NC} Successfully created game!"

  # Parse game_id from response
  if command -v python3 >/dev/null 2>&1; then
    if ! GAME_ID=$(python3 -c 'import json, sys; data = json.loads(sys.stdin.read()); print(data.get("game_id", ""))' <<<"$HTTP_BODY" 2>/dev/null); then
      GAME_ID=""
    fi
  else
    GAME_ID=$(echo "$HTTP_BODY" | jq -r '.game_id')
  fi

  if [ -z "${GAME_ID:-}" ]; then
    echo -e "${RED}✗${NC} Unable to parse game_id from response"
    echo "Response: $HTTP_BODY"
    exit 1
  fi
  echo "Game ID: $GAME_ID"
  echo ""
  echo -e "${GREEN}========================================"
  echo "✓ Programmatic API authentication works!"
  echo "========================================${NC}"
elif [ "$HTTP_STATUS" = "401" ]; then
  echo -e "${RED}✗${NC} Authentication failed (401 Unauthorized)"
  echo "Response: $HTTP_BODY"
  echo ""
  echo -e "${YELLOW}Troubleshooting:${NC}"
  echo "1. Check if oauth2-proxy is configured with --api-route flag"
  echo "2. Verify test-client has directAccessGrantsEnabled=true in Keycloak"
  echo "3. Check oauth2-proxy logs: docker compose logs oauth2-proxy"
  exit 1
else
  echo -e "${RED}✗${NC} Unexpected HTTP status: $HTTP_STATUS"
  echo "Response: $HTTP_BODY"
  exit 1
fi

# Step 4: Make a guess to test the full flow
echo "Step 4: Making a guess to verify game was created..."
GUESS_RESPONSE=$(curl -s -w "\nHTTP_STATUS:%{http_code}" \
  -X POST "${OAUTH2_PROXY_URL}/api/games/${GAME_ID}/guess" \
  -H "Authorization: Bearer ${ACCESS_TOKEN}" \
  -H "Content-Type: application/json" \
  -d '{"guess": 50}')

GUESS_BODY=$(echo "$GUESS_RESPONSE" | sed -e 's/HTTP_STATUS\:.*//g')
GUESS_STATUS=$(echo "$GUESS_RESPONSE" | tr -d '\n' | sed -e 's/.*HTTP_STATUS://')

echo "HTTP Status: $GUESS_STATUS"
echo "Response: $GUESS_BODY"
echo ""

if [ "$GUESS_STATUS" = "200" ]; then
  RESULT=$(echo "$GUESS_BODY" | jq -r '.result')
  echo -e "${GREEN}✓${NC} Successfully made guess!"
  echo "Result: $RESULT"
else
  echo -e "${RED}✗${NC} Failed to make guess (HTTP $GUESS_STATUS)"
fi
