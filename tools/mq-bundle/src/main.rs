//! `cargo run -p mq-bundle -- --help`: see `macroquad_bundle` in shared-crates.
//! A binary here, not there, because cargo only runs its own workspace's
//! binaries; this keeps the bundler on the same rev as the game.

fn main() -> std::process::ExitCode {
    macroquad_bundle::main()
}
