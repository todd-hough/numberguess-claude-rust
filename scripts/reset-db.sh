#!/usr/bin/env bash
set -euo pipefail

DB_NAME=${TEST_DB_NAME:-numberguess_test}
DB_USER=${POSTGRES_USER:-numberguess}
DB_PASSWORD=${POSTGRES_PASSWORD:-password}
DB_HOST=${POSTGRES_HOST:-localhost}
DB_PORT=${POSTGRES_PORT:-5432}

export PGPASSWORD="${DB_PASSWORD}"

psql -h "${DB_HOST}" -U "${DB_USER}" -p "${DB_PORT}" -d postgres -c "SELECT pg_terminate_backend(pid) FROM pg_stat_activity WHERE datname='${DB_NAME}' AND pid <> pg_backend_pid();" >/dev/null 2>&1 || true
psql -h "${DB_HOST}" -U "${DB_USER}" -p "${DB_PORT}" -d postgres -c "DROP DATABASE IF EXISTS ${DB_NAME};"
psql -h "${DB_HOST}" -U "${DB_USER}" -p "${DB_PORT}" -d postgres -c "CREATE DATABASE ${DB_NAME};"

unset PGPASSWORD

echo "✓ Reset database ${DB_NAME} at ${DB_HOST}:${DB_PORT}" 
