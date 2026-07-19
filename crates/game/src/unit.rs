//! Units: the pieces on the board and their Alpha Strike "card" stats.
//!
//! This module is pure game logic — it has no dependency on Leptos, `web-sys`,
//! or the DOM. Everything here is plain data plus the movement-budget rules that
//! used to live inside the `DraggableRobotState` Leptos component.

use serde::{Deserialize, Serialize};

/// A mobile unit can always move at least this far, regardless of penalties
/// (rules.md §4 "Minimum Movement").
pub const MIN_MOVE: f64 = 2.0;

/// Weight class of a unit. The numeric Size value (1–4) drives physical-attack
/// damage in the full rules; kept as a real type so logic can branch on it.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default, Serialize, Deserialize)]
pub enum Size {
    Light,
    #[default]
    Medium,
    Heavy,
    Assault,
}

impl Size {
    pub fn as_str(&self) -> &'static str {
        match self {
            Size::Light => "Light",
            Size::Medium => "Medium",
            Size::Heavy => "Heavy",
            Size::Assault => "Assault",
        }
    }

    /// The Size value (1–4) printed on the unit card.
    pub fn value(&self) -> u32 {
        match self {
            Size::Light => 1,
            Size::Medium => 2,
            Size::Heavy => 3,
            Size::Assault => 4,
        }
    }
}

/// How a unit moved this turn, in the rules' terms (rules.md §4 "Movement
/// Modes"). Free-drag movement only ever produces `Stationary` or `Ground`;
/// `Jump` is reachable through the domain API.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum MovementMode {
    /// Moved less than 1" (a "Standstill").
    Stationary,
    /// Moved at least 1" without jumping (the default "Ground Move").
    Ground,
    /// Jumping movement.
    Jump,
}

impl MovementMode {
    pub fn as_str(&self) -> &'static str {
        match self {
            MovementMode::Stationary => "Standstill",
            MovementMode::Ground => "Ground",
            MovementMode::Jump => "Jump",
        }
    }
}

/// A straight segment of a unit's movement path. Distances are in board
/// "inches" (which map 1:1 to on-screen pixels in the UI).
#[derive(Clone, PartialEq, Debug)]
pub struct Line {
    pub start: (f64, f64),
    pub end: (f64, f64),
}

impl Line {
    pub fn new(start: (f64, f64), end: (f64, f64)) -> Self {
        Self { start, end }
    }

    pub fn length(&self) -> f64 {
        let (x1, y1) = self.start;
        let (x2, y2) = self.end;
        ((x2 - x1).powi(2) + (y2 - y1).powi(2)).sqrt()
    }
}

/// Short / Medium / Long damage values printed on the unit card.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default, Serialize, Deserialize)]
pub struct DamageValues {
    pub short: u32,
    pub medium: u32,
    pub long: u32,
}

impl DamageValues {
    /// Reduce every range value by 1, flooring at 0 (a Weapon Hit critical).
    pub fn reduced_by_one(self) -> Self {
        Self {
            short: self.short.saturating_sub(1),
            medium: self.medium.saturating_sub(1),
            long: self.long.saturating_sub(1),
        }
    }
}

