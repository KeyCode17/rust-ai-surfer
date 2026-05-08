#!/usr/bin/env bash
set -euo pipefail
MSG_FILE="${1:-.git/COMMIT_EDITMSG}"
[ -f "$MSG_FILE" ] || exit 0
SUBJECT=$(head -n 1 "$MSG_FILE")
case "$SUBJECT" in
    feat:*|feat\(*\):*|fix:*|fix\(*\):*|chore:*|chore\(*\):*|docs:*|docs\(*\):*|refactor:*|refactor\(*\):*|test:*|test\(*\):*|perf:*|perf\(*\):*|build:*|build\(*\):*|ci:*|ci\(*\):*|style:*|style\(*\):*|revert:*|revert\(*\):*) ;;
    Merge*|Revert*) ;;
    *)
        echo "X commit subject must follow Conventional Commits"
        echo "  got: $SUBJECT"
        exit 1
        ;;
esac
