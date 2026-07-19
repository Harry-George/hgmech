//! To-hit math: range, movement, cover and heat modifiers
//! (rules.md §5 "Combat Phase" and §9 "Quick Reference — Attack Modifiers").
//!
//! Target Number = Skill + Range + Target movement + Attacker movement
//!               + Terrain + Heat level + Fire-Control hits.

mod common;

use common::{card, unit};
use game::combat::{
    attacker_move_modifier, damage_for_range, range_modifier, range_of, resolve_attack,
    target_movement_modifier, terrain_modifier, to_hit_number, Range,
};
use game::dice::ScriptedDice;
use game::terrain::Cover;
use game::unit::MovementMode;

// --- Range brackets (rules §5 Range Table) -------------------------------

#[test]
fn range_brackets_follow_the_range_table() {
    assert_eq!(range_of(0.0), Range::Short);
    assert_eq!(range_of(6.0), Range::Short); // "Up to 6"" is Short
    assert_eq!(range_of(6.0001), Range::Medium); // "Over 6"" is Medium
    assert_eq!(range_of(24.0), Range::Medium);
    assert_eq!(range_of(24.0001), Range::Long); // "Over 24"" is Long
    assert_eq!(range_of(42.0), Range::Long); // "up to 42"" is Long
    assert_eq!(range_of(42.0001), Range::OutOfRange);
}

#[test]
fn range_modifiers_are_zero_two_four() {
    assert_eq!(range_modifier(Range::Short), 0);
    assert_eq!(range_modifier(Range::Medium), 2);
    assert_eq!(range_modifier(Range::Long), 4);
}

#[test]
fn a_target_beyond_long_range_cannot_be_attacked() {
    // rules §5 Range Table: nothing past 42" is a valid weapon-attack range.
    let a = unit(0, 0, card("A", 10.0, 2, (3, 2, 1), 4, 3, 4, 0), (0.0, 0.0));
    let t = unit(1, 1, card("T", 10.0, 2, (3, 2, 1), 4, 3, 4, 0), (0.0, 0.0));
    let mut dice = ScriptedDice::new([]); // must not be consulted — no roll is made
    let r = resolve_attack(&a, &t, 50.0, Cover::None, false, &mut dice);
    assert!(!r.in_range);
    assert!(!r.hit);
    assert_eq!(r.range, Range::OutOfRange);
}

#[test]
fn a_zero_damage_bracket_deals_no_damage() {
    // rules §5: "A 0 or — means no attack possible at that bracket." The engine
    // models this as a 0-damage hit at that bracket.
    let no_long = unit(0, 0, card("A", 10.0, 2, (3, 2, 0), 4, 3, 4, 0), (0.0, 0.0));
    assert_eq!(damage_for_range(&no_long.cur_damage, Range::Long), 0);
}

// --- Attacker movement (rules §9) ----------------------------------------

#[test]
fn attacker_movement_modifiers_match_the_table() {
    assert_eq!(attacker_move_modifier(MovementMode::Stationary), -1); // Standstill
    assert_eq!(attacker_move_modifier(MovementMode::Ground), 0); // Ground Movement
    assert_eq!(attacker_move_modifier(MovementMode::Jump), 2); // Jumping
}

// --- Target movement (rules §9) ------------------------------------------

#[test]
fn stationary_target_gives_no_movement_modifier() {
    let t = unit(0, 0, card("T", 10.0, 3, (3, 2, 1), 4, 3, 4, 0), (0.0, 0.0));
    // mode defaults to Stationary
    assert_eq!(target_movement_modifier(&t), 0);
}

#[test]
fn ground_moving_target_contributes_its_tmm() {
    let mut t = unit(0, 0, card("T", 10.0, 3, (3, 2, 1), 4, 3, 4, 0), (0.0, 0.0));
    t.mode = MovementMode::Ground;
    assert_eq!(target_movement_modifier(&t), 3);
}

