//! Damage application and critical hits
//! (rules.md §5 "Determine & Apply Damage" / "Critical Hits", §9 crit table).

mod common;

use common::{card, duel, unit};
use game::combat::{
    apply_critical_hit, apply_damage, critical_hit_for_roll, damage_for_range, CriticalHit, Range,
};
use game::dice::ScriptedDice;
use game::state::Phase;

// --- Damage value per range (rules §5) -----------------------------------

#[test]
fn damage_uses_the_value_for_the_range_bracket() {
    let u = unit(0, 0, card("A", 10.0, 2, (3, 2, 1), 4, 3, 4, 0), (0.0, 0.0));
    assert_eq!(damage_for_range(&u.cur_damage, Range::Short), 3);
    assert_eq!(damage_for_range(&u.cur_damage, Range::Medium), 2);
    assert_eq!(damage_for_range(&u.cur_damage, Range::Long), 1);
}

// --- Armor then structure (rules §5 applying-damage Q&A) -----------------

#[test]
fn damage_depletes_armor_before_structure() {
    let mut t = unit(0, 0, card("T", 10.0, 2, (3, 2, 1), 5, 4, 4, 0), (0.0, 0.0));
    apply_damage(&mut t, 2);
    assert_eq!(t.cur_armor, 3);
    assert_eq!(t.cur_structure, 4);
}

#[test]
fn damage_overflows_from_armor_into_structure() {
    let mut t = unit(0, 0, card("T", 10.0, 2, (3, 2, 1), 5, 4, 4, 0), (0.0, 0.0));
    apply_damage(&mut t, 7); // 5 armor absorbed, 2 into structure
    assert_eq!(t.cur_armor, 0);
    assert_eq!(t.cur_structure, 2);
    assert!(!t.is_destroyed());
}

#[test]
fn a_unit_with_no_structure_left_is_destroyed() {
    let mut t = unit(0, 0, card("T", 10.0, 2, (3, 2, 1), 2, 2, 4, 0), (0.0, 0.0));
    apply_damage(&mut t, 100);
    assert_eq!(t.cur_armor, 0);
    assert_eq!(t.cur_structure, 0);
    assert!(t.is_destroyed());
}

// --- Critical Hits Table (rules §9) --------------------------------------

#[test]
fn critical_hits_table_maps_every_roll() {
    use CriticalHit::*;
    assert_eq!(critical_hit_for_roll(2), Ammo);
    assert_eq!(critical_hit_for_roll(3), Engine);
    assert_eq!(critical_hit_for_roll(4), FireControl);
    assert_eq!(critical_hit_for_roll(5), None);
    assert_eq!(critical_hit_for_roll(6), Weapon);
    assert_eq!(critical_hit_for_roll(7), None);
    assert_eq!(critical_hit_for_roll(8), Mp);
    assert_eq!(critical_hit_for_roll(9), Weapon);
    assert_eq!(critical_hit_for_roll(10), None);
    assert_eq!(critical_hit_for_roll(11), FireControl);
    assert_eq!(critical_hit_for_roll(12), UnitDestroyed);
}

// --- Critical hit effects (rules §5 "Critical Hit Effects") --------------

#[test]
fn weapon_hit_reduces_all_damage_values_by_one() {
    let mut t = unit(0, 0, card("T", 10.0, 2, (3, 2, 0), 4, 3, 4, 0), (0.0, 0.0));
    apply_critical_hit(&mut t, CriticalHit::Weapon);
    assert_eq!(t.cur_damage.short, 2);
    assert_eq!(t.cur_damage.medium, 1);
    assert_eq!(t.cur_damage.long, 0); // floored at 0
}

#[test]
fn fire_control_hit_is_cumulative() {
    let mut t = unit(0, 0, card("T", 10.0, 2, (3, 2, 1), 4, 3, 4, 0), (0.0, 0.0));
    apply_critical_hit(&mut t, CriticalHit::FireControl);
    apply_critical_hit(&mut t, CriticalHit::FireControl);
    assert_eq!(t.fire_control_hits, 2);
}

#[test]
fn second_engine_hit_destroys_the_unit() {
    let mut t = unit(0, 0, card("T", 10.0, 2, (3, 2, 1), 4, 3, 4, 0), (0.0, 0.0));
    apply_critical_hit(&mut t, CriticalHit::Engine);
    assert_eq!(t.engine_hits, 1);
    assert!(!t.is_destroyed());
    apply_critical_hit(&mut t, CriticalHit::Engine);
    assert!(t.is_destroyed());
}

#[test]
fn mp_hit_halves_move_and_tmm() {
    // rules §5: lose half current Move and TMM (min loss 2"/1).
    let mut t = unit(0, 0, card("T", 10.0, 4, (3, 2, 1), 4, 3, 4, 0), (0.0, 0.0));
    apply_critical_hit(&mut t, CriticalHit::Mp);
    assert!((t.cur_mv - 5.0).abs() < 1e-9);
    assert_eq!(t.cur_tmm, 2);
}

#[test]
fn mp_hits_can_immobilise_a_unit() {
    let mut t = unit(0, 0, card("T", 2.0, 1, (3, 2, 1), 4, 3, 4, 0), (0.0, 0.0));
    apply_critical_hit(&mut t, CriticalHit::Mp); // min loss 2" -> 0
    assert!(t.movement_budget().abs() < 1e-9);
}

