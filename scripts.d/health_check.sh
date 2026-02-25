#!/usr/bin/env bash
# @name:        Health Check
# @description: Runs health checks across all registered services.
# @category:    Monitoring

set -euo pipefail

SERVICES=(web-api auth-service cache-service)

for svc in "${SERVICES[@]}"; do
    echo -n "Checking $svc ... "
    # curl -sf "http://$svc/health" && echo "OK" || echo "FAIL"
    echo "OK (stub)"
done
