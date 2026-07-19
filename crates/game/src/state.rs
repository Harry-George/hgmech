//! The top-level game state and the turn/phase state machine.
//!
//! `GameState` owns every unit, the terrain, the dice source, and a running
//! combat log. It is pure logic: the UI holds it in a signal and calls these
//! methods, but nothing here knows the UI exists.
//!
//! Turn cycle: **Initiative → Movement → Attack → End → (next turn) Initiative**.
//! Within Movement and Attack, players take turns activating in initiative
//! order (the player with the *lower* initiative roll activates first, so the
//! winner reacts last).

use crate::combat::{self, CriticalHit, AttackResult, HEAT_MAX, HEAT_SHUTDOWN_THRESHOLD};
use crate::dice::Dice;
use crate::line_of_sight::line_of_sight;
use crate::scenario::{BOARD_H, BOARD_W};
use crate::terrain::TerrainFeature;
use crate::unit::UnitState;
use serde::{Deserialize, Serialize};

/// Depth of a home-edge deployment zone, in inches (rules.md §3: on-board start
/// is within 10" of the home edge).
pub const DEPLOY_DEPTH: f64 = 10.0;

#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub enum Phase {
    /// Pre-game: players alternately place their units in their home-edge zone.
    Deployment,
    Initiative,
    Movement,
    Attack,
    End,
}

impl Phase {
    pub fn as_str(&self) -> &'static str {
        match self {
            Phase::Deployment => "Deployment",
            Phase::Initiative => "Initiative",
            Phase::Movement => "Movement",
            Phase::Attack => "Attack",
            Phase::End => "End",
        }
    }
}

/// The whole game. Generic over the dice source `D` so production code uses a
/// real PRNG while tests inject deterministic rolls; keeping `D` concrete (not a
/// boxed trait object) also leaves `GameState` `Send + Sync`, which the default
/// Leptos signal storage requires.
///
/// It derives `Serialize`/`Deserialize` (when the dice source does) so the whole
/// game — units, terrain, log, and PRNG position — can be snapshotted to JSON and
/// shipped to a peer for multiplayer sync.
#[derive(Serialize, Deserialize)]
pub struct GameState<D: Dice> {
    pub turn: usize,
    pub phase: Phase,
    pub num_players: usize,
    pub units: Vec<UnitState>,
    pub terrain: Vec<TerrainFeature>,
    /// Initiative rolls for the current turn as `(player, roll)`.
    pub initiative: Vec<(usize, u8)>,
    /// Players in the order they activate this turn (lowest roll first).
    pub activation_order: Vec<usize>,
    /// Index into `activation_order` of the player activating now (Attack phase,
    /// which is resolved one whole player at a time).
    pub active_idx: usize,
    /// Units that have completed their Movement-phase activation this turn.
    /// Movement alternates one unit at a time (rules.md §2 Step 2), so this
    /// tracks per-unit progress rather than per-player.
    pub moved_units: Vec<usize>,
    /// During Movement, the unit the active player is currently moving (locks
    /// the activation to one unit until it is ended).
    pub selected_mover: Option<usize>,
    /// Units placed during the Deployment phase (per-unit alternation, like
    /// movement). Deployment is complete once every unit is in this list.
    pub deployed_units: Vec<usize>,
    /// During Deployment, the unit the active player is currently placing.
    pub selected_deployer: Option<usize>,
    /// During the Attack phase, the unit the active player has selected to fire.
    pub selected_attacker: Option<usize>,
    /// Newest-last human-readable log of dice rolls and phase changes.
    pub log: Vec<String>,
    /// The winning player once one side is wiped out.
    pub winner: Option<usize>,
    dice: D,
}

impl<D: Dice> GameState<D> {
    /// Build a game from a set of units, terrain, and a dice source.
    pub fn new(units: Vec<UnitState>, terrain: Vec<TerrainFeature>, dice: D) -> Self {
        let num_players = units.iter().map(|u| u.player + 1).max().unwrap_or(1);
        Self {
            turn: 1,
            phase: Phase::Initiative,
            num_players,
            units,
            terrain,
            initiative: Vec::new(),
            activation_order: (0..num_players).collect(),
            active_idx: 0,
            moved_units: Vec::new(),
            selected_mover: None,
            deployed_units: Vec::new(),
            selected_deployer: None,
            selected_attacker: None,
            log: vec!["Turn 1 — roll for initiative.".to_string()],
            winner: None,
            dice,
        }
    }