#[test]
fn jumping_target_contributes_tmm_plus_one() {
    let mut t = unit(0, 0, card("T", 10.0, 3, (3, 2, 1), 4, 3, 4, 0), (0.0, 0.0));
    t.mode = MovementMode::Jump;
    assert_eq!(target_movement_modifier(&t), 4); // +TMM +1
}

#[test]
fn shutdown_target_has_minus_four_modifier() {
    let mut t = unit(0, 0, card("T", 10.0, 3, (3, 2, 1), 4, 3, 4, 0), (0.0, 0.0));
    t.shutdown = true;
    assert_eq!(target_movement_modifier(&t), -4); // Immobile / Shutdown
}

#[test]
fn heat_two_or_more_drops_target_tmm_by_one() {
    // rules §8: "Subtract 1 from the unit's TMM at Heat Level 2 or higher."
    let mut t = unit(0, 0, card("T", 10.0, 3, (3, 2, 1), 4, 3, 4, 0), (0.0, 0.0));
    t.mode = MovementMode::Ground;
    t.heat = 2;
    assert_eq!(target_movement_modifier(&t), 2); // 3 - 1
}

#[test]
fn heat_does_not_reduce_jumping_tmm() {
    // rules §8: "Jumping Move and jumping TMM are not affected by heat."
    let mut t = unit(0, 0, card("T", 10.0, 3, (3, 2, 1), 4, 3, 4, 0), (0.0, 0.0));
    t.mode = MovementMode::Jump;
    t.heat = 3; // would cost a ground-mover -1 TMM; jumping ignores it
    assert_eq!(target_movement_modifier(&t), 4); // TMM 3 + 1, no heat penalty
}

// --- Terrain / cover (rules §5, §9) --------------------------------------

#[test]
fn cover_modifiers_are_plus_one() {
    // Both partial cover and occupied/intervening woods are +1 in the rules.
    assert_eq!(terrain_modifier(Cover::None), 0);
    assert_eq!(terrain_modifier(Cover::Partial), 1);
    assert_eq!(terrain_modifier(Cover::Woods), 1);
}

// --- Full to-hit summation (rules §5) ------------------------------------

#[test]
fn to_hit_number_sums_every_modifier() {
    let attacker = unit(0, 0, card("A", 10.0, 2, (3, 2, 1), 4, 3, 4, 0), (0.0, 0.0));
    let mut target = unit(1, 1, card("T", 10.0, 2, (3, 2, 1), 4, 3, 4, 0), (0.0, 0.0));
    target.mode = MovementMode::Ground; // TMM 2 applies
    // Skill 4 + target TMM 2 + medium range 2 + attacker stationary -1 + woods 1
    let tn = to_hit_number(&attacker, &target, Range::Medium, Cover::Woods);
    assert_eq!(tn, 8);
}

#[test]
fn attacker_heat_level_raises_its_target_number() {
    // rules §9: an attack "from a unit with Heat Level > 0" adds +Heat level.
    let mut attacker = unit(0, 0, card("A", 10.0, 2, (3, 2, 1), 4, 3, 4, 0), (0.0, 0.0));
    let target = unit(1, 1, card("T", 10.0, 2, (3, 2, 1), 4, 3, 4, 0), (0.0, 0.0));
    attacker.heat = 3;
    // Skill 4 + (stationary attacker -1) + heat 3 = 6, target stationary, short range.
    let tn = to_hit_number(&attacker, &target, Range::Short, Cover::None);
    assert_eq!(tn, 6);
}

#[test]
fn fire_control_hits_add_plus_two_each() {
    // rules §5: "Each Fire Control Hit adds a cumulative attack modifier of +2."
    let mut attacker = unit(0, 0, card("A", 10.0, 2, (3, 2, 1), 4, 3, 4, 0), (0.0, 0.0));
    let target = unit(1, 1, card("T", 10.0, 2, (3, 2, 1), 4, 3, 4, 0), (0.0, 0.0));
    attacker.fire_control_hits = 2;
    // Skill 4 + attacker stationary -1 + 2*2 = 7.
    let tn = to_hit_number(&attacker, &target, Range::Short, Cover::None);
    assert_eq!(tn, 7);
}
