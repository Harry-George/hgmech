//! Sequence of play (rules.md §2 "Sequence of Play").
//!
//! Turn = Initiative → Movement → Attack → End. The initiative WINNER (higher
//! 2D6) acts LAST in Movement and Attack; players alternate activations.

mod common;

use common::{card, duel, unit};
use game::dice::ScriptedDice;
use game::state::Phase;

fn fresh() -> game::state::GameState<ScriptedDice> {
    // Initiative rolls 9 (P0) then 4 (P1); P0 wins and so acts last.
    let a = unit(0, 0, card("A", 10.0, 2, (3, 2, 1), 4, 3, 4, 0), (0.0, 0.0));
    let b = unit(1, 1, card("B", 10.0, 2, (3, 2, 1), 4, 3, 4, 0), (3.0, 0.0));
    duel(a, b, ScriptedDice::new([9, 4]))
}

#[test]
fn turn_begins_in_initiative_phase() {
    let g = fresh();
    assert_eq!(g.phase, Phase::Initiative);
    assert_eq!(g.turn, 1);
}

#[test]
fn phases_cycle_initiative_movement_attack_end() {
    let mut g = fresh();
    g.advance(); // roll initiative -> Movement
    assert_eq!(g.phase, Phase::Movement);
    g.advance(); // first player ends activation
    g.advance(); // second player ends activation -> Attack
    assert_eq!(g.phase, Phase::Attack);
    g.advance();
    g.advance(); // both ended -> End
    assert_eq!(g.phase, Phase::End);
    g.advance(); // resolve End -> next turn Initiative
    assert_eq!(g.phase, Phase::Initiative);
    assert_eq!(g.turn, 2);
}

#[test]
fn initiative_winner_acts_last() {
    // Higher roll wins initiative; the winner moves/fires AFTER the loser.
    let mut g = fresh(); // P0 rolls 9, P1 rolls 4 -> P0 wins
    g.advance(); // -> Movement
    // The loser (P1) activates first.
    assert_eq!(g.current_player(), 1);
    g.advance(); // P1 done
    assert_eq!(g.current_player(), 0); // winner acts last
}

/// Build a game where each player owns `per_player` units, with P0 the
/// initiative loser (so P0 moves first). Initiative: P0 rolls 4, P1 rolls 9.
fn squads(p0_units: usize, p1_units: usize) -> game::state::GameState<ScriptedDice> {
    let mut units = Vec::new();
    let mut id = 0;
    for _ in 0..p0_units {
        units.push(unit(id, 0, card("A", 10.0, 2, (3, 2, 1), 4, 3, 4, 0), (0.0, id as f64)));
        id += 1;
    }
    for _ in 0..p1_units {
        units.push(unit(id, 1, card("B", 10.0, 2, (3, 2, 1), 4, 3, 4, 0), (30.0, id as f64)));
        id += 1;
    }
    game::state::GameState::new(units, Vec::new(), ScriptedDice::new([4, 9]))
}

#[test]
fn movement_alternates_one_unit_at_a_time() {
    // rules §2 Step 2: in Movement, players alternate moving ONE unit at a time,
    // loser first — NOT "one player moves everything, then the other".
    let mut g = squads(2, 2);
    g.advance(); // -> Movement

    let mut sequence = Vec::new();
    for _ in 0..4 {
        sequence.push(g.current_player());
        g.advance(); // end one unit's activation
    }
    // Loser (P0) first, strictly alternating per unit.
    assert_eq!(sequence, vec![0, 1, 0, 1]);
    // All four units have now been activated -> Attack phase.
    assert_eq!(g.phase, Phase::Attack);
}

