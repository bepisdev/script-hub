#!/usr/bin/env bash
# @name:        Log Analyzer
# @description: Parses and summarises recent application log files.
# @category:    Monitoring

set -euo pipefail

LOG_DIR="${1:-/var/log/app}"
echo "Analysing logs in: $LOG_DIR"
# grep -E "ERROR|WARN" "$LOG_DIR"/*.log | sort | uniq -c | sort -rn | head -20
echo "Analysis complete (stub)."
