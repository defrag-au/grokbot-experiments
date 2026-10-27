# House rules for agents working in this repo

1. **Typed serde, always.** API bodies, events and config are Rust types with
   `Serialize`/`Deserialize`; no stringly-typed JSON, no `serde_json::Value`
   passed around. Shared types live in the experiment's pure `game` crate.
2. **`cargo clippy --workspace --all-targets -- -D warnings`** and
   `cargo fmt --all -- --check` pass before every push, natively and for
   `--target wasm32-unknown-unknown` on the experiment crates.
3. **Nothing secret or account-specific is committed**: no tokens, no
   account ids, zone ids, binding ids, hostnames, routes or custom domains.
   CI reads `secrets.CLOUDFLARE_API_TOKEN`, `vars.CLOUDFLARE_ACCOUNT_ID` and
   `vars.DISPATCH_NAMESPACE` by name only.
4. **Worker names start with `grokbot-experiments-`.** The Cloudflare
   account is shared with production services; never create, change or
   delete anything outside that prefix (scripts, routes, KV, D1, R2, DNS).
5. **One shared-crates rev.** Every shared-crates dependency is declared once
   in `[workspace.dependencies]` at the same `rev`, so a worker and its client
   can never disagree. Bump them together, in one commit.
6. **No wasm-bindgen in a macroquad client.** miniquad's gl.js cannot host
   it (a blank canvas, no error). `scripts/check-no-wasm-bindgen.sh` must
   pass; reach the browser through miniquad plugins (`miniquad-platform`,
   `quad-net`), never `web-sys`/`js-sys`.
7. **Keep the macroquad `[patch.crates-io]`** copied from the shared-crates
   root. A client never gets its own `[workspace]`, or the patch stops applying.
8. **Rules are pure.** The `game` crate takes time as ticks and randomness as
   a seed, has no I/O, and is covered by golden tests. If a golden value moves,
   say why in the commit.
9. **Browser HTTP answers 200.** quad-net drops non-200 bodies, so API
   failures are `{"unavailable": "<why>"}` with status 200.
