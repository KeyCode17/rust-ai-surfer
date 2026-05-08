#!/usr/bin/env bash
set -euo pipefail
ROOT_VER=$(grep -m1 '^version' Cargo.toml | sed -E 's/.*"([^"]+)".*/\1/')
LAST_TAG=$(git describe --tags --abbrev=0 2>/dev/null || echo "v0.0.0")
LAST_VER=${LAST_TAG#v}

if [ "$ROOT_VER" = "$LAST_VER" ]; then
    echo "[bump-gate] no version change, skipping"
    exit 0
fi

echo "[bump-gate] $LAST_VER -> $ROOT_VER"
cargo build --workspace --all-targets
cargo nextest run --workspace --no-fail-fast
cargo run -p ras-cli -- --help >/dev/null
echo "[bump-gate] OK"
