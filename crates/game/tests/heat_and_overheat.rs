//! Heat and overheating (rules.md §8 "Heat & Overheating").

mod common;

use common::{card, duel, unit};
use game::combat::{resolve_attack, Range};
use game::dice::ScriptedDice;
use game::state::Phase;
use game::terrain::Cover;

// --- Overheat bonus damage (rules §8 "Using Overheat Value") -------------

#[test]
fn overheat_adds_bonus_damage_at_short_range() {
    let a = unit(0, 0, card("A", 10.0, 2, (3, 2, 1), 4, 3, 4, 2), (0.0, 0.0)); // OV 2
    let t = unit(1, 1, card("T", 10.0, 2, (3, 2, 1), 9, 5, 4, 0), (0.0, 0.0));
    let mut dice = ScriptedDice::new([6]);
    let r = resolve_attack(&a, &t, 3.0, Cover::None, true, &mut dice);
    assert!(r.hit);
    assert_eq!(r.range, Range::Short);
    assert_eq!(r.overheat_bonus, 2);
    assert_eq!(r.damage, 5); // short 3 + overheat 2
}

#[test]
fn overheat_gives_no_long_range_bonus_without_ovl() {
    // rules §8: without OVL, overheat only helps at Short and Medium range.
    let a = unit(0, 0, card("A", 10.0, 2, (3, 2, 1), 4, 3, 4, 2), (0.0, 0.0));
    let t = unit(1, 1, card("T", 10.0, 2, (3, 2, 1), 9, 5, 4, 0), (0.0, 0.0));
    let mut dice = ScriptedDice::new([12]);
    let r = resolve_attack(&a, &t, 30.0, Cover::None, true, &mut dice); // long range
    assert!(r.hit);
    assert_eq!(r.range, Range::Long);
    assert_eq!(r.overheat_bonus, 0);
    assert_eq!(r.damage, 1); // long 1, no bonus
}

#[test]
fn ovl_extends_overheat_to_long_range() {
    let mut c = card("A", 10.0, 2, (3, 2, 1), 4, 3, 4, 2);
    c.ovl = true; // Overheat Long
    let a = unit(0, 0, c, (0.0, 0.0));
    let t = unit(1, 1, card("T", 10.0, 2, (3, 2, 1), 9, 5, 4, 0), (0.0, 0.0));
    let mut dice = ScriptedDice::new([12]);
    let r = resolve_attack(&a, &t, 30.0, Cover::None, true, &mut dice);
    assert_eq!(r.overheat_bonus, 2);
    assert_eq!(r.damage, 3); // long 1 + overheat 2
}

#[test]
fn overheat_is_capped_by_the_heat_scale() {
    // rules §8: a unit "cannot overheat more than the heat scale will allow"
    // (max heat 4). At heat 3, an OV-3 unit may only spend 1 more point.
    let mut a = unit(0, 0, card("A", 10.0, 2, (3, 2, 1), 4, 3, 4, 3), (0.0, 0.0));
    a.heat = 3;
    let t = unit(1, 1, card("T", 10.0, 2, (3, 2, 1), 9, 5, 4, 0), (0.0, 0.0));
    let mut dice = ScriptedDice::new([6]); // TN = 4 -1 +3(heat) = 6
    let r = resolve_attack(&a, &t, 3.0, Cover::None, true, &mut dice);
    assert!(r.hit);
    assert_eq!(r.overheat_bonus, 1); // only 1 point left on the scale
    assert_eq!(r.damage, 4); // short 3 + 1
}

#[test]
fn the_stalker_overheat_example_from_the_rules() {
    // rules §11: "Stalker STK-3F: Damage 3/4/2, OV 3, no OVL. With OV 3: up to
    // 6 dmg Short, 7 Medium, but still 2 at Long (no OVL)."
    let stalker = card("Stalker", 8.0, 1, (3, 4, 2), 4, 3, 4, 3); // OV 3, no OVL
    let a = unit(0, 0, stalker, (0.0, 0.0));
    let t = unit(1, 1, card("T", 10.0, 2, (3, 2, 1), 30, 5, 4, 0), (0.0, 0.0));

    // Short range (3"): 3 + 3 overheat = 6.
    let mut dice = ScriptedDice::new([6]);
    let r = resolve_attack(&a, &t, 3.0, Cover::None, true, &mut dice);
    assert_eq!(r.range, Range::Short);
    assert_eq!(r.damage, 6);

    // Medium range (10"): 4 + 3 overheat = 7.
    let mut dice = ScriptedDice::new([8]);
    let r = resolve_attack(&a, &t, 10.0, Cover::None, true, &mut dice);
    assert_eq!(r.range, Range::Medium);
    assert_eq!(r.damage, 7);

    // Long range (30"): 2, no overheat bonus (no OVL).
    let mut dice = ScriptedDice::new([12]);
    let r = resolve_attack(&a, &t, 30.0, Cover::None, true, &mut dice);
    assert_eq!(r.range, Range::Long);
    assert_eq!(r.damage, 2);
}

// --- Heat is paid at the End Phase (rules §8) ----------------------------

