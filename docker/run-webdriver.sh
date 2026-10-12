#!/usr/bin/env bash
set -euo pipefail

bun install --frozen-lockfile
xvfb-run -a bun --bun run test:e2e
