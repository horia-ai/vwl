#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
IAC_DIR="$(dirname "$SCRIPT_DIR")"

echo "=== VWL Infrastructure as Code (IaC) Runner ==="

if [ -z "${GITHUB_TOKEN:-}" ] && [ -n "${ATRIFAN_GITHUB_PAT_TOKEN:-}" ]; then
  export GITHUB_TOKEN="$ATRIFAN_GITHUB_PAT_TOKEN"
fi

if [ -z "${GITHUB_TOKEN:-}" ]; then
  echo "Error: GITHUB_TOKEN or ATRIFAN_GITHUB_PAT_TOKEN environment variable is required."
  exit 1
fi

cd "$IAC_DIR"
echo "1. Initializing Terraform..."
terraform init -upgrade

echo "2. Applying Terraform configuration (Teams, Labels, Permissions)..."
terraform apply -auto-approve

echo "3. Synchronizing Project Board items..."
python3 "$SCRIPT_DIR/sync_project_board.py"

echo "=== IaC Reconciliation Complete ==="
