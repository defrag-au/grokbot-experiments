#!/usr/bin/env bash
# Build one experiment's worker: the wrangler.toml `[build] command`, run from
# the experiment directory (by `wrangler dev` and `wrangler deploy`).
#
#   1. every macroquad client in macroquad-clients.txt → dist/<served at>/
#      via mq-bundle (builds the wasm, stages the matching JS, renders
#      index.html, checks every import is provided)
#   2. worker-build --release, at the worker-build version matching the
#      resolved `worker` crate (0.7.x → 0.7.x)
#
# A trimmed copy of cnft.dev-workers' scripts/build-worker.sh: no trunk/Leptos
# frontend and no mitos modules here yet.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"

if [ -f macroquad-clients.txt ]; then
    while read -r package dir served title; do
        case "$package" in ''|'#'*) continue ;; esac
        echo "📦 macroquad client ${package} → dist/${served}/"
        (cd "$ROOT" && cargo run -q --release -p mq-bundle -- \
            --package "$package" \
            --wasm "${package}.wasm" \
            --title "$title" \
            --dest "$OLDPWD/dist/${served}" \
            --web "$OLDPWD/${dir}/web" \
            --base "/${served}/")
        # Belt and braces: the miniquad runtime cannot host wasm-bindgen.
        if grep -q __wbindgen "dist/${served}/${package}.wasm"; then
            echo "❌ ${package}.wasm imports wasm-bindgen; macroquad clients must not" >&2
            exit 1
        fi
    done < macroquad-clients.txt
fi

# Something must exist for [assets] even in a worker with no client.
mkdir -p dist

WORKER_VERSION=$(cargo metadata --format-version 1 --manifest-path "$ROOT/Cargo.toml" \
    | jq -r '.packages[] | select(.name == "worker") | .version' | head -1)
if [ -z "$WORKER_VERSION" ]; then
    echo "Error: could not resolve the worker crate version" >&2
    exit 1
fi
case "$(echo "$WORKER_VERSION" | cut -d. -f1,2)" in
    0.6) WORKER_BUILD_VERSION="0.1.11" ;;
    0.7|0.8|0.9|1.0) WORKER_BUILD_VERSION="$WORKER_VERSION" ;;
    *) echo "Error: unknown worker version $WORKER_VERSION; update build-worker.sh" >&2; exit 1 ;;
esac

if ! worker-build --version 2>/dev/null | grep -q "$WORKER_BUILD_VERSION"; then
    cargo install -q worker-build --version "$WORKER_BUILD_VERSION" --locked
fi
echo "🔧 worker ${WORKER_VERSION} → worker-build ${WORKER_BUILD_VERSION}"
exec worker-build --release
