#!/usr/bin/env bash
# @name:        Log Analyzer
# @description: Parses and summarises recent application log files.
# @category:    Monitoring
# @arg: log_dir   | Log Directory | required | /var/log/app
# @arg: lines     | Top N Lines   | optional | 20
# @arg: level     | Min Log Level | optional | ERROR

set -euo pipefail

LOG_DIR="${1:-/var/log/app}"
LINES="${2:-20}"
LEVEL="${3:-ERROR}"

echo "Analysing $LEVEL+ logs in: $LOG_DIR (top $LINES)"
# grep -E "$LEVEL|WARN" "$LOG_DIR"/*.log | sort | uniq -c | sort -rn | head -"$LINES"
echo "Analysis complete (stub)."
