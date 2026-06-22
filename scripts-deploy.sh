#!/usr/bin/env bash
# Helper: build and deploy Guild to the configured cluster.
# Usage: ./scripts-deploy.sh [devnet|mainnet-beta]
set -euo pipefail
CLUSTER="${1:-devnet}"
echo "==> Setting cluster to $CLUSTER"
solana config set --url "$CLUSTER"
echo "==> Building"
anchor build
echo "==> Program id:"
anchor keys list
echo "==> Deploying (ensure the wallet is funded)"
anchor deploy --provider.cluster "$CLUSTER"
echo "Done. Update declare_id! and Anchor.toml if the program id changed."
