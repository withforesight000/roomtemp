#!/usr/bin/env bash
set -euo pipefail

# export PNPM_NODE_LINKER=hoisted

# if [ -d node_modules ]; then
#   rm -rf node_modules
# fi

# if [ -d src-tauri/target ]; then
#   rm -rf src-tauri/target
# fi

pnpm install --frozen-lockfile

xvfb-run -a pnpm test:e2e
