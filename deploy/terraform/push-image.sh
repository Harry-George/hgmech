#!/usr/bin/env bash
#
# Build the signaling-server image and push it to the ECR repo created by this
# Terraform stack, then roll the ECS service onto the new image.
#
# Usage (from deploy/terraform, after `terraform apply` has created the repo):
#   ./push-image.sh [tag]        # tag defaults to "latest"
#
# Requires: docker, aws cli (authenticated), and a completed `terraform apply`.

set -euo pipefail
cd "$(dirname "$0")"
REPO_ROOT="$(cd ../.. && pwd)"
TAG="${1:-latest}"

REPO_URL="$(terraform output -raw ecr_repository_url)"
REGION="$(terraform output -raw region)"
REGISTRY="${REPO_URL%/*}" # strip the trailing /<repo-name>

echo "Logging in to $REGISTRY …"
aws ecr get-login-password --region "$REGION" \
  | docker login --username AWS --password-stdin "$REGISTRY"

echo "Building $REPO_URL:$TAG …"
docker build --platform linux/amd64 \
  -f "$REPO_ROOT/crates/multiplayer_server/Dockerfile" \
  -t "$REPO_URL:$TAG" \
  "$REPO_ROOT"

echo "Pushing …"
docker push "$REPO_URL:$TAG"

# Roll the service so it picks up the new image (only meaningful once the
# service exists and if you reused an existing tag like "latest").
CLUSTER="$(terraform output -raw ecs_cluster 2>/dev/null || true)"
SERVICE="$(terraform output -raw ecs_service 2>/dev/null || true)"
if [ -n "$CLUSTER" ] && [ -n "$SERVICE" ]; then
  echo "Forcing new deployment of $SERVICE …"
  aws ecs update-service --region "$REGION" \
    --cluster "$CLUSTER" --service "$SERVICE" \
    --force-new-deployment >/dev/null
fi

echo "Done."
