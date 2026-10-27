# grokbot-experiments

Small Rust experiments on Cloudflare Workers, built in the style of
`defrag-au/cnft.dev-workers`: a Rust worker (workers-rs, via shared-crates
`worker_stack`) serving a macroquad-gamekit client as static assets, with the
game's rules in a pure crate both sides share. Grok Bot manages this repo;
each experiment is a demonstrator, not a product.

## Layout

```
Cargo.toml                 one workspace; every shared-crates dep on ONE rev,
                           plus the macroquad [patch.crates-io] from shared-crates
rust-toolchain.toml        stable + wasm32-unknown-unknown + clippy/rustfmt
.cargo/config.toml         wasm32 getrandom backend flag
scripts/build-worker.sh    wrangler's [build] command: mq-bundle each client, then worker-build
scripts/check-no-wasm-bindgen.sh
tools/mq-bundle/           the bundler (shared-crates macroquad-bundle), pinned by this workspace
experiments/
  hello-arena/             the template experiment
    Cargo.toml, src/       the worker (cdylib): /api/* + static assets
    wrangler.toml          no account id, no routes: workers.dev or a dispatch namespace
    macroquad-clients.txt  <package> <crate dir> <served at> <title…>
    game/                  pure rules: serde only, typed events, 60 Hz steps, golden tests
    client/                the macroquad-gamekit screen (no wasm-bindgen, ever)
```

## Add an experiment

1. Copy `experiments/hello-arena` to `experiments/<name>`.
2. Rename the packages (`<name>`, `<name>-game`, `<name>-client`) and add all
   three to `members` in the root `Cargo.toml`.
3. In `wrangler.toml`, set every `name` to `grokbot-experiments-<name>[-dev]`.
   The prefix is mandatory: the deploy workflow refuses anything else.
4. Update `macroquad-clients.txt` and add `<name>` to the `build` matrix in
   `.github/workflows/ci.yml`.

## Run it locally

```sh
cargo test --workspace
cargo run -p hello-arena-client -- --seed 42     # native window, no worker
cd experiments/hello-arena && npx wrangler dev --env dev
# → http://localhost:8787/ redirects to /app/ (the client); /api/hello is the API
```

`wrangler dev` runs `scripts/build-worker.sh`, which needs `jq`, the wasm32
target, and installs the matching `worker-build` if it is missing.

## Deploy model

GitHub Actions (`.github/workflows/deploy.yml`) deploys on every push to
`main`, or by hand (Actions → Deploy → choose experiment and env). It needs:

| Kind | Name | Notes |
| --- | --- | --- |
| Org secret | `CLOUDFLARE_API_TOKEN` | Workers Scripts edit on the account |
| Org secret | `CLOUDFLARE_ACCOUNT_ID` | the account the workers live in |
| Variable (optional) | `DISPATCH_NAMESPACE` | when set, workers deploy into this Workers for Platforms namespace instead of workers.dev |

Nothing account-specific is committed: no account id, routes, custom domains
or binding ids. Without `DISPATCH_NAMESPACE`, an experiment is served at
`https://grokbot-experiments-<name>.<account subdomain>.workers.dev`.
