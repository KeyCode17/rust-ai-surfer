#!/usr/bin/env bash
set -euo pipefail

MSG_FILE="${1:-.git/COMMIT_EDITMSG}"
[ -f "$MSG_FILE" ] || exit 0

SUBJECT=$(head -n 1 "$MSG_FILE")
BODY=$(cat "$MSG_FILE")

SCOPED_TYPES='feat|fix'
UNSCOPED_TYPES='chore|docs'
OPTIONAL_SCOPE_TYPES='refactor|test|perf|build|ci|style|revert'
VAGUE_SUBJECTS='update|updates|wip|misc|fix stuff|changes|stuff|cleanup|minor'

fail() {
    echo "X $1"
    echo "  got: $SUBJECT"
    exit 1
}

if printf '%s\n' "$SUBJECT" | grep -qE '^(Merge|Revert)'; then
    exit 0
fi

if printf '%s\n' "$BODY" | grep -qiE 'co-authored-by:.*(claude|anthropic|copilot|gpt)|generated with .*claude'; then
    fail "AI attribution trailers are not allowed in commit messages"
fi

if ! printf '%s\n' "$SUBJECT" | grep -qE "^($SCOPED_TYPES|$UNSCOPED_TYPES|$OPTIONAL_SCOPE_TYPES)(\([a-z0-9,._-]+\))?!?: .+"; then
    fail "commit subject must follow Conventional Commits: <type>[(scope)][!]: <subject>"
fi

if printf '%s\n' "$SUBJECT" | grep -qE "^($SCOPED_TYPES)!?: "; then
    fail "'feat' and 'fix' require a module scope, e.g. feat(ras-agent): ..."
fi

if printf '%s\n' "$SUBJECT" | grep -qE "^($UNSCOPED_TYPES)\("; then
    fail "'chore' and 'docs' take no scope"
fi

DESCRIPTION=${SUBJECT#*: }

if [ ${#SUBJECT} -gt 72 ]; then
    fail "subject must be 72 characters or fewer (is ${#SUBJECT})"
fi

if printf '%s\n' "$DESCRIPTION" | grep -qE '^[A-Z]'; then
    fail "subject description must be lowercase"
fi

if printf '%s\n' "$DESCRIPTION" | grep -qE '\.$'; then
    fail "subject description must not end with a period"
fi

if printf '%s\n' "$DESCRIPTION" | grep -qiE "^($VAGUE_SUBJECTS)$"; then
    fail "subject description is too vague, say what changed"
fi
