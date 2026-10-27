//! `hello-arena` worker.
//!
//! - `GET /api/hello` → [`HelloResponse`] as JSON, always 200 (a failure is
//!   `{"unavailable": …}`: quad-net drops non-200 bodies in the browser).
//! - `/` → redirect to `/app/`.
//! - everything else → the static bundle in `dist/` (the macroquad client,
//!   staged there by `scripts/build-worker.sh` via mq-bundle).

use hello_arena_game::{Hello, HelloResponse};
use worker::{Context, Date, Env, Request, Response, Result, event};

#[event(fetch)]
async fn fetch(req: Request, env: Env, _ctx: Context) -> Result<Response> {
    let path = req.path();
    match path.as_str() {
        "/" => Response::redirect(req.url()?.join("/app/")?),
        "/api/hello" => Response::from_json(&hello()),
        _ => env.assets("ASSETS")?.fetch_request(req).await,
    }
}

fn hello() -> HelloResponse {
    HelloResponse::Hello(Hello {
        greeting: "hello from the edge".into(),
        seed: Date::now().as_millis(),
    })
}
