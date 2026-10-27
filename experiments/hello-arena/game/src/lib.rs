//! `hello-arena-game` — the rules, and nothing else.
//!
//! Pure: no clock, no randomness source, no I/O, no wasm-bindgen. Time is the
//! [`TICK`] step and randomness is a seeded generator, so a run is a function
//! of `(seed, inputs)` and the golden tests replay it exactly. The worker
//! hands out a [`Hello`] (a seed); the client steps an [`Arena`] with it.

use serde::{Deserialize, Serialize};

/// The rules step at a fixed 60 Hz.
pub const TICK_HZ: u32 = 60;
pub const TICK: f32 = 1.0 / TICK_HZ as f32;

/// The play field, in design units (the client's 390×844 design minus HUD).
pub const FIELD_W: f32 = 390.0;
pub const FIELD_H: f32 = 700.0;
/// Orbs on the field at once.
pub const ORBS: usize = 5;
/// How close a tap must land to collect an orb.
pub const REACH: f32 = 36.0;
/// Ticks an orb lives before it fades and another takes its place.
pub const ORB_LIFE: u32 = 4 * TICK_HZ;

/// What `GET /api/hello` answers. A failure is still a 200, as
/// `{"unavailable": "<why>"}`: quad-net drops a non-200 body in the browser.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HelloResponse {
    Hello(Hello),
    Unavailable(String),
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Hello {
    pub greeting: String,
    /// Seeds the round: the same seed and taps replay the same game.
    pub seed: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Orb {
    pub x: f32,
    pub y: f32,
    /// Ticks left before it fades.
    pub life: u32,
    pub value: u32,
}

/// What the player did this tick.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub enum Input {
    Tap { x: f32, y: f32 },
}

/// What happened, for the client to animate. Typed, never strings.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub enum Event {
    Spawned { slot: usize, orb: Orb },
    Collected { slot: usize, value: u32, combo: u32 },
    Faded { slot: usize },
    Missed { x: f32, y: f32 },
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Arena {
    rng: u64,
    pub tick: u64,
    pub score: u64,
    pub combo: u32,
    pub orbs: [Option<Orb>; ORBS],
}

impl Arena {
    pub fn new(seed: u64) -> Self {
        Self {
            // Never zero: xorshift sticks at zero.
            rng: seed ^ 0x9E37_79B9_7F4A_7C15,
            tick: 0,
            score: 0,
            combo: 0,
            orbs: [None; ORBS],
        }
    }

    fn next(&mut self) -> u64 {
        let mut x = self.rng;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.rng = x;
        x
    }

    fn unit(&mut self) -> f32 {
        (self.next() >> 40) as f32 / (1u64 << 24) as f32
    }

    fn spawn(&mut self) -> Orb {
        let margin = REACH;
        Orb {
            x: margin + self.unit() * (FIELD_W - 2.0 * margin),
            y: margin + self.unit() * (FIELD_H - 2.0 * margin),
            life: ORB_LIFE / 2 + (self.unit() * (ORB_LIFE / 2) as f32) as u32,
            value: 1 + (self.next() % 3) as u32,
        }
    }

    /// One 60 Hz step: inputs first, then ageing, then refilling empty slots.
    pub fn step(&mut self, inputs: &[Input]) -> Vec<Event> {
        let mut events = Vec::new();
        for input in inputs {
            let Input::Tap { x, y } = *input;
            let hit = self
                .orbs
                .iter()
                .enumerate()
                .filter_map(|(i, o)| o.map(|o| (i, (o.x - x).hypot(o.y - y))))
                .filter(|(_, d)| *d <= REACH)
                .min_by(|a, b| a.1.total_cmp(&b.1));
            match hit {
                Some((slot, _)) => {
                    let value = self.orbs[slot].take().map_or(0, |o| o.value);
                    self.combo += 1;
                    self.score += u64::from(value * self.combo);
                    events.push(Event::Collected {
                        slot,
                        value,
                        combo: self.combo,
                    });
                }
                None => {
                    self.combo = 0;
                    events.push(Event::Missed { x, y });
                }
            }
        }
        for slot in 0..ORBS {
            if let Some(orb) = &mut self.orbs[slot] {
                orb.life = orb.life.saturating_sub(1);
                if orb.life == 0 {
                    self.orbs[slot] = None;
                    events.push(Event::Faded { slot });
                }
            }
        }
        // One spawn a tick at most, so they arrive staggered.
        if let Some(slot) = self.orbs.iter().position(Option::is_none) {
            let orb = self.spawn();
            self.orbs[slot] = Some(orb);
            events.push(Event::Spawned { slot, orb });
        }
        self.tick += 1;
        events
    }
}