#[test]
fn overheating_adds_heat_at_the_end_phase_not_immediately() {
    let a = unit(0, 0, card("A", 10.0, 2, (3, 2, 1), 4, 3, 4, 2), (0.0, 0.0));
    let t = unit(1, 1, card("T", 10.0, 2, (3, 2, 1), 10, 5, 4, 0), (0.0, 0.0));
    let mut g = duel(a, t, ScriptedDice::new([6])); // hits, armor-only -> no crit roll
    g.phase = Phase::Attack;
    g.attack(0, 1, true);
    assert_eq!(g.unit(0).unwrap().heat, 0, "heat is not applied during combat");
    assert_eq!(g.unit(0).unwrap().overheat_used_this_turn, 2);

    g.phase = Phase::End;
    g.advance(); // resolve End Phase
    assert_eq!(g.unit(0).unwrap().heat, 2, "heat applied at the End Phase");
}

// --- Cooling (rules §8 "Cooling Down") -----------------------------------

#[test]
fn a_unit_that_did_not_fire_cools_to_zero() {
    let a = unit(0, 0, card("A", 10.0, 2, (3, 2, 1), 4, 3, 4, 0), (0.0, 0.0));
    let b = unit(1, 1, card("B", 10.0, 2, (3, 2, 1), 4, 3, 4, 0), (3.0, 0.0));
    let mut g = duel(a, b, ScriptedDice::new([]));
    g.units[0].heat = 3;
    g.units[0].attacked_this_turn = false; // did not make a weapon attack
    g.phase = Phase::End;
    g.advance();
    assert_eq!(g.unit(0).unwrap().heat, 0);
}

#[test]
fn a_unit_that_fired_without_overheating_keeps_its_heat() {
    // rules §8: firing without overheat leaves Heat Level unchanged (Alpha
    // Strike has no passive per-turn dissipation while you keep shooting).
    let a = unit(0, 0, card("A", 10.0, 2, (3, 2, 1), 4, 3, 4, 0), (0.0, 0.0));
    let b = unit(1, 1, card("B", 10.0, 2, (3, 2, 1), 4, 3, 4, 0), (3.0, 0.0));
    let mut g = duel(a, b, ScriptedDice::new([]));
    g.units[0].heat = 3;
    g.units[0].attacked_this_turn = true;
    g.units[0].overheat_used_this_turn = 0;
    g.phase = Phase::End;
    g.advance();
    assert_eq!(g.unit(0).unwrap().heat, 3);
}

// --- Shutdown (rules §8 "Shutdown") --------------------------------------

#[test]
fn reaching_heat_four_shuts_a_unit_down() {
    let a = unit(0, 0, card("A", 10.0, 2, (3, 2, 1), 4, 3, 4, 0), (0.0, 0.0));
    let b = unit(1, 1, card("B", 10.0, 2, (3, 2, 1), 4, 3, 4, 0), (3.0, 0.0));
    let mut g = duel(a, b, ScriptedDice::new([]));
    g.units[0].heat = 2;
    g.units[0].attacked_this_turn = true;
    g.units[0].overheat_used_this_turn = 2; // 2 + 2 = 4 = S
    g.phase = Phase::End;
    g.advance();
    assert_eq!(g.unit(0).unwrap().heat, 4);
    assert!(g.unit(0).unwrap().shutdown);
}

#[test]
fn an_engine_hit_adds_one_heat_when_the_unit_fires() {
    // rules §5 Engine Hit: the unit "generates +1 heat whenever it fires". A
    // unit that fires without overheating normally holds its heat, but with an
    // engine hit it gains 1 at the End Phase.
    let a = unit(0, 0, card("A", 10.0, 2, (3, 2, 1), 4, 3, 4, 0), (0.0, 0.0));
    let b = unit(1, 1, card("B", 10.0, 2, (3, 2, 1), 4, 3, 4, 0), (3.0, 0.0));
    let mut g = duel(a, b, ScriptedDice::new([]));
    g.units[0].engine_hits = 1;
    g.units[0].heat = 0;
    g.units[0].attacked_this_turn = true;
    g.units[0].overheat_used_this_turn = 0;
    g.phase = Phase::End;
    g.advance();
    assert_eq!(g.unit(0).unwrap().heat, 1);
}

#[test]
fn a_shutdown_unit_cannot_attack() {
    // rules §8 Shutdown: "the unit cannot move or attack the following turn."
    let a = unit(0, 0, card("A", 10.0, 2, (3, 2, 1), 4, 3, 4, 0), (0.0, 0.0));
    let b = unit(1, 1, card("B", 10.0, 2, (3, 2, 1), 4, 3, 4, 0), (3.0, 0.0));
    let mut g = duel(a, b, ScriptedDice::new([6]));
    g.units[0].shutdown = true;
    g.phase = Phase::Attack;
    assert!(!g.is_actionable(0));
    assert!(g.attack(0, 1, false).is_none());
}

#[test]
fn a_shutdown_unit_restarts_at_the_next_end_phase() {
    // rules §8: "A unit that begins the End Phase as a shutdown unit
    // automatically drops to a Heat Level of 0 (and restarts)."
    let a = unit(0, 0, card("A", 10.0, 2, (3, 2, 1), 4, 3, 4, 0), (0.0, 0.0));
    let b = unit(1, 1, card("B", 10.0, 2, (3, 2, 1), 4, 3, 4, 0), (3.0, 0.0));
    let mut g = duel(a, b, ScriptedDice::new([]));
    g.units[0].heat = 4;
    g.units[0].shutdown = true;
    g.phase = Phase::End;
    g.advance();
    assert_eq!(g.unit(0).unwrap().heat, 0);
    assert!(!g.unit(0).unwrap().shutdown);
}