/// The immutable reference card for a unit type. Mutable, in-game stats that can
/// degrade (move, TMM, damage) are tracked on [`UnitState`], not here.
///
/// Loaded from `data/mechs/<name>.json`; every field defaults so a mech file
/// need only list the stats that differ from zero/false. `tmm` is derived from
/// the move value and `pilot_skill` is filled from the scenario, so both are
/// normally absent from the JSON (see [`crate::scenario`]).
#[derive(Clone, PartialEq, Debug, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct UnitCard {
    pub name: String,
    pub size: Size,
    /// Movement value (MV) in inches — the per-turn movement budget.
    pub mv_inches: f64,
    /// Target Movement Modifier printed on the card.
    pub tmm: i32,
    pub damage: DamageValues,
    /// Armor points (depleted before structure).
    pub armor: u32,
    /// Structure points (the unit is destroyed when these reach 0).
    pub structure: u32,
    /// Pilot skill — the base of every to-hit number (lower is better).
    pub pilot_skill: i32,
    /// Overheat value (OV): bonus damage available by accruing heat.
    pub overheat: u32,
    /// OVL — Overheat Long: overheat bonus also applies at Long range.
    pub ovl: bool,
    /// ENE — Energy: no ammunition to explode (ignores Ammo Hit criticals).
    pub ene: bool,
    /// CASE: survives an Ammo Hit, taking 1 extra damage instead of exploding.
    pub case: bool,
    /// CASEII: ignores Ammo Hit criticals entirely.
    pub caseii: bool,
    /// MEL — Melee: +1 physical attack damage (physical attacks not yet modelled).
    pub mel: bool,
    /// Point Value — approximate battlefield strength, used to tally each side's
    /// force in the force builder. Not used by combat logic.
    pub pv: u32,
    /// Tactical role (Scout, Striker, …) — flavour and force-building filter only.
    pub role: String,
    /// Jump movement in inches, when the unit has a `j` move. Display-only: the
    /// engine's movement is ground-only, so this does not feed the budget yet.
    pub jump_inches: f64,
    /// Raw Special Abilities text from the master list. Many abilities are not
    /// modelled; the full string is kept so the card can show it verbatim.
    pub specials: String,
    /// Optional artwork URL from the master list. When empty the UI falls back to
    /// a generated DiceBear avatar.
    pub image_url: String,
}

/// The mutable, in-game state of a single unit on the board.
#[derive(Clone, PartialEq, Debug)]
pub struct UnitState {
    pub id: usize,
    pub player: usize,
    pub card: UnitCard,
    /// DiceBear avatar seed colour, e.g. "FF0000".
    pub color: String,
    pub position: (f64, f64),
    /// Anchor for the in-progress movement segment; advances on each commit.
    pub starting_position: (f64, f64),
    pub lines: Vec<Line>,
    pub cur_armor: u32,
    pub cur_structure: u32,
    /// Current movement allowance — starts at `card.mv_inches`, reduced by MP Hits.
    pub cur_mv: f64,
    /// Current TMM — starts at `card.tmm`, reduced by MP Hits.
    pub cur_tmm: i32,
    /// Current damage values — start at `card.damage`, reduced by Weapon Hits.
    pub cur_damage: DamageValues,
    pub mode: MovementMode,
    pub heat: u32,
    pub shutdown: bool,
    /// Engine Hit criticals taken (2 destroys the unit).
    pub engine_hits: u32,
    /// Fire Control Hit criticals taken (+2 to-hit each).
    pub fire_control_hits: u32,
    /// Whether this unit made a weapon attack this turn (drives heat cooling).
    pub attacked_this_turn: bool,
    /// Overheat points spent this turn — applied to the heat scale at End Phase.
    pub overheat_used_this_turn: u32,
    /// Whether this unit was already a wreck at the start of the current turn.
    /// A unit killed *during* the turn is not finalised (removed) until the End
    /// Phase, so it may still make its own attack — see [`destroyed_this_turn`].
    ///
    /// [`destroyed_this_turn`]: UnitState::destroyed_this_turn
    pub destroyed_at_turn_start: bool,
}

impl UnitState {
    pub fn new(
        id: usize,
        player: usize,
        card: UnitCard,
        color: impl Into<String>,
        position: (f64, f64),
    ) -> Self {
        let cur_armor = card.armor;
        let cur_structure = card.structure;
        let cur_mv = card.mv_inches;
        let cur_tmm = card.tmm;
        let cur_damage = card.damage;
        Self {
            id,
            player,
            card,
            color: color.into(),
            position,
            starting_position: position,
            lines: Vec::new(),
            cur_armor,
            cur_structure,
            cur_mv,
            cur_tmm,
            cur_damage,
            mode: MovementMode::Stationary,
            heat: 0,
            shutdown: false,
            engine_hits: 0,
            fire_control_hits: 0,
            attacked_this_turn: false,
            overheat_used_this_turn: 0,
            destroyed_at_turn_start: false,
        }
    }

    pub fn is_owned_by(&self, player: usize) -> bool {
        self.player == player
    }

    pub fn is_destroyed(&self) -> bool {
        self.cur_structure == 0
    }

