//! Line of sight (rules.md §5 Step 1).
//!
//! Written from the rules, not the implementation. Solid terrain (buildings /
//! hills) blocks the view when less than 1/3 of the target is visible and gives
//! partial cover between 1/3 and 2/3 hidden; woods block only past 6" of
//! traversal and otherwise add +1; a 'Mech in water has partial cover;
//! base-to-base units always see each other; and intervening units never block.

mod common;

use common::{card, unit};
use game::dice::ScriptedDice;
use game::line_of_sight::line_of_sight;
use game::state::{GameState, Phase};
use game::terrain::{Cover, TerrainFeature, TerrainKind};

fn feature(kind: TerrainKind, x: f64, y: f64, w: f64, h: f64) -> TerrainFeature {
    TerrainFeature::new(kind, "T", x, y, w, h)
}

// --- Geometry: line_of_sight (rules §5 Step 1) ---------------------------

#[test]
fn open_ground_has_a_clear_line_of_sight() {
    let s = line_of_sight((5.0, 20.0), (35.0, 20.0), &[]);
    assert!(!s.blocked);
    assert_eq!(s.cover, Cover::None);
}

#[test]
fn solid_terrain_hiding_most_of_the_target_blocks_los() {
    // A building spanning the full height between attacker and target hides the
    // whole silhouette — less than 1/3 visible, so LOS is blocked.
    let building = feature(TerrainKind::Building, 18.0, 0.0, 4.0, 40.0);
    let s = line_of_sight((5.0, 20.0), (35.0, 20.0), &[building]);
    assert!(s.blocked);
}

#[test]
fn solid_terrain_hiding_half_the_target_gives_partial_cover() {
    // Building top edge at the sightline height hides the lower half of the
    // target: between 1/3 and 2/3 hidden → partial cover, not blocked.
    let building = feature(TerrainKind::Building, 18.0, 0.0, 4.0, 20.0);
    let s = line_of_sight((5.0, 20.0), (35.0, 20.0), &[building]);
    assert!(!s.blocked);
    assert_eq!(s.cover, Cover::Partial);
    assert_eq!(s.to_hit_modifier(), 1);
}

#[test]
fn solid_terrain_clear_of_the_sightline_neither_blocks_nor_covers() {
    // Building sits well below the line of sight; up to 1/3 hidden → clear.
    let building = feature(TerrainKind::Building, 18.0, 0.0, 4.0, 15.0);
    let s = line_of_sight((5.0, 20.0), (35.0, 20.0), &[building]);
    assert!(!s.blocked);
    assert_eq!(s.cover, Cover::None);
}

#[test]
fn intervening_woods_under_six_inches_add_one_but_do_not_block() {
    let woods = feature(TerrainKind::Woods, 18.0, 0.0, 4.0, 40.0); // 4" of traversal
    let s = line_of_sight((5.0, 20.0), (35.0, 20.0), &[woods]);
    assert!(!s.blocked);
    assert_eq!(s.cover, Cover::Woods);
    assert_eq!(s.to_hit_modifier(), 1);
}

#[test]
fn six_inches_or_more_of_woods_block_los() {
    let woods = feature(TerrainKind::Woods, 16.0, 0.0, 8.0, 40.0); // 8" of traversal
    let s = line_of_sight((5.0, 20.0), (35.0, 20.0), &[woods]);
    assert!(s.blocked);
}

#[test]
fn a_target_standing_in_woods_is_concealed() {
    // Occupied woods: the line ends inside a woods patch shorter than 6".
    let woods = feature(TerrainKind::Woods, 30.0, 10.0, 10.0, 20.0);
    let s = line_of_sight((5.0, 20.0), (35.0, 20.0), &[woods]);
    assert!(!s.blocked);
    assert_eq!(s.cover, Cover::Woods);
}

#[test]
fn a_mech_standing_in_water_has_partial_cover() {
    let water = feature(TerrainKind::Water, 30.0, 10.0, 10.0, 20.0);
    let s = line_of_sight((5.0, 20.0), (35.0, 20.0), &[water]);
    assert!(!s.blocked);
    assert_eq!(s.cover, Cover::Partial);
}

#[test]
fn base_to_base_units_always_have_line_of_sight() {
    // Even with a wall over both units, adjacent 'Mechs can see each other.
    let wall = feature(TerrainKind::Building, 0.0, 0.0, 40.0, 40.0);
    let s = line_of_sight((20.0, 20.0), (20.5, 20.0), &[wall]);
    assert!(!s.blocked);
}

// --- Attack integration (rules §5) ---------------------------------------

/// Put two enemies at fixed positions with the given terrain, jump straight to
/// the Attack phase, and hand player 0 the initiative to fire first.
fn attack_setup(
    attacker_pos: (f64, f64),
    target_pos: (f64, f64),
    terrain: Vec<TerrainFeature>,
    dice: ScriptedDice,
) -> GameState<ScriptedDice> {
    let a = unit(0, 0, card("A", 10.0, 2, (3, 2, 1), 4, 3, 4, 0), attacker_pos);
    let t = unit(1, 1, card("T", 10.0, 2, (3, 2, 1), 4, 3, 4, 0), target_pos);
    let mut g = GameState::new(vec![a, t], terrain, dice);
    g.phase = Phase::Attack;
    g
}

#[test]
fn an_attack_without_line_of_sight_is_refused() {
    // A building between the two units blocks the shot: no roll, no damage.
    let building = feature(TerrainKind::Building, 12.0, 0.0, 4.0, 40.0);
    // Empty dice: a blocked attack must never consult the dice.
    let mut g = attack_setup((5.0, 20.0), (25.0, 20.0), vec![building], ScriptedDice::new([]));
    let r = g.attack(0, 1, false).expect("attack resolves to a result");
    assert!(!r.has_los);
    assert!(!r.hit);
    assert_eq!(r.damage, 0);
    assert_eq!(g.unit(1).unwrap().cur_armor, 4); // untouched
}

#[test]
fn partial_cover_raises_the_to_hit_number() {
    // Building hiding half the target adds +1. TN = skill 4 + medium range 2
    // + attacker standstill -1 + partial cover 1 = 6. (Range: 20" is Medium.)
    let building = feature(TerrainKind::Building, 12.0, 0.0, 4.0, 20.0);
    let mut g = attack_setup((5.0, 20.0), (25.0, 20.0), vec![building], ScriptedDice::new([6]));
    let r = g.attack(0, 1, false).expect("attack resolves");
    assert!(r.has_los);
    assert_eq!(r.target_number, 6);
}

#[test]
fn an_intervening_unit_never_blocks_the_shot() {
    // A friendly unit standing directly between attacker and target is not
    // terrain and does not block LOS (rules §5 Step 1).
    let a = unit(0, 0, card("A", 10.0, 2, (3, 2, 1), 4, 3, 4, 0), (5.0, 20.0));
    let blocker = unit(2, 0, card("B", 10.0, 2, (3, 2, 1), 4, 3, 4, 0), (15.0, 20.0));
    let t = unit(1, 1, card("T", 10.0, 2, (3, 2, 1), 4, 3, 4, 0), (25.0, 20.0));
    let mut g = GameState::new(vec![a, blocker, t], Vec::new(), ScriptedDice::new([8]));
    g.phase = Phase::Attack;
    let r = g.attack(0, 1, false).expect("attack resolves");
    assert!(r.has_los);
    assert!(r.hit); // TN = 4 + 2 (medium) - 1 (standstill) = 5; rolled 8
    assert_eq!(g.unit(1).unwrap().cur_armor, 2); // medium damage 2 landed
}
