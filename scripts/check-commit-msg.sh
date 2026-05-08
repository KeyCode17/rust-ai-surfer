#!/usr/bin/env bash
set -euo pipefail
MSG_FILE="${1:-.git/COMMIT_EDITMSG}"
[ -f "$MSG_FILE" ] || exit 0
SUBJECT=$(head -n 1 "$MSG_FILE")
if printf '%s\n' "$SUBJECT" | grep -qE '^(Merge|Revert)'; then
    exit 0
fi
if printf '%s\n' "$SUBJECT" | grep -qE '^(feat|fix|chore|docs|refactor|test|perf|build|ci|style|revert|release)(\([^)]+\))?!?: .+'; then
    exit 0
fi
echo "X commit subject must follow Conventional Commits"
echo "  got: $SUBJECT"
exit 1