    /// Build a game that opens in the Deployment phase: players alternately place
    /// their forces before turn 1. Rolls the setup initiative up front (lower
    /// roll places first, per the loser-first convention) and seeds the log.
    pub fn new_deploying(units: Vec<UnitState>, terrain: Vec<TerrainFeature>, dice: D) -> Self {
        let mut game = Self::new(units, terrain, dice);
        game.phase = Phase::Deployment;
        game.roll_and_order();
        game.log = vec![format!(
            "Deployment — P{} places first. Drag units into your home edge.",
            game.activation_order.first().copied().unwrap_or(0)
        )];
        game
    }

    /// The player whose activation it currently is.
    ///
    /// In Movement this is the next player due to move a unit under one-unit-at-
    /// a-time alternation (rules.md §2 Step 2). In every other phase it is the
    /// player at `active_idx` in the initiative order (Attack resolves one whole
    /// player at a time).
    pub fn current_player(&self) -> usize {
        if self.phase == Phase::Movement {
            if let Some(p) = self.next_mover() {
                return p;
            }
        }
        if self.phase == Phase::Deployment {
            if let Some(p) = self.next_deployer() {
                return p;
            }
        }
        self.activation_order
            .get(self.active_idx)
            .copied()
            .unwrap_or(0)
    }

    /// The home-edge deployment zone for `player` as `(x0, x1, y0, y1)` in
    /// inches: player 0 owns the left band, everyone else the right band (a
    /// simplification of "the initiative winner picks an edge" for the 2-player
    /// game). Units may be placed anywhere within 10" of that edge.
    pub fn deployment_zone(&self, player: usize) -> (f64, f64, f64, f64) {
        if player == 0 {
            (0.0, DEPLOY_DEPTH, 0.0, BOARD_H)
        } else {
            (BOARD_W - DEPLOY_DEPTH, BOARD_W, 0.0, BOARD_H)
        }
    }

    /// Place a unit during Deployment, clamped to its owner's zone. Ignored
    /// unless it is that unit's owner's turn to place and the unit is still
    /// un-deployed.
    pub fn set_deploy_position(&mut self, id: usize, target: (f64, f64)) {
        if self.phase != Phase::Deployment {
            return;
        }
        let player = self.current_player();
        let ok = self
            .unit(id)
            .is_some_and(|u| u.player == player && !self.deployed_units.contains(&id));
        if !ok {
            return;
        }
        let (x0, x1, y0, y1) = self.deployment_zone(player);
        let (tx, ty) = target;
        let pos = (tx.clamp(x0, x1), ty.clamp(y0, y1));
        if let Some(u) = self.unit_mut(id) {
            u.position = pos;
            // Keep the movement anchor in step so turn 1's movement measures from
            // the deployed spot.
            u.starting_position = pos;
        }
    }

    /// Units of `player` already placed this Deployment phase.
    fn deployed_count(&self, player: usize) -> usize {
        self.units
            .iter()
            .filter(|u| u.player == player && self.deployed_units.contains(&u.id))
            .count()
    }

    /// The next player to place a unit, or `None` once all are deployed. Uses the
    /// same least-progress, loser-first alternation as [`Self::next_mover`].
    fn next_deployer(&self) -> Option<usize> {
        let mut best: Option<(usize, f64, usize)> = None;
        for (order_idx, &p) in self.activation_order.iter().enumerate() {
            let total = self.units.iter().filter(|u| u.player == p).count();
            if total == 0 {
                continue;
            }
            let placed = self.deployed_count(p);
            if placed >= total {
                continue;
            }
            let progress = placed as f64 / total as f64;
            let better = match best {
                None => true,
                Some((_, bp, boi)) => {
                    progress < bp - 1e-9 || ((progress - bp).abs() <= 1e-9 && order_idx < boi)
                }
            };
            if better {
                best = Some((p, progress, order_idx));
            }
        }
        best.map(|(p, _, _)| p)
    }