#[test]
fn the_larger_force_moves_proportionally_more_units() {
    // rules §2 Step 2: "the side with more units moves proportionally more units
    // per alternation." P0 (loser) has 3 units, P1 has 1.
    let mut g = squads(3, 1);
    g.advance(); // -> Movement

    let mut sequence = Vec::new();
    for _ in 0..4 {
        sequence.push(g.current_player());
        g.advance();
    }
    // P1's lone unit is interleaved rather than left to the end, and P0 takes
    // three activations to P1's one.
    assert_eq!(sequence, vec![0, 1, 0, 0]);
    assert_eq!(sequence.iter().filter(|&&p| p == 0).count(), 3);
    assert_eq!(sequence.iter().filter(|&&p| p == 1).count(), 1);
    assert_eq!(g.phase, Phase::Attack);
}

#[test]
fn a_unit_may_attack_only_once_per_turn() {
    // rules §5 / §2 Step 3: "each unit may execute one attack" per turn.
    let attacker = unit(0, 0, card("A", 10.0, 2, (3, 2, 1), 4, 3, 4, 0), (0.0, 0.0));
    let target = unit(1, 1, card("T", 10.0, 2, (3, 2, 1), 20, 5, 4, 0), (3.0, 0.0));
    let mut g = duel(attacker, target, ScriptedDice::new([11, 11]));
    g.phase = Phase::Attack; // P0 is the current attacker by default

    let first = g.attack(0, 1, false).expect("first attack resolves");
    assert!(first.hit);
    assert_eq!(g.unit(1).unwrap().cur_armor, 17); // 20 - 3

    // The same unit cannot fire again this turn.
    assert!(g.attack(0, 1, false).is_none());
    assert!(!g.is_actionable(0), "a unit that fired is no longer actionable");
    assert_eq!(g.unit(1).unwrap().cur_armor, 17); // unchanged — no second shot
}

#[test]
fn undoing_a_move_frees_a_different_unit_to_activate() {
    // Picking a unit to move locks the activation to it, but fully undoing that
    // unit's move must release the lock so the player can move a different unit
    // instead (without burning the activation).
    let mut g = squads(2, 1); // P0 (loser) owns units 0 & 1; P1 owns unit 2
    g.advance(); // -> Movement; P0 moves first
    assert_eq!(g.current_player(), 0);

    // Begin moving unit 0 and commit a segment — the activation locks to it.
    g.selected_mover = Some(0);
    g.unit_mut(0).unwrap().set_position((4.0, 0.0));
    g.unit_mut(0).unwrap().finalise_position();
    assert!(g.is_actionable(0));
    assert!(!g.is_actionable(1), "siblings lock out while a unit is mid-move");

    // Undo unit 0's move entirely: the lock releases and either unit can move.
    g.undo_unit_move(0);
    assert_eq!(g.selected_mover, None);
    assert!(g.is_actionable(0));
    assert!(g.is_actionable(1));
    // No activation was consumed — it is still P0's turn to move a unit.
    assert_eq!(g.current_player(), 0);
    assert!(g.moved_units.is_empty());
}

#[test]
fn a_partial_undo_keeps_the_activation_locked() {
    // Undoing only one of several committed segments leaves the unit still
    // partway through its move, so the activation stays locked to it.
    let mut g = squads(2, 1);
    g.advance(); // -> Movement

    g.selected_mover = Some(0);
    g.unit_mut(0).unwrap().set_position((3.0, 0.0));
    g.unit_mut(0).unwrap().finalise_position();
    g.unit_mut(0).unwrap().set_position((6.0, 0.0));
    g.unit_mut(0).unwrap().finalise_position();

    g.undo_unit_move(0); // undo only the second segment
    assert_eq!(g.selected_mover, Some(0));
    assert!(!g.is_actionable(1));
}

