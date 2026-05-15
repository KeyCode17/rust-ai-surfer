#!/usr/bin/env bash
set -euo pipefail
fail=0
for f in "$@"; do
    case "$f" in
        *.rs) ;;
        *) continue ;;
    esac
    case "$f" in
        */tests/*|tests/*) continue ;;
        */examples/*|examples/*) continue ;;
        */xtask/*|xtask/*) continue ;;
    esac
    [ -f "$f" ] || continue
    if grep -nE '\.unwrap\(\)' "$f" >/dev/null 2>&1; then
        echo "X $f: unwrap() not allowed"
        grep -nE '\.unwrap\(\)' "$f" || true
        fail=1
    fi
done
exit $fail