    /// True if the unit is a wreck now but was still alive at the start of this
    /// turn — i.e. it was destroyed *this* turn. Such a unit is not removed
    /// until the End Phase and may still make its one attack (rules.md §2:
    /// attacks are effectively simultaneous, so a unit killed this phase can
    /// return fire). A wreck left from an earlier turn is not "this turn's".
    pub fn destroyed_this_turn(&self) -> bool {
        self.is_destroyed() && !self.destroyed_at_turn_start
    }

    /// A unit that cannot move at all: shut down, or reduced to 0" of move by
    /// MP Hits. Immobile units get the −4 target modifier (rules.md §9).
    pub fn is_immobile(&self) -> bool {
        self.shutdown || self.cur_mv <= 0.0
    }

    /// The movement budget for this turn, in inches. Ground move is reduced by
    /// 2" per heat level, but a still-mobile unit can always move the 2"
    /// minimum (rules.md §4, §8). An immobile unit has a budget of 0.
    pub fn movement_budget(&self) -> f64 {
        if self.is_immobile() {
            return 0.0;
        }
        let reduced = self.cur_mv - 2.0 * self.heat as f64;
        reduced.max(MIN_MOVE)
    }

    /// Distance committed so far this turn (sum of finalised segments).
    fn committed(&self) -> f64 {
        self.lines.iter().map(Line::length).sum()
    }

    /// Total distance moved this turn including the in-progress segment.
    pub fn distance_moved(&self) -> f64 {
        self.committed() + Line::new(self.position, self.starting_position).length()
    }

    /// Remaining movement budget (budget minus everything moved this turn).
    pub fn available_movement(&self) -> f64 {
        self.movement_budget() - self.distance_moved()
    }

    /// Clamp a desired position so the in-progress segment never pushes total
    /// movement past the budget.
    fn clamp_to_budget(&self, target: (f64, f64)) -> (f64, f64) {
        let remaining = self.movement_budget() - self.committed();
        let (sx, sy) = self.starting_position;
        let (tx, ty) = target;
        let dx = tx - sx;
        let dy = ty - sy;
        let dist = (dx * dx + dy * dy).sqrt();
        if dist <= remaining || dist == 0.0 {
            target
        } else {
            let scale = (remaining / dist).max(0.0);
            (sx + dx * scale, sy + dy * scale)
        }
    }

    /// Move the unit toward `target`, clamped to the remaining budget.
    pub fn set_position(&mut self, target: (f64, f64)) {
        self.position = self.clamp_to_budget(target);
    }

    /// Commit the in-progress segment to the movement history if it fits the
    /// remaining budget, then re-anchor for the next segment. Updates `mode`.
    pub fn finalise_position(&mut self) {
        let line = Line::new(self.position, self.starting_position);
        if line.length() <= self.movement_budget() - self.committed() {
            self.lines.push(line);
            self.starting_position = self.position;
        }
        self.recompute_mode();
    }

    /// Undo the most recently committed segment.
    pub fn undo_last_move(&mut self) {
        if let Some(line) = self.lines.pop() {
            self.starting_position = line.end;
            self.position = line.end;
        }
        self.recompute_mode();
    }

    /// Movement path including the in-progress segment (for rendering).
    pub fn get_lines(&self) -> Vec<Line> {
        let mut lines = self.lines.clone();
        lines.push(Line::new(self.position, self.starting_position));
        lines
    }

    /// Derive the movement mode from how far the unit has moved. In Alpha Strike
    /// any ground movement of 1"+ is simply "Ground Move" (no Walk/Run split);
    /// `Jump` is set explicitly by the caller and is not inferred here.
    fn recompute_mode(&mut self) {
        if self.mode == MovementMode::Jump {
            return;
        }
        let moved = self.distance_moved();
        self.mode = if moved <= f64::EPSILON {
            MovementMode::Stationary
        } else {
            MovementMode::Ground
        };
    }

