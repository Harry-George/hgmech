//! Shared helpers for the rules-based integration tests.
//!
//! These tests are written from `rules.md` (the Alpha Strike Quick Start rules),
//! NOT from the current implementation, so they double as an executable spec.

#![allow(dead_code)]

use game::dice::ScriptedDice;
use game::state::GameState;
use game::unit::{DamageValues, Size, UnitCard, UnitState};

/// Build a unit card with the fields the rules care about. Special abilities
/// default to off; tweak the returned card's fields for OVL/ENE/etc.
#[allow(clippy::too_many_arguments)]
pub fn card(
    name: &str,
    mv: f64,
    tmm: i32,
    dmg: (u32, u32, u32),
    armor: u32,
    structure: u32,
    pilot: i32,
    overheat: u32,
) -> UnitCard {
    UnitCard {
        name: name.into(),
        size: Size::Medium,
        mv_inches: mv,
        tmm,
        damage: DamageValues {
            short: dmg.0,
            medium: dmg.1,
            long: dmg.2,
        },
        armor,
        structure,
        pilot_skill: pilot,
        overheat,
        ..Default::default()
    }
}

/// Place a unit on the board.
pub fn unit(id: usize, player: usize, c: UnitCard, pos: (f64, f64)) -> UnitState {
    UnitState::new(id, player, c, "FF0000", pos)
}

/// A bare two-unit, two-player game (player 0 = unit 0, player 1 = unit 1) with
/// no terrain, driven by the supplied scripted dice.
pub fn duel(a: UnitState, b: UnitState, dice: ScriptedDice) -> GameState<ScriptedDice> {
    GameState::new(vec![a, b], Vec::new(), dice)
}
