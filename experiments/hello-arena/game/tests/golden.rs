//! Golden runs: a seed and a tap script replay to the same score and events.
//! If a rules change moves these numbers, that is a rules change: update the
//! golden values in the same commit and say why.

use hello_arena_game::{Arena, Event, HelloResponse, Input, ORBS, TICK_HZ};

/// Taps the first live orb every half second for ten seconds.
fn run(seed: u64) -> (Arena, Vec<Event>) {
    let mut arena = Arena::new(seed);
    let mut all = Vec::new();
    for t in 0..10 * TICK_HZ {
        let inputs: Vec<Input> = if t % (TICK_HZ / 2) == 0 {
            arena
                .orbs
                .iter()
                .flatten()
                .next()
                .map(|o| Input::Tap { x: o.x, y: o.y })
                .into_iter()
                .collect()
        } else {
            Vec::new()
        };
        all.extend(arena.step(&inputs));
    }
    (arena, all)
}

#[test]
fn same_seed_same_game() {
    assert_eq!(run(42), run(42));
    assert_ne!(run(42).0.score, run(43).0.score);
}

#[test]
fn field_fills_and_taps_collect() {
    let (arena, events) = run(7);
    assert_eq!(arena.tick, u64::from(10 * TICK_HZ));
    assert_eq!(arena.orbs.iter().flatten().count(), ORBS);
    let collected = events
        .iter()
        .filter(|e| matches!(e, Event::Collected { .. }))
        .count();
    assert_eq!(collected, 19, "every scripted tap after the first lands");
    assert!(!events.iter().any(|e| matches!(e, Event::Missed { .. })));
}

#[test]
fn golden_score() {
    // Recorded from the first run of these rules.
    assert_eq!(run(42).0.score, GOLDEN_42);
}

const GOLDEN_42: u64 = 330;

#[test]
fn api_shape_is_stable() {
    let ok: HelloResponse =
        serde_json::from_str(r#"{"hello":{"greeting":"hi","seed":1}}"#).unwrap();
    assert!(matches!(ok, HelloResponse::Hello(h) if h.seed == 1));
    let down: HelloResponse = serde_json::from_str(r#"{"unavailable":"later"}"#).unwrap();
    assert_eq!(down, HelloResponse::Unavailable("later".into()));
}