    /// Living (non-destroyed) units belonging to `player`.
    fn living_units(&self, player: usize) -> usize {
        self.units
            .iter()
            .filter(|u| u.player == player && !u.is_destroyed())
            .count()
    }

    /// Units of `player` that have already taken their Movement activation.
    fn moved_count(&self, player: usize) -> usize {
        self.units
            .iter()
            .filter(|u| {
                u.player == player && !u.is_destroyed() && self.moved_units.contains(&u.id)
            })
            .count()
    }

    /// The next player to move a unit this Movement phase, or `None` once every
    /// living unit has been activated.
    ///
    /// Each step picks the player who has used the *least* of their activation
    /// allotment (`moved / total`), breaking ties by initiative order (loser
    /// first). With equal counts this is strict alternation; with unequal counts
    /// the larger force moves proportionally more often.
    fn next_mover(&self) -> Option<usize> {
        let mut best: Option<(usize, f64, usize)> = None; // (player, progress, order index)
        for (order_idx, &p) in self.activation_order.iter().enumerate() {
            let total = self.living_units(p);
            if total == 0 {
                continue;
            }
            let moved = self.moved_count(p);
            if moved >= total {
                continue;
            }
            let progress = moved as f64 / total as f64;
            let better = match best {
                None => true,
                Some((_, bp, boi)) => {
                    progress < bp - 1e-9 || ((progress - bp).abs() <= 1e-9 && order_idx < boi)
                }
            };
            if better {
                best = Some((p, progress, order_idx));
            }
        }
        best.map(|(p, _, _)| p)
    }

