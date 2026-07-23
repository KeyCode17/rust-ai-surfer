#!/usr/bin/env bash
set -euo pipefail

read_version() {
    printf '%s\n' "$1" |
        awk '/^\[workspace\.package\]/ {inside=1; next} /^\[/ {inside=0} inside && /^version[[:space:]]*=/ {print; exit}' |
        sed -E 's/.*"([^"]+)".*/\1/'
}

BASE=""
if git rev-parse --verify --quiet origin/main >/dev/null; then
    BASE="origin/main"
elif git rev-parse --abbrev-ref --symbolic-full-name '@{upstream}' >/dev/null 2>&1; then
    BASE=$(git rev-parse --abbrev-ref --symbolic-full-name '@{upstream}')
fi

if [ -z "$BASE" ] || [ "$(git rev-parse HEAD)" = "$(git rev-parse "$BASE")" ]; then
    echo "[bump-gate] no release base to compare against, skipping bump check"
else
    COMMITS=$(git log --pretty=format:%s "$BASE..HEAD")
    if [ -z "$COMMITS" ]; then
        echo "[bump-gate] nothing to push"
        exit 0
    fi

    CHANGED=$(git diff --name-only "$BASE...HEAD" | wc -l | tr -d ' ')
    if printf '%s\n' "$COMMITS" | grep -qE '^[a-z]+(\([a-z0-9,._-]+\))?!: '; then
        REQUIRED="major"
    elif printf '%s\n' "$COMMITS" | grep -qE '^feat(\([a-z0-9,._-]+\))?: '; then
        REQUIRED="minor"
    elif [ "$CHANGED" -lt 5 ]; then
        REQUIRED="patch"
    else
        REQUIRED="minor"
    fi

    CURRENT=$(read_version "$(cat Cargo.toml)")
    PREVIOUS=$(read_version "$(git show "$BASE:Cargo.toml")")

    if [ "$CURRENT" = "$PREVIOUS" ]; then
        echo "X [bump-gate] every push carries a version bump: $PREVIOUS is unchanged (need $REQUIRED)"
        exit 1
    fi

    IFS=. read -r CUR_MAJOR CUR_MINOR CUR_PATCH <<<"$CURRENT"
    IFS=. read -r PRE_MAJOR PRE_MINOR PRE_PATCH <<<"$PREVIOUS"

    if [ "$CUR_MAJOR" -gt "$PRE_MAJOR" ]; then
        ACTUAL="major"
    elif [ "$CUR_MINOR" -gt "$PRE_MINOR" ]; then
        ACTUAL="minor"
    elif [ "$CUR_PATCH" -gt "$PRE_PATCH" ]; then
        ACTUAL="patch"
    else
        echo "X [bump-gate] version went backwards: $PREVIOUS -> $CURRENT"
        exit 1
    fi

    if [ "$ACTUAL" != "$REQUIRED" ]; then
        echo "X [bump-gate] $CHANGED changed file(s) require a $REQUIRED bump, got $ACTUAL ($PREVIOUS -> $CURRENT)"
        exit 1
    fi

    echo "[bump-gate] $PREVIOUS -> $CURRENT ($REQUIRED)"
fi

cargo build --workspace --all-targets
cargo test --workspace --no-fail-fast
cargo run -p ras-cli -- --help >/dev/null
echo "[bump-gate] OK"
