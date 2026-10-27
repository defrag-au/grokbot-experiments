#!/usr/bin/env bash
# Every macroquad client (any package listed in an experiment's
# macroquad-clients.txt) must have NO wasm-bindgen in its wasm32 tree:
# miniquad's gl.js cannot host it, and the failure is a blank canvas.
set -euo pipefail
cd "$(dirname "$0")/.."

status=0
for list in experiments/*/macroquad-clients.txt; do
    [ -e "$list" ] || continue
    while read -r package _rest; do
        case "$package" in ''|'#'*) continue ;; esac
        hits=$(cargo tree -p "$package" --target wasm32-unknown-unknown -e normal --prefix none \
            | grep -E '^wasm-bindgen ' || true)
        if [ -n "$hits" ]; then
            echo "❌ ${package} pulls wasm-bindgen on wasm32:" >&2
            cargo tree -p "$package" --target wasm32-unknown-unknown -e normal --invert wasm-bindgen >&2 || true
            status=1
        else
            echo "✓ ${package}: no wasm-bindgen"
        fi
    done < "$list"
done
exit $status