    /// Clear per-turn movement and combat-bookkeeping state at the start of a
    /// new turn. Leaves persistent state (armor/structure, heat, criticals)
    /// untouched — heat is resolved by the End phase *before* this is called.
    pub fn reset_for_new_turn(&mut self) {
        self.lines.clear();
        self.starting_position = self.position;
        self.mode = MovementMode::Stationary;
        self.attacked_this_turn = false;
        self.overheat_used_this_turn = 0;
        // Snapshot the destruction state for the turn about to begin: a unit
        // killed this turn was allowed to act; once the turn ends it becomes a
        // finalised wreck for every subsequent turn.
        self.destroyed_at_turn_start = self.is_destroyed();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_card(mv: f64) -> UnitCard {
        UnitCard {
            name: "Test".into(),
            size: Size::Medium,
            mv_inches: mv,
            tmm: 2,
            damage: DamageValues {
                short: 3,
                medium: 2,
                long: 1,
            },
            armor: 5,
            structure: 4,
            pilot_skill: 4,
            overheat: 2,
            ..Default::default()
        }
    }

    fn unit(mv: f64) -> UnitState {
        UnitState::new(0, 0, test_card(mv), "FF0000", (0.0, 0.0))
    }

    #[test]
    fn set_position_within_budget_is_exact() {
        let mut u = unit(100.0);
        u.set_position((30.0, 40.0)); // distance 50
        assert_eq!(u.position, (30.0, 40.0));
    }

    #[test]
    fn set_position_is_clamped_to_budget() {
        let mut u = unit(50.0);
        u.set_position((300.0, 400.0)); // distance 500, budget 50 -> scale 0.1
        assert!((u.position.0 - 30.0).abs() < 1e-9);
        assert!((u.position.1 - 40.0).abs() < 1e-9);
        assert!((Line::new((0.0, 0.0), u.position).length() - 50.0).abs() < 1e-9);
    }

    #[test]
    fn finalise_commits_and_reanchors() {
        let mut u = unit(100.0);
        u.set_position((30.0, 40.0));
        u.finalise_position();
        assert_eq!(u.lines.len(), 1);
        assert_eq!(u.starting_position, (30.0, 40.0));
        assert!((u.available_movement() - 50.0).abs() < 1e-9);
    }

    #[test]
    fn budget_is_shared_across_multiple_segments() {
        let mut u = unit(100.0);
        u.set_position((60.0, 0.0));
        u.finalise_position(); // used 60
        u.set_position((1000.0, 0.0)); // wants +940, only 40 left
        assert!((u.position.0 - 100.0).abs() < 1e-9);
        assert!(u.available_movement().abs() < 1e-9);
    }

    #[test]
    fn undo_restores_previous_anchor() {
        let mut u = unit(100.0);
        u.set_position((30.0, 40.0));
        u.finalise_position();
        u.undo_last_move();
        assert!(u.lines.is_empty());
        assert_eq!(u.position, (0.0, 0.0));
        assert!((u.available_movement() - 100.0).abs() < 1e-9);
    }

    #[test]
    fn mode_tracks_distance_moved() {
        let mut u = unit(100.0);
        assert_eq!(u.mode, MovementMode::Stationary);
        u.set_position((30.0, 0.0));
        u.finalise_position();
        assert_eq!(u.mode, MovementMode::Ground);
    }

    #[test]
    fn destroyed_when_structure_zero() {
        let mut u = unit(100.0);
        assert!(!u.is_destroyed());
        u.cur_structure = 0;
        assert!(u.is_destroyed());
    }

    #[test]
    fn reset_clears_movement_not_combat() {
        let mut u = unit(100.0);
        u.set_position((30.0, 0.0));
        u.finalise_position();
        u.cur_armor = 2;
        u.attacked_this_turn = true;
        u.reset_for_new_turn();
        assert!(u.lines.is_empty());
        assert_eq!(u.mode, MovementMode::Stationary);
        assert_eq!(u.cur_armor, 2);
        assert!(!u.attacked_this_turn);
    }

    #[test]
    fn killed_this_turn_is_distinct_from_a_prior_wreck() {
        let mut u = unit(100.0);
        // Alive at the start of the turn: not destroyed, nothing to finalise.
        assert!(!u.is_destroyed());
        assert!(!u.destroyed_this_turn());

        // Destroyed during this turn — still counts as "this turn's" casualty,
        // so it stays actionable enough to return fire before the End Phase.
        u.cur_structure = 0;
        assert!(u.is_destroyed());
        assert!(u.destroyed_this_turn());

        // The End-Phase reset finalises the wreck: from the next turn on it is a
        // prior-turn casualty and no longer "killed this turn".
        u.reset_for_new_turn();
        assert!(u.is_destroyed());
        assert!(!u.destroyed_this_turn());
    }
}
