# hello-arena

The template experiment: tap the orbs before they fade; consecutive hits
build a combo.

- `game/` — the rules (`Arena::step` at 60 Hz, typed `Input`/`Event`, the
  `/api/hello` types). `cargo test -p hello-arena-game`.
- `client/` — the macroquad-gamekit screen. Seed from `--seed`, `?seed=`, or
  the worker's `/api/hello`.
- `src/lib.rs` — the worker: `/` → `/app/`, `/api/hello`, otherwise assets.

`npx wrangler dev --env dev` from this directory builds everything and serves
it on http://localhost:8787/.
