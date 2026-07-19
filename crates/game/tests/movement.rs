//! Movement phase (rules.md §4 "Movement Phase").
//!
//! Scope note: this engine models free movement up to the MV budget rather than
//! per-tile terrain movement costs, so the terrain Movement Cost Table and
//! level-change limits are out of scope here. The rules that ARE modelled:
//! the MV budget ceiling, the always-available 2" minimum move, the heat
//! slowdown (−2" of ground move per heat level), and immobility.

mod common;

use common::{card, unit};
use game::unit::MovementMode;

#[test]
fn a_unit_may_not_move_further_than_its_mv() {
    let mut u = unit(0, 0, card("A", 10.0, 2, (3, 2, 1), 4, 3, 4, 0), (0.0, 0.0));
    u.set_position((100.0, 0.0)); // wants 100", MV is 10"
    assert!((u.position.0 - 10.0).abs() < 1e-9);
    assert!(u.available_movement().abs() < 1e-9);
}

#[test]
fn movement_within_budget_is_exact() {
    let mut u = unit(0, 0, card("A", 12.0, 2, (3, 2, 1), 4, 3, 4, 0), (0.0, 0.0));
    u.set_position((3.0, 4.0)); // distance 5 <= 12
    assert_eq!(u.position, (3.0, 4.0));
    assert!((u.available_movement() - 7.0).abs() < 1e-9);
}

#[test]
fn budget_is_shared_across_committed_segments() {
    let mut u = unit(0, 0, card("A", 10.0, 2, (3, 2, 1), 4, 3, 4, 0), (0.0, 0.0));
    u.set_position((6.0, 0.0));
    u.finalise_position(); // used 6 of 10
    u.set_position((100.0, 0.0)); // only 4 left
    assert!((u.position.0 - 10.0).abs() < 1e-9);
}

#[test]
fn undo_restores_the_previous_budget() {
    let mut u = unit(0, 0, card("A", 10.0, 2, (3, 2, 1), 4, 3, 4, 0), (0.0, 0.0));
    u.set_position((6.0, 0.0));
    u.finalise_position();
    u.undo_last_move();
    assert_eq!(u.position, (0.0, 0.0));
    assert!((u.available_movement() - 10.0).abs() < 1e-9);
}

#[test]
fn standing_still_keeps_the_stationary_mode() {
    let u = unit(0, 0, card("A", 10.0, 2, (3, 2, 1), 4, 3, 4, 0), (0.0, 0.0));
    assert_eq!(u.mode, MovementMode::Stationary);
}

#[test]
fn moving_at_least_an_inch_is_ground_movement() {
    let mut u = unit(0, 0, card("A", 10.0, 2, (3, 2, 1), 4, 3, 4, 0), (0.0, 0.0));
    u.set_position((5.0, 0.0));
    u.finalise_position();
    assert_eq!(u.mode, MovementMode::Ground);
}

#[test]
fn heat_subtracts_two_inches_of_ground_move_per_level() {
    // rules §8: "twice its current heat level (in inches) will be subtracted
    // from the unit's ground movement rating."
    let mut u = unit(0, 0, card("A", 12.0, 2, (3, 2, 1), 4, 3, 4, 0), (0.0, 0.0));
    u.heat = 2; // -4"
    assert!((u.movement_budget() - 8.0).abs() < 1e-9);
}

#[test]
fn a_mobile_unit_can_always_move_two_inches() {
    // rules §4 "Minimum Movement": a mobile unit can always move 2", regardless
    // of penalties, so heat can never drop a still-mobile unit below 2".
    let mut u = unit(0, 0, card("A", 6.0, 2, (3, 2, 1), 4, 3, 4, 0), (0.0, 0.0));
    u.heat = 3; // 6 - 6 = 0, but minimum movement floors it at 2"
    assert!((u.movement_budget() - 2.0).abs() < 1e-9);
}

#[test]
fn a_shutdown_unit_is_immobile() {
    let mut u = unit(0, 0, card("A", 10.0, 2, (3, 2, 1), 4, 3, 4, 0), (0.0, 0.0));
    u.shutdown = true;
    assert!((u.movement_budget()).abs() < 1e-9);
}