#[test]
fn attack_phase_does_not_alternate_per_unit() {
    // rules §2 Step 3: in Combat the acting player resolves ALL their units'
    // attacks before the other player acts (the loser goes first, then the
    // winner) — so the current player only changes once per player, not per unit.
    let mut g = squads(2, 2);
    g.advance(); // -> Movement
    for _ in 0..4 {
        g.advance(); // move all four units -> Attack
    }
    assert_eq!(g.phase, Phase::Attack);
    assert_eq!(g.current_player(), 0); // loser fires first
    g.advance(); // loser is done with ALL attacks
    assert_eq!(g.current_player(), 1); // winner fires all of theirs
    g.advance(); // winner done -> End
    assert_eq!(g.phase, Phase::End);
}

#[test]
fn activations_alternate_between_players() {
    let mut g = fresh();
    g.advance(); // -> Movement, loser first
    let first = g.current_player();
    g.advance();
    let second = g.current_player();
    assert_ne!(first, second, "activation must alternate between the two players");
}

#[test]
fn a_unit_killed_this_turn_can_still_return_fire() {
    // rules §2: weapon attacks are effectively simultaneous — a unit destroyed
    // during the Attack phase is not removed until the End Phase, so it may
    // still make its own one attack this turn rather than being locked out.
    let killer = unit(0, 0, card("Killer", 10.0, 2, (3, 2, 1), 10, 3, 4, 0), (0.0, 0.0));
    // The victim has no armour and only 2 structure, so a 3-damage short-range
    // hit destroys it outright (no crit roll on a dead target).
    let victim = unit(1, 1, card("Victim", 10.0, 2, (3, 2, 1), 0, 2, 4, 0), (3.0, 0.0));
    // A second P1 unit far away so destroying the victim doesn't end the game.
    let bystander = unit(2, 1, card("Bystander", 10.0, 2, (3, 2, 1), 20, 5, 4, 0), (200.0, 0.0));
    // Two hits at short range (TN 3): P0's kill shot, then the victim's reply.
    let mut g = game::state::GameState::new(
        vec![killer, victim, bystander],
        Vec::new(),
        ScriptedDice::new([11, 11]),
    );
    g.phase = Phase::Attack; // jump to combat; P0 (loser slot) fires first

    // P0 destroys the victim outright, but P1 still has the bystander alive.
    assert_eq!(g.current_player(), 0);
    let kill = g.attack(0, 1, false).expect("kill shot resolves");
    assert!(kill.hit);
    assert!(g.unit(1).unwrap().is_destroyed());
    assert!(g.winner.is_none());

    // Hand the turn to P1. The freshly-destroyed victim is NOT yet removed and
    // may return fire — it is not locked out until the End Phase.
    g.advance(); // P0 ends its attacks
    assert_eq!(g.current_player(), 1);
    assert!(
        g.is_actionable(1),
        "a unit killed this turn can still attack until the End Phase"
    );

    let reply = g.attack(1, 0, false).expect("dead unit returns fire");
    assert!(reply.hit);
    // The reply landed: the killer's 10 armour took 3 damage.
    assert_eq!(g.unit(0).unwrap().cur_armor, 7);
}

#[test]
fn victory_when_one_side_has_no_living_units() {
    // Default victory condition: destroy all of the opposing units (rules §2).
    let mut a = unit(0, 0, card("A", 10.0, 2, (3, 2, 1), 4, 3, 4, 0), (0.0, 0.0));
    let mut b = unit(1, 1, card("B", 10.0, 2, (3, 2, 1), 4, 3, 4, 0), (3.0, 0.0));
    // P0 fires a fatal short-range shot at P1.
    a.cur_armor = a.card.armor;
    b.cur_armor = 0;
    b.cur_structure = 1;
    // Attack roll 12 (auto hit), then a crit roll is NOT needed because the unit
    // dies outright; supply a spare just in case the engine consults the dice.
    let mut g = duel(a, b, ScriptedDice::new([12, 5]));
    g.phase = Phase::Attack; // jump straight to combat; P0 is current by default
    assert_eq!(g.current_player(), 0);
    g.attack(0, 1, false);
    assert!(g.unit(1).unwrap().is_destroyed());
    assert_eq!(g.winner, Some(0));
}