    /// Label for the HUD's primary action button, given the current phase.
    pub fn primary_action_label(&self) -> &'static str {
        match self.phase {
            Phase::Deployment => "Place Unit",
            Phase::Initiative => "Roll Initiative",
            Phase::Movement => "End Unit Move",
            Phase::Attack => "End Attacks",
            Phase::End => "Resolve End Phase",
        }
    }

    /// Find a unit by id.
    pub fn unit(&self, id: usize) -> Option<&UnitState> {
        self.units.iter().find(|u| u.id == id)
    }

    pub fn unit_mut(&mut self, id: usize) -> Option<&mut UnitState> {
        self.units.iter_mut().find(|u| u.id == id)
    }

    /// Whether `id` may be controlled right now (right owner, right phase,
    /// alive, not shut down, and the game is still going).
    ///
    /// In Movement a unit is only actionable until it has taken its activation,
    /// and once the player starts moving one unit the rest lock out until that
    /// unit's activation ends.
    ///
    /// A unit destroyed *this* turn is a special case: because attacks are
    /// effectively simultaneous (rules.md §2), its wreck is not removed until
    /// the End Phase, so it may still make its own attack in the Attack phase.
    pub fn is_actionable(&self, id: usize) -> bool {
        if self.winner.is_some() {
            return false;
        }
        let player = self.current_player();
        let Some(u) = self.unit(id) else {
            return false;
        };
        if !u.is_owned_by(player) || u.shutdown {
            return false;
        }
        if u.is_destroyed() {
            // Only a unit killed this very turn, and only to fire back, escapes
            // the "wrecks do nothing" rule; anything else is out.
            if !(self.phase == Phase::Attack && u.destroyed_this_turn()) {
                return false;
            }
        }
        if self.phase == Phase::Deployment {
            return !self.deployed_units.contains(&id)
                && self.selected_deployer.is_none_or(|m| m == id);
        }
        if self.phase == Phase::Movement {
            return !self.moved_units.contains(&id)
                && self.selected_mover.is_none_or(|m| m == id);
        }
        if self.phase == Phase::Attack {
            // Each unit may make only one attack per turn (rules §5).
            return !u.attacked_this_turn;
        }
        true
    }

    /// Undo the most recent committed segment of `id`'s movement. If this
    /// returns the unit to its turn-start position with nothing committed, the
    /// Movement activation lock is released so the player may move a different
    /// unit instead (rules.md §2 Step 2: one unit at a time, but the choice of
    /// which unit isn't final until its move is ended).
    pub fn undo_unit_move(&mut self, id: usize) {
        if let Some(u) = self.unit_mut(id) {
            u.undo_last_move();
        }
        let fully_undone = self
            .unit(id)
            .is_some_and(|u| u.lines.is_empty() && u.distance_moved() <= f64::EPSILON);
        if fully_undone && self.selected_mover == Some(id) {
            self.selected_mover = None;
        }
    }

    /// Advance the game by the HUD's primary action.
    pub fn advance(&mut self) {
        if self.winner.is_some() {
            return;
        }
        match self.phase {
            Phase::Deployment => self.end_deployment_activation(),
            Phase::Initiative => self.roll_initiative(),
            Phase::Movement => self.end_movement_activation(),
            Phase::Attack => self.end_activation(),
            Phase::End => self.resolve_end_phase(),
        }
    }

    /// Roll 2D6 per player and set the activation order (lowest roll first).
    /// Shared by the setup deployment roll and each turn's initiative roll.
    fn roll_and_order(&mut self) {
        let mut rolls: Vec<(usize, u8)> = (0..self.num_players)
            .map(|p| (p, self.dice.roll_2d6()))
            .collect();
        self.initiative = rolls.clone();
        // Lowest roll activates first; stable sort keeps player order on ties.
        rolls.sort_by_key(|(_, roll)| *roll);
        self.activation_order = rolls.iter().map(|(p, _)| *p).collect();
    }

    /// End one unit's Deployment placement. Marks the unit being placed (or the
    /// player's next un-deployed unit) as deployed and hands off to the next
    /// player. Once every unit is placed, enters the turn loop at Initiative.
    fn end_deployment_activation(&mut self) {
        let player = self.current_player();
        let unit_id = self
            .selected_deployer
            .filter(|&id| {
                self.unit(id)
                    .is_some_and(|u| u.player == player && !self.deployed_units.contains(&id))
            })
            .or_else(|| {
                self.units
                    .iter()
                    .find(|u| u.player == player && !self.deployed_units.contains(&u.id))
                    .map(|u| u.id)
            });
        if let Some(id) = unit_id {
            self.deployed_units.push(id);
        }
        self.selected_deployer = None;

        let all_deployed = self
            .units
            .iter()
            .all(|u| self.deployed_units.contains(&u.id));
        if all_deployed {
            self.phase = Phase::Initiative;
            self.turn = 1;
            self.active_idx = 0;
            self.initiative.clear();
            self.activation_order = (0..self.num_players).collect();
            self.log
                .push("Deployment complete — roll for initiative.".to_string());
        }
    }

    /// Roll 2D6 per player, set activation order, and enter Movement.
    fn roll_initiative(&mut self) {
        self.roll_and_order();

        let summary: Vec<String> = self
            .initiative
            .iter()
            .map(|(p, r)| format!("P{p}={r}"))
            .collect();
        self.log.push(format!(
            "Initiative: {} → P{} moves first.",
            summary.join(", "),
            self.activation_order.first().copied().unwrap_or(0)
        ));

        self.phase = Phase::Movement;
        self.active_idx = 0;
        self.moved_units.clear();
        self.selected_mover = None;
        self.selected_attacker = None;
    }

    /// End one unit's Movement-phase activation (rules.md §2 Step 2). Marks the
    /// unit being moved (or, if none is selected, the current player's next
    /// un-moved unit — i.e. standing it still) as activated and hands the turn
    /// to the next player. Once every living unit has moved, enters Attack.
    fn end_movement_activation(&mut self) {
        let player = self.current_player();
        let unit_id = self
            .selected_mover
            .filter(|&id| {
                self.unit(id).is_some_and(|u| {
                    u.player == player && !u.is_destroyed() && !self.moved_units.contains(&id)
                })
            })
            .or_else(|| {
                self.units
                    .iter()
                    .find(|u| {
                        u.player == player
                            && !u.is_destroyed()
                            && !self.moved_units.contains(&u.id)
                    })
                    .map(|u| u.id)
            });

        if let Some(id) = unit_id {
            self.moved_units.push(id);
        }
        self.selected_mover = None;

        let all_moved = self
            .units
            .iter()
            .filter(|u| !u.is_destroyed())
            .all(|u| self.moved_units.contains(&u.id));
        if all_moved {
            self.phase = Phase::Attack;
            self.active_idx = 0;
            self.log.push("All units moved — begin attacks.".to_string());
        }
    }

    /// End the current player's Attack activation; when both players have fired
    /// all of their units, advance to the End phase (rules.md §2 Step 3).
    fn end_activation(&mut self) {
        self.selected_attacker = None;
        self.active_idx += 1;
        if self.active_idx >= self.num_players {
            self.active_idx = 0;
            if self.phase == Phase::Attack {
                self.log.push("Attacks resolved — end phase.".to_string());
                self.phase = Phase::End;
            }
        }
    }

    /// Resolve heat (rules.md §8 "Cooling Down" / "Shutdown"), reset per-turn
    /// movement, and start the next turn.
    ///
    /// Alpha Strike heat does not passively dissipate: a unit that keeps firing
    /// holds its heat. Heat only drops when a unit skips its weapon attack (→ 0)
    /// or restarts from shutdown (→ 0). Overheat points spent this turn are
    /// charged here, and a unit reaching the "S" box shuts down.
    fn resolve_end_phase(&mut self) {
        for unit in &mut self.units {
            if unit.shutdown {
                // A unit that begins the End Phase shut down restarts at heat 0.
                unit.heat = 0;
                unit.shutdown = false;
            } else {
                // Charge overheat (and engine-hit heat) accrued this turn.
                let mut generated = unit.overheat_used_this_turn;
                if unit.attacked_this_turn && unit.engine_hits > 0 {
                    generated += 1; // damaged engine adds 1 heat when firing
                }
                unit.heat = (unit.heat + generated).min(HEAT_MAX);

                // A unit that made no weapon attack cools fully.
                if !unit.attacked_this_turn {
                    unit.heat = 0;
                }

                if unit.heat >= HEAT_SHUTDOWN_THRESHOLD {
                    unit.shutdown = true;
                }
            }
            unit.reset_for_new_turn();
        }
        self.turn += 1;
        self.phase = Phase::Initiative;
        self.active_idx = 0;
        self.moved_units.clear();
        self.selected_mover = None;
        self.selected_attacker = None;
        self.initiative.clear();
        self.log
            .push(format!("Turn {} — roll for initiative.", self.turn));
    }

    /// Resolve an attack from `attacker_id` against `target_id`, applying any
    /// damage and logging the result. Returns `None` if the attack is illegal.
    pub fn attack(&mut self, attacker_id: usize, target_id: usize, overheat: bool) -> Option<AttackResult> {
        if self.phase != Phase::Attack || self.winner.is_some() {
            return None;
        }
        if !self.is_actionable(attacker_id) {
            return None;
        }
        let attacker = self.unit(attacker_id)?.clone();
        let target = self.unit(target_id)?.clone();
        // Can't shoot your own units or a wreck.
        if target.player == attacker.player || target.is_destroyed() {
            return None;
        }

        let distance = {
            let (ax, ay) = attacker.position;
            let (tx, ty) = target.position;
            ((tx - ax).powi(2) + (ty - ay).powi(2)).sqrt()
        };

        // Step 1 — Line of Sight (rules.md §5). Terrain between the two units may
        // block the shot outright, or grant the target cover (+1 to hit).
        let sighting = line_of_sight(attacker.position, target.position, &self.terrain);
        if sighting.blocked {
            let range = combat::range_of(distance);
            self.log.push(format!(
                "{} → {}: no line of sight.",
                attacker.card.name, target.card.name
            ));
            return Some(AttackResult {
                roll: 0,
                target_number: 0,
                hit: false,
                range,
                damage: 0,
                in_range: range != combat::Range::OutOfRange,
                has_los: false,
                overheat_bonus: 0,
                overheat_heat: 0,
                crit: None,
            });
        }
        let cover = sighting.cover;

        let mut result = combat::resolve_attack(
            &attacker,
            &target,
            distance,
            cover,
            overheat && attacker.card.overheat > 0,
            &mut self.dice,
        );

        if !result.in_range {
            self.log.push(format!(
                "{} → {}: out of range ({:.0}\").",
                attacker.card.name, target.card.name, distance
            ));
            return Some(result);
        }

        // Record this attack for End-Phase heat resolution. The heat cost of any
        // overheat is charged then, not now (rules.md §8: heat changes at the
        // End Phase of the turn the unit overheated).
        if let Some(a) = self.unit_mut(attacker_id) {
            a.attacked_this_turn = true;
            a.overheat_used_this_turn = result.overheat_heat;
        }

        if result.hit {
            let structure_before = target.cur_structure;
            if let Some(t) = self.unit_mut(target_id) {
                combat::apply_damage(t, result.damage);
            }
            let (destroyed, structure_now) = self
                .unit(target_id)
                .map(|t| (t.is_destroyed(), t.cur_structure))
                .unwrap_or((true, 0));

            // A surviving target rolls for a critical hit if the attack damaged
            // its structure, or if the attacker rolled a natural 12 (rules §5).
            if !destroyed && (result.roll == 12 || structure_now < structure_before) {
                let crit_roll = self.dice.roll_2d6();
                let crit = combat::critical_hit_for_roll(crit_roll);
                result.crit = Some(crit);
                if !matches!(crit, CriticalHit::None) {
                    if let Some(t) = self.unit_mut(target_id) {
                        combat::apply_critical_hit(t, crit);
                    }
                }
            }

            let destroyed = self.unit(target_id).map(UnitState::is_destroyed).unwrap_or(false);
            let crit_note = match result.crit {
                Some(c) if !matches!(c, CriticalHit::None) => format!(" [{}]", c.as_str()),
                _ => String::new(),
            };
            self.log.push(format!(
                "{} → {}: rolled {} vs {} — HIT for {} ({} range){}{}.",
                attacker.card.name,
                target.card.name,
                result.roll,
                result.target_number,
                result.damage,
                result.range.as_str(),
                crit_note,
                if destroyed { " — DESTROYED" } else { "" }
            ));
            self.check_for_winner();
        } else {
            self.log.push(format!(
                "{} → {}: rolled {} vs {} — miss ({} range).",
                attacker.card.name,
                target.card.name,
                result.roll,
                result.target_number,
                result.range.as_str()
            ));
        }

        Some(result)
    }

    /// Set the winner if exactly one player still has living units.
    fn check_for_winner(&mut self) {
        let mut alive_players: Vec<usize> = self
            .units
            .iter()
            .filter(|u| !u.is_destroyed())
            .map(|u| u.player)
            .collect();
        alive_players.sort_unstable();
        alive_players.dedup();
        if alive_players.len() == 1 {
            let winner = alive_players[0];
            self.winner = Some(winner);
            self.log.push(format!("Player {winner} wins!"));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dice::ScriptedDice;
    use crate::terrain::{TerrainFeature, TerrainKind};
    use crate::unit::{DamageValues, Size, UnitCard, UnitState};

    fn card(name: &str) -> UnitCard {
        UnitCard {
            name: name.into(),
            size: Size::Medium,
            mv_inches: 10.0,
            tmm: 2,
            damage: DamageValues {
                short: 3,
                medium: 2,
                long: 1,
            },
            armor: 4,
            structure: 2,
            pilot_skill: 4,
            overheat: 2,
            ..Default::default()
        }
    }

    /// Two units, one per player, at a short-range distance apart.
    fn two_player_game(dice: ScriptedDice) -> GameState<ScriptedDice> {
        let units = vec![
            UnitState::new(0, 0, card("Atlas"), "FF0000", (0.0, 0.0)),
            UnitState::new(1, 1, card("Wolf"), "0000FF", (3.0, 0.0)),
        ];
        GameState::new(units, Vec::new(), dice)
    }

    #[test]
    fn phase_cycle_runs_in_order() {
        // 2 initiative rolls, then end-activation steps.
        let mut g = two_player_game(ScriptedDice::new([5, 8]));
        assert_eq!(g.phase, Phase::Initiative);
        g.advance(); // roll initiative
        assert_eq!(g.phase, Phase::Movement);
        g.advance(); // player A ends movement
        assert_eq!(g.phase, Phase::Movement);
        g.advance(); // player B ends movement -> Attack
        assert_eq!(g.phase, Phase::Attack);
        g.advance();
        g.advance(); // both end attacks -> End
        assert_eq!(g.phase, Phase::End);
        g.advance(); // resolve end -> next turn Initiative
        assert_eq!(g.phase, Phase::Initiative);
        assert_eq!(g.turn, 2);
    }

    #[test]
    fn initiative_lowest_roll_activates_first() {
        let mut g = two_player_game(ScriptedDice::new([9, 4]));
        g.advance();
        // Player 1 rolled 4 (lower) so activates first.
        assert_eq!(g.activation_order, vec![1, 0]);
        assert_eq!(g.current_player(), 1);
    }

    #[test]
    fn activation_alternates_between_players() {
        let mut g = two_player_game(ScriptedDice::new([3, 7]));
        g.advance(); // initiative -> Movement, order [0, 1]
        assert_eq!(g.current_player(), 0);
        g.advance(); // end player 0
        assert_eq!(g.current_player(), 1);
    }

    #[test]
    fn attack_only_resolves_in_attack_phase() {
        let mut g = two_player_game(ScriptedDice::new([7, 5, 12]));
        // Still in Initiative.
        assert!(g.attack(0, 1, false).is_none());
    }

    #[test]
    fn attack_hit_applies_damage_to_target() {
        // initiative rolls 7,5 (player1 first); then attack roll 11 (a hit that
        // is not a natural 12, so it triggers no critical-hit roll).
        let mut g = two_player_game(ScriptedDice::new([7, 5, 11]));
        g.advance(); // initiative -> order [1,0]
        // step to Attack phase: end both movements
        g.advance();
        g.advance();
        assert_eq!(g.phase, Phase::Attack);
        // current player is activation_order[0] = 1; unit 1 attacks unit 0.
        assert_eq!(g.current_player(), 1);
        let r = g.attack(1, 0, false).expect("attack should resolve");
        assert!(r.hit);
        let target = g.unit(0).unwrap();
        // short-range damage 3 hits armor 4 -> armor 1.
        assert_eq!(target.cur_armor, 1);
    }

    #[test]
    fn cannot_attack_friendly_unit() {
        let mut g = two_player_game(ScriptedDice::new([7, 5]));
        g.advance();
        g.advance();
        g.advance();
        let attacker = g.current_player();
        // attacking own unit returns None
        let own = g.units.iter().find(|u| u.player == attacker).unwrap().id;
        let other_own = own; // only one unit per player here
        assert!(g.attack(own, other_own, false).is_none());
    }

    #[test]
    fn lethal_attacks_declare_a_winner() {
        // A unit may fire only once per turn, so soften the target first and let
        // a single short-range hit (roll 11, not a natural 12) be lethal: 3
        // damage destroys its remaining 2 structure (no crit on a dead unit).
        let mut g = two_player_game(ScriptedDice::new([7, 5, 11]));
        g.advance();
        g.advance();
        g.advance();
        let shooter = g.current_player();
        let target = g.units.iter().find(|u| u.player != shooter).unwrap().id;
        let shooter_unit = g.units.iter().find(|u| u.player == shooter).unwrap().id;
        {
            let t = g.unit_mut(target).unwrap();
            t.cur_armor = 0;
            t.cur_structure = 2;
        }
        g.attack(shooter_unit, target, false); // 3 damage destroys 2 structure
        assert!(g.unit(target).unwrap().is_destroyed());
        assert_eq!(g.winner, Some(shooter));
    }

    #[test]
    fn end_phase_resets_movement_and_increments_turn() {
        let mut g = two_player_game(ScriptedDice::new([3, 7]));
        g.advance(); // -> Movement
        g.units[0].set_position((5.0, 0.0));
        g.units[0].finalise_position();
        assert!(!g.units[0].lines.is_empty());
        // March to End and resolve.
        g.advance();
        g.advance(); // -> Attack
        g.advance();
        g.advance(); // -> End
        g.advance(); // resolve
        assert_eq!(g.turn, 2);
        assert!(g.units[0].lines.is_empty());
    }

    #[test]
    fn overheated_unit_shuts_down_in_end_phase() {
        // A unit that overheats to the top of the scale shuts down at End Phase.
        let mut g = two_player_game(ScriptedDice::new([3, 7]));
        g.units[0].heat = 2;
        g.units[0].attacked_this_turn = true;
        g.units[0].overheat_used_this_turn = 2; // 2 + 2 = 4 = shutdown
        // advance through to End phase and resolve.
        g.advance(); // Movement
        g.advance();
        g.advance(); // Attack
        g.advance();
        g.advance(); // End
        g.advance(); // resolve
        assert!(g.units[0].shutdown);
        assert_eq!(g.units[0].heat, HEAT_SHUTDOWN_THRESHOLD);
    }

    #[test]
    fn cover_raises_the_to_hit_number() {
        // Put the target in woods; a roll that would hit in the open now misses.
        let units = vec![
            UnitState::new(0, 0, card("Atlas"), "FF0000", (0.0, 0.0)),
            UnitState::new(1, 1, card("Wolf"), "0000FF", (3.0, 0.0)),
        ];
        let terrain = vec![TerrainFeature::new(TerrainKind::Woods, "Forest", -10.0, -10.0, 40.0, 40.0)];
        // initiative 7,5 -> player1 first; attack roll 3.
        let mut g = GameState::new(units, terrain, ScriptedDice::new([7, 5, 3]));
        g.advance();
        g.advance();
        g.advance();
        // Attacker is player 1 (the Wolf, id 1); target player 0 (id 0) in woods.
        // TN = pilot 4 + woods 1 + attacker standstill -1 = 4 (target stationary,
        // short range). Roll 3 -> miss.
        let r = g.attack(1, 0, false).unwrap();
        assert_eq!(r.target_number, 4);
        assert!(!r.hit);
    }

    fn two_player_deploy(dice: ScriptedDice) -> GameState<ScriptedDice> {
        let units = vec![
            UnitState::new(0, 0, card("A"), "FF0000", (5.0, 10.0)),
            UnitState::new(1, 1, card("B"), "0000FF", (55.0, 10.0)),
        ];
        GameState::new_deploying(units, Vec::new(), dice)
    }

    #[test]
    fn deployment_starts_before_the_turn_loop() {
        // P0 rolls 3, P1 rolls 9 → P0 (lower) places first.
        let g = two_player_deploy(ScriptedDice::new([3, 9]));
        assert_eq!(g.phase, Phase::Deployment);
        assert_eq!(g.activation_order, vec![0, 1]);
        assert_eq!(g.current_player(), 0);
    }

    #[test]
    fn deploy_position_is_clamped_to_the_home_zone() {
        use crate::scenario::BOARD_H;
        let mut g = two_player_deploy(ScriptedDice::new([3, 9]));
        // Player 0 places first; a drag far outside the left band is clamped.
        g.set_deploy_position(0, (100.0, 100.0));
        assert_eq!(g.unit(0).unwrap().position, (DEPLOY_DEPTH, BOARD_H));
        // The opponent's unit cannot be moved out of turn.
        g.set_deploy_position(1, (30.0, 30.0));
        assert_eq!(g.unit(1).unwrap().position, (55.0, 10.0));
    }

    #[test]
    fn deploying_all_units_enters_the_turn_loop() {
        let mut g = two_player_deploy(ScriptedDice::new([3, 9]));
        assert_eq!(g.current_player(), 0);
        g.advance(); // P0 places its only unit
        assert_eq!(g.phase, Phase::Deployment);
        assert_eq!(g.current_player(), 1); // alternates to the other side
        g.advance(); // P1 places -> all deployed
        assert_eq!(g.phase, Phase::Initiative);
        assert_eq!(g.turn, 1);
    }
}
