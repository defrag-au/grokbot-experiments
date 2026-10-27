//! `hello-arena-client` — the template experiment's screen.
//!
//! The rules are `hello-arena-game`; this crate draws them and feeds taps in.
//!
//! ```sh
//! cargo run -p hello-arena-client                         # native window
//! cargo run -p hello-arena-client -- --seed 42            # a fixed round
//! cargo run -p hello-arena-client -- --shot .tmp/a.png --at 3 --seed 42
//! ```
//!
//! In the browser the seed comes from the worker's `GET /api/hello`, or from
//! `?seed=` on the page URL. Natively it is `--seed` or a fixed default.

use hello_arena_game::{Arena, Event, FIELD_H, HelloResponse, Input, REACH, TICK_HZ};
use macroquad::prelude::*;
use macroquad_gamekit::harness::{self, Args};
use macroquad_gamekit::{Backdrop, Frame, Game, Tick, app};

pub const DESIGN: Vec2 = Vec2::new(390.0, 844.0);
/// The field sits under a HUD band.
const HUD: f32 = 96.0;
const SEED: u64 = 42;

/// Where the round's seed comes from.
enum Source {
    Ready(u64),
    /// Waiting on `/api/hello` (browser only).
    Asking(quad_net::http_request::Request),
}

struct Screen {
    source: Source,
    greeting: String,
    arena: Option<Arena>,
    taps: Vec<Input>,
    /// Recent events for the HUD line, newest last.
    last: Option<Event>,
}

/// `--seed N` natively, `?seed=N` in the browser.
fn seed_flag() -> Option<u64> {
    harness::flag("--seed")
        .or_else(|| miniquad_platform::launch_query("seed"))
        .and_then(|s| s.parse().ok())
}

impl Screen {
    fn new() -> Self {
        let source = match seed_flag() {
            Some(seed) => Source::Ready(seed),
            None if cfg!(target_arch = "wasm32") => {
                Source::Asking(quad_net::http_request::RequestBuilder::new("/api/hello").send())
            }
            None => Source::Ready(SEED),
        };
        Self {
            source,
            greeting: "hello-arena".into(),
            arena: None,
            taps: Vec::new(),
            last: None,
        }
    }

    fn poll(&mut self) {
        let Source::Asking(req) = &mut self.source else {
            return;
        };
        let Some(answer) = req.try_recv() else {
            return;
        };
        let seed = match answer
            .ok()
            .and_then(|body| serde_json::from_str::<HelloResponse>(&body).ok())
        {
            Some(HelloResponse::Hello(hello)) => {
                self.greeting = hello.greeting;
                hello.seed
            }
            Some(HelloResponse::Unavailable(why)) => {
                self.greeting = format!("offline: {why}");
                SEED
            }
            None => {
                self.greeting = "offline".into();
                SEED
            }
        };
        self.source = Source::Ready(seed);
    }
}

impl Game for Screen {
    fn update(&mut self, frame: &Frame) {
        self.poll();
        if let (Source::Ready(seed), None) = (&self.source, &self.arena) {
            self.arena = Some(Arena::new(*seed));
        }
        if let Some(at) = frame.input.gesture.tap {
            self.taps.push(Input::Tap {
                x: at.x,
                y: at.y - HUD,
            });
        }
    }

    fn tick_rate(&self) -> Option<f32> {
        Some(TICK_HZ as f32)
    }

    fn tick(&mut self, _tick: &Tick, _frame: &Frame) {
        let Some(arena) = &mut self.arena else {
            return;
        };
        let taps = std::mem::take(&mut self.taps);
        if let Some(e) = arena
            .step(&taps)
            .into_iter()
            .rfind(|e| !matches!(e, Event::Spawned { .. }))
        {
            self.last = Some(e);
        }
    }

    fn draw(&mut self, _frame: &Frame) {
        let ink = Color::from_hex(0xe8eef5);
        draw_text(&self.greeting, 20.0, 36.0, 24.0, ink);
        let Some(arena) = &self.arena else {
            draw_text("asking the worker for a seed…", 20.0, 72.0, 20.0, GRAY);
            return;
        };
        let status = match self.last {
            Some(Event::Collected { value, combo, .. }) => format!("+{value} ×{combo}"),
            Some(Event::Missed { .. }) => "miss".into(),
            Some(Event::Faded { .. }) => "faded".into(),
            _ => String::new(),
        };
        let line = format!("score {} {status}", arena.score);
        draw_text(&line, 20.0, 72.0, 22.0, Color::from_hex(0x93c47d));
        draw_rectangle_lines(0.0, HUD, DESIGN.x, FIELD_H, 2.0, Color::from_hex(0x2a3b4f));
        for orb in arena.orbs.iter().flatten() {
            let fade = (orb.life as f32 / TICK_HZ as f32).min(1.0);
            let r = REACH * 0.5 + orb.value as f32 * 4.0;
            let colour = Color::from_hex(0x6fa8dc).with_alpha(0.35 + 0.65 * fade);
            draw_circle(orb.x, HUD + orb.y, r, colour);
        }
    }

    fn backdrop(&self) -> Backdrop {
        Backdrop {
            top: Color::from_hex(0x0b1118),
            bottom: Color::from_hex(0x132030),
            accent: Color::from_hex(0x6fa8dc),
        }
    }

    fn capture_note(&self, frame: &Frame) -> String {
        match &self.arena {
            Some(a) => format!("tick {} score {} at {:.2}s", a.tick, a.score, frame.clock),
            None => "waiting for seed".into(),
        }
    }
}

fn window_conf() -> Conf {
    let args = Args::read();
    let (w, h) = args.window_size((390, 844));
    Conf {
        window_title: args.window_title("Hello Arena"),
        window_width: w,
        window_height: h,
        high_dpi: true,
        window_resizable: true,
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    app::run(DESIGN, Screen::new()).await;
}
