#!/usr/bin/env bash
set -euo pipefail
MAX=200
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
    lines=$(wc -l < "$f")
    if [ "$lines" -gt "$MAX" ]; then
        echo "X $f: $lines LOC (max $MAX)"
        fail=1
    fi
done
exit $fail
