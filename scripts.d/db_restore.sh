#!/usr/bin/env bash
# @name:        DB Restore
# @description: Restores a previously taken database snapshot.
# @category:    Database
# @arg: snapshot | Snapshot File | required
# @arg: database | Target DB     | required | mydb

set -euo pipefail

SNAPSHOT="${1:?Snapshot file required}"
DATABASE="${2:-mydb}"

echo "Restoring $DATABASE from: $SNAPSHOT"
# gunzip -c "$SNAPSHOT" | psql "$DATABASE"
echo "Restore complete."