#[test]
fn ammo_hit_destroys_a_unit_without_protection() {
    let mut t = unit(0, 0, card("T", 10.0, 2, (3, 2, 1), 4, 3, 4, 0), (0.0, 0.0));
    apply_critical_hit(&mut t, CriticalHit::Ammo);
    assert!(t.is_destroyed());
}

#[test]
fn energy_units_ignore_ammo_hits() {
    let mut c = card("T", 10.0, 2, (3, 2, 1), 4, 3, 4, 0);
    c.ene = true; // Energy: no ammo to explode
    let mut t = unit(0, 0, c, (0.0, 0.0));
    apply_critical_hit(&mut t, CriticalHit::Ammo);
    assert!(!t.is_destroyed());
}

#[test]
fn case_survives_an_ammo_hit_but_takes_one_extra_damage() {
    // rules §5/§10 CASE: "Survives Ammo Hit crits, but takes 1 extra damage."
    let mut c = card("T", 10.0, 2, (3, 2, 1), 4, 3, 4, 0);
    c.case = true;
    let mut t = unit(0, 0, c, (0.0, 0.0));
    apply_critical_hit(&mut t, CriticalHit::Ammo);
    assert!(!t.is_destroyed());
    assert_eq!(t.cur_armor, 3); // 1 extra point of damage, absorbed by armor
}

#[test]
fn caseii_ignores_ammo_hits_entirely() {
    // rules §10 CASEII: "Ignores Ammo Hit crits entirely (treat as No Crit)."
    let mut c = card("T", 10.0, 2, (3, 2, 1), 4, 3, 4, 0);
    c.caseii = true;
    let mut t = unit(0, 0, c, (0.0, 0.0));
    apply_critical_hit(&mut t, CriticalHit::Ammo);
    assert!(!t.is_destroyed());
    assert_eq!(t.cur_armor, 4); // no damage at all
}

#[test]
fn a_single_engine_hit_deals_no_damage() {
    // rules §5: an Engine Hit adds heat-on-fire but "no extra damage"; only a
    // SECOND engine hit destroys the unit.
    let mut t = unit(0, 0, card("T", 10.0, 2, (3, 2, 1), 4, 3, 4, 0), (0.0, 0.0));
    apply_critical_hit(&mut t, CriticalHit::Engine);
    assert_eq!(t.engine_hits, 1);
    assert_eq!(t.cur_armor, 4);
    assert_eq!(t.cur_structure, 3);
    assert!(!t.is_destroyed());
}

// --- Crit triggering during a real attack (rules §5 Q&A) -----------------

#[test]
fn a_hit_that_damages_structure_rolls_for_a_crit() {
    let attacker = unit(0, 0, card("A", 10.0, 2, (3, 2, 1), 4, 3, 4, 0), (0.0, 0.0));
    let target = unit(1, 1, card("T", 10.0, 2, (3, 2, 1), 1, 3, 4, 0), (3.0, 0.0));
    // attack roll 11 (hit, not a natural 12), then crit roll 6 = Weapon Hit.
    let mut g = duel(attacker, target, ScriptedDice::new([11, 6]));
    g.phase = Phase::Attack;
    let r = g.attack(0, 1, false).unwrap();
    assert!(r.hit);
    assert_eq!(r.crit, Some(CriticalHit::Weapon));
    // 3 damage: 1 armor + 2 structure -> structure damaged -> crit applied.
    let t = g.unit(1).unwrap();
    assert_eq!(t.cur_structure, 1);
    assert_eq!(t.cur_damage.short, 2); // Weapon Hit reduced its damage
}

#[test]
fn a_natural_twelve_rolls_a_crit_even_on_armor_only() {
    // rules §5 Q1: "Was the attack roll a natural 12? -> roll on the crit table."
    let attacker = unit(0, 0, card("A", 10.0, 2, (3, 2, 1), 4, 3, 4, 0), (0.0, 0.0));
    let target = unit(1, 1, card("T", 10.0, 2, (3, 2, 1), 10, 3, 4, 0), (3.0, 0.0));
    let mut g = duel(attacker, target, ScriptedDice::new([12, 4])); // hit nat-12, crit 4 = FireControl
    g.phase = Phase::Attack;
    let r = g.attack(0, 1, false).unwrap();
    assert_eq!(r.crit, Some(CriticalHit::FireControl));
    let t = g.unit(1).unwrap();
    assert_eq!(t.cur_armor, 7); // only armor was damaged
    assert_eq!(t.fire_control_hits, 1);
}

#[test]
fn an_armor_only_hit_below_twelve_rolls_no_crit() {
    let attacker = unit(0, 0, card("A", 10.0, 2, (3, 2, 1), 4, 3, 4, 0), (0.0, 0.0));
    let target = unit(1, 1, card("T", 10.0, 2, (3, 2, 1), 10, 3, 4, 0), (3.0, 0.0));
    let mut g = duel(attacker, target, ScriptedDice::new([11])); // single die: only the attack roll
    g.phase = Phase::Attack;
    let r = g.attack(0, 1, false).unwrap();
    assert!(r.hit);
    assert_eq!(r.crit, None);
    assert_eq!(g.unit(1).unwrap().fire_control_hits, 0);
}
