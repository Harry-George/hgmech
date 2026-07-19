//! Pure Alpha Strike game logic — the `game` crate.
//!
//! Nothing in this crate depends on Leptos, `web-sys`, or the DOM — it is plain
//! Rust that compiles and unit-tests on the host target with `cargo test -p
//! game`. The behaviour is specified by `rules.md` at the repo root and pinned
//! by the integration tests under `crates/game/tests/`. The sibling `ui` crate
//! drives this engine through a single [`state::GameState`].

pub mod catalog;
// The CSV parser behind the catalog. It runs at *build* time (see `build.rs`,
// which `include!`s this same file), so the crate only needs it compiled in
// order to run its unit tests.
#[cfg(test)]
mod catalog_csv;
pub mod combat;
pub mod dice;
pub mod scenario;
pub mod state;
pub mod terrain;
pub mod unit;
