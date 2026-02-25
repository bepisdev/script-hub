#!/usr/bin/env bash
# @name:        Deploy to Staging
# @description: Builds and deploys the current branch to the staging environment.
# @category:    Development
# @arg: environment | Environment     | required | staging
# @arg: tag         | Image Tag       | optional
# @arg: notify      | Notify Channel  | optional | #deployments

set -euo pipefail

ENV="${1:-staging}"
TAG="${2:-}"
BRANCH=$(git rev-parse --abbrev-ref HEAD 2>/dev/null || echo "unknown")

echo "Deploying branch '$BRANCH' to $ENV (tag: ${TAG:-latest})..."
# ./build.sh && rsync -avz dist/ "$ENV:/var/www/app/"
echo "Deploy complete."
