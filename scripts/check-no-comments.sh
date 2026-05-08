#!/usr/bin/env bash
set -euo pipefail
fail=0
for f in "$@"; do
    case "$f" in
        *.rs) ;;
        *) continue ;;
    esac
    case "$f" in
        */tests/*) continue ;;
        */examples/*) continue ;;
        */xtask/*) continue ;;
    esac
    [ -f "$f" ] || continue
    if grep -nE '^[[:space:]]*(//[^/!]|/\*)' "$f" >/dev/null 2>&1; then
        echo "X $f: comment found (only doc-comments /// or //! allowed)"
        grep -nE '^[[:space:]]*(//[^/!]|/\*)' "$f" || true
        fail=1
    fi
done
exit $fail
