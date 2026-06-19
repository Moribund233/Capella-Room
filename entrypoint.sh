#!/bin/sh
set -e

# If running as root, fix upload permissions and drop to appuser
if [ "$(id -u)" = "0" ]; then
    chown -R appuser:appuser /app/uploads 2>/dev/null || true
    exec su -s /bin/sh -c 'exec /app/server "$@"' appuser -- "$@"
fi

exec /app/server "$@"
