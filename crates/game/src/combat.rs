//! Combat resolution: range brackets, the to-hit number, dice resolution,
//! damage application, and critical hits.
//!
//! Alpha Strike collapses BattleTech's many rolls into a single 2D6 check:
//! roll `>= target number` to hit, then apply the unit's S/M/L damage directly
//! to the target's Armor (overflowing into Structure). The target number is
//! (rules.md §5, §9):
//!
//! ```text
//! TN = pilot skill + target movement modifier + range modifier
//!      + attacker movement modifier + terrain/cover modifier
//!      + attacker heat level + 2 per attacker Fire-Control hit
//! ```

use crate::dice::Dice;
use crate::terrain::Cover;
use crate::unit::{DamageValues, MovementMode, UnitState, MIN_MOVE};

/// Upper bound (inclusive) of the Short range bracket, in inches.
pub const SHORT_RANGE: f64 = 6.0;
/// Upper bound (inclusive) of the Medium range bracket, in inches.
pub const MEDIUM_RANGE: f64 = 24.0;
/// Upper bound (inclusive) of the Long range bracket, in inches.
pub const LONG_RANGE: f64 = 42.0;

/// Heat level at which a unit shuts down (the "S" box on the heat scale).
pub const HEAT_SHUTDOWN_THRESHOLD: u32 = 4;
/// The maximum heat a unit can hold; overheating cannot exceed this.
pub const HEAT_MAX: u32 = 4;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Range {
    Short,
    Medium,
    Long,
    OutOfRange,
}

impl Range {
    pub fn as_str(&self) -> &'static str {
        match self {
            Range::Short => "Short",
            Range::Medium => "Medium",
            Range::Long => "Long",
            Range::OutOfRange => "Out of range",
        }
    }
}

/// Classify a straight-line distance (inches) into a range bracket.
pub fn range_of(distance: f64) -> Range {
    if distance <= SHORT_RANGE {
        Range::Short
    } else if distance <= MEDIUM_RANGE {
        Range::Medium
    } else if distance <= LONG_RANGE {
        Range::Long
    } else {
        Range::OutOfRange
    }
}

/// To-hit penalty for firing at longer ranges (rules.md §9).
pub fn range_modifier(range: Range) -> i32 {
    match range {
        Range::Short => 0,
        Range::Medium => 2,
        Range::Long => 4,
        Range::OutOfRange => i32::MAX,
    }
}

/// A reasonable default TMM for a unit's MV, used to populate card stats. The
/// real game reads TMM from the unit card; this is only a convenience.
pub fn tmm_for_mv(mv_inches: f64) -> i32 {
    match mv_inches as i32 {
        i32::MIN..=2 => 0,
        3..=6 => 1,
        7..=10 => 2,
        11..=14 => 3,
        15..=18 => 4,
        _ => 5,
    }
}

/// The target's contribution to the to-hit number (rules.md §9 "Target
/// Movement Modifiers"):
///
/// * Immobile / shut down → −4 (much easier to hit).
/// * Standstill → +0.
/// * Ground move → +TMM (−1 more if at Heat Level 2+).
/// * Jumping → +TMM +1 (heat does not affect jumping TMM).
pub fn target_movement_modifier(target: &UnitState) -> i32 {
    if target.is_immobile() {
        return -4;
    }
    match target.mode {
        MovementMode::Stationary => 0,
        MovementMode::Ground => {
            let heat_penalty = if target.heat >= 2 { 1 } else { 0 };
            (target.cur_tmm - heat_penalty).max(0)
        }
        MovementMode::Jump => target.cur_tmm + 1,
    }
}

/// How the attacker's own movement worsens its aim (rules.md §9 "Attacker
/// Movement Modifiers").
pub fn attacker_move_modifier(mode: MovementMode) -> i32 {
    match mode {
        MovementMode::Stationary => -1, // Standstill
        MovementMode::Ground => 0,
        MovementMode::Jump => 2,
    }
}

/// To-hit penalty from the target's cover. Partial cover and occupied/
/// intervening woods are both +1 (rules.md §9 "Other Modifiers").
pub fn terrain_modifier(cover: Cover) -> i32 {
    match cover {
        Cover::None => 0,
        Cover::Partial => 1,
        Cover::Woods => 1,
    }
}

/// One labelled term of the to-hit sum, for showing the player *why* a shot is
/// as hard as it is (see [`to_hit_components`]).
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct ToHitComponent {
    pub label: String,
    pub value: i32,
}

/// The itemised to-hit number: every contributing modifier with a human label,
/// in the order the rules list them (rules.md §5 Step 4). Summing the `value`s
/// gives [`to_hit_number`]; the always-present terms (skill, range, both
/// movement mods) are kept even when zero so the breakdown reads in full, while
/// the situational terms (cover, heat, fire control) appear only when non-zero.
pub fn to_hit_components(
    attacker: &UnitState,
    target: &UnitState,
    range: Range,
    cover: Cover,
) -> Vec<ToHitComponent> {
    let mut c = Vec::new();
    let mut push = |label: &str, value: i32| c.push(ToHitComponent { label: label.into(), value });

    push("Pilot skill", attacker.card.pilot_skill);
    push(&format!("{} range", range.as_str()), range_modifier(range));

    let target_label = if target.is_immobile() {
        "Target immobile/shutdown"
    } else {
        match target.mode {
            MovementMode::Stationary => "Target stood still",
            MovementMode::Ground => "Target movement (TMM)",
            MovementMode::Jump => "Target jumped (TMM +1)",
        }
    };
    push(target_label, target_movement_modifier(target));

    let attacker_label = match attacker.mode {
        MovementMode::Stationary => "Attacker stood still",
        MovementMode::Ground => "Attacker moved",
        MovementMode::Jump => "Attacker jumped",
    };
    push(attacker_label, attacker_move_modifier(attacker.mode));

    let cover_mod = terrain_modifier(cover);
    if cover_mod != 0 {
        let label = match cover {
            Cover::Woods => "Woods",
            Cover::Partial => "Partial cover",
            Cover::None => "Cover",
        };
        push(label, cover_mod);
    }
    if attacker.heat > 0 {
        push(&format!("Attacker heat {}", attacker.heat), attacker.heat as i32);
    }
    if attacker.fire_control_hits > 0 {
        push(
            &format!("Fire-control hits ×{}", attacker.fire_control_hits),
            2 * attacker.fire_control_hits as i32,
        );
    }
    c
}

/// The 2D6 number the attacker must meet or beat — the sum of every term in
/// [`to_hit_components`].
pub fn to_hit_number(attacker: &UnitState, target: &UnitState, range: Range, cover: Cover) -> i32 {
    to_hit_components(attacker, target, range, cover)
        .iter()
        .map(|c| c.value)
        .sum()
}

/// Probability that a fair 2D6 roll meets or beats `target_number` (0.0–1.0).
/// Used to preview a shot before committing to it.
pub fn hit_probability(target_number: i32) -> f64 {
    // Ways to roll each 2D6 sum 2..=12 (out of 36 equally-likely outcomes).
    const WAYS: [i32; 11] = [1, 2, 3, 4, 5, 6, 5, 4, 3, 2, 1];
    if target_number <= 2 {
        return 1.0;
    }
    if target_number > 12 {
        return 0.0;
    }
    let favorable: i32 = (target_number..=12).map(|s| WAYS[(s - 2) as usize]).sum();
    favorable as f64 / 36.0
}

/// How many overheat points an attack would actually spend: the attacker's OV,
/// capped by the room left on the heat scale, and only when the bonus applies at
/// this range (Short/Medium, or Long too with `OVL`). Returns 0 if `requested`
/// is false. Shared by [`resolve_attack`] and attack previews.
pub fn overheat_points(attacker: &UnitState, range: Range, requested: bool) -> u32 {
    if !requested {
        return 0;
    }
    let room = HEAT_MAX.saturating_sub(attacker.heat);
    let usable = attacker.card.overheat.min(room);
    let applies = matches!(range, Range::Short | Range::Medium)
        || (range == Range::Long && attacker.card.ovl);
    if applies {
        usable
    } else {
        0
    }
}

/// Damage dealt at a given range bracket.
pub fn damage_for_range(damage: &DamageValues, range: Range) -> u32 {
    match range {
        Range::Short => damage.short,
        Range::Medium => damage.medium,
        Range::Long => damage.long,
        Range::OutOfRange => 0,
    }
}

/// A critical-hit result from the Determining Critical Hits Table (rules.md §9).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum CriticalHit {
    None,
    Ammo,
    Engine,
    FireControl,
    Weapon,
    Mp,
    UnitDestroyed,
}

impl CriticalHit {
    pub fn as_str(&self) -> &'static str {
        match self {
            CriticalHit::None => "No Critical Hit",
            CriticalHit::Ammo => "Ammo Hit",
            CriticalHit::Engine => "Engine Hit",
            CriticalHit::FireControl => "Fire Control Hit",
            CriticalHit::Weapon => "Weapon Hit",
            CriticalHit::Mp => "MP Hit",
            CriticalHit::UnitDestroyed => "Unit Destroyed",
        }
    }
}

/// Map a 2D6 roll to a critical-hit effect (rules.md §9 table).
pub fn critical_hit_for_roll(roll: u8) -> CriticalHit {
    match roll {
        2 => CriticalHit::Ammo,
        3 => CriticalHit::Engine,
        4 => CriticalHit::FireControl,
        6 => CriticalHit::Weapon,
        8 => CriticalHit::Mp,
        9 => CriticalHit::Weapon,
        11 => CriticalHit::FireControl,
        12 => CriticalHit::UnitDestroyed,
        // 5, 7, 10 (and any out-of-band value) → no effect.
        _ => CriticalHit::None,
    }
}

/// Apply a critical-hit effect to a unit (rules.md §5 "Critical Hit Effects").
pub fn apply_critical_hit(target: &mut UnitState, crit: CriticalHit) {
    match crit {
        CriticalHit::None => {}
        CriticalHit::Ammo => {
            if target.card.ene || target.card.caseii {
                // No ammo to cook off / protected — ignored.
            } else if target.card.case {
                // CASE: survive, but take 1 extra point of damage.
                apply_damage(target, 1);
            } else {
                target.cur_structure = 0;
            }
        }
        CriticalHit::Engine => {
            target.engine_hits += 1;
            if target.engine_hits >= 2 {
                target.cur_structure = 0; // second engine hit is fatal
            }
        }
        CriticalHit::FireControl => target.fire_control_hits += 1,
        CriticalHit::Weapon => target.cur_damage = target.cur_damage.reduced_by_one(),
        CriticalHit::Mp => {
            // Lose half current Move and TMM, minimum loss 2" / 1.
            let mv_loss = (target.cur_mv / 2.0).round().max(MIN_MOVE);
            target.cur_mv = (target.cur_mv - mv_loss).max(0.0);
            let tmm_loss = ((target.cur_tmm as f64) / 2.0).round() as i32;
            let tmm_loss = tmm_loss.max(1);
            target.cur_tmm = (target.cur_tmm - tmm_loss).max(0);
        }
        CriticalHit::UnitDestroyed => target.cur_structure = 0,
    }
}

/// The outcome of a single attack, suitable for logging and UI display.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct AttackResult {
    pub roll: u8,
    pub target_number: i32,
    pub hit: bool,
    pub range: Range,
    pub damage: u32,
    /// False if the target was out of range (no roll was made).
    pub in_range: bool,
    /// False if terrain blocked the line of sight (no roll was made). Line of
    /// sight is a map-level concern resolved by [`crate::state`]; a bare
    /// [`resolve_attack`] always reports `true`.
    pub has_los: bool,
    /// Bonus damage delivered by overheating (included in `damage`, hits only).
    pub overheat_bonus: u32,
    /// Overheat points that incur heat at the End Phase (charged hit or miss).
    pub overheat_heat: u32,
    /// The critical hit rolled as a result of this attack, if any.
    pub crit: Option<CriticalHit>,
}

/// Resolve an attack: classify range, compute the TN, roll, and decide damage.
/// Does **not** mutate the target or apply heat — the caller applies
/// `result.damage` via [`apply_damage`] and charges `result.overheat_heat`.
///
/// `overheat` requests spending the attacker's overheat value. The points
/// actually spent are capped by the heat scale, and the bonus only helps at
/// Short/Medium range (or Long range too, if the unit has OVL).
pub fn resolve_attack(
    attacker: &UnitState,
    target: &UnitState,
    distance: f64,
    cover: Cover,
    overheat: bool,
    dice: &mut dyn Dice,
) -> AttackResult {
    let range = range_of(distance);
    if range == Range::OutOfRange {
        return AttackResult {
            roll: 0,
            target_number: range_modifier(range),
            hit: false,
            range,
            damage: 0,
            in_range: false,
            has_los: true,
            overheat_bonus: 0,
            overheat_heat: 0,
            crit: None,
        };
    }

    // How many overheat points may actually be spent (capped by the heat scale)
    // and whether they yield a damage bonus at this range.
    let overheat_points = if overheat {
        let room = HEAT_MAX.saturating_sub(attacker.heat);
        let usable = attacker.card.overheat.min(room);
        let applies = matches!(range, Range::Short | Range::Medium)
            || (range == Range::Long && attacker.card.ovl);
        if applies {
            usable
        } else {
            0
        }
    } else {
        0
    };

    let target_number = to_hit_number(attacker, target, range, cover);
    let roll = dice.roll_2d6();
    let hit = i32::from(roll) >= target_number;

    let (damage, overheat_bonus) = if hit {
        let base = damage_for_range(&attacker.cur_damage, range);
        (base + overheat_points, overheat_points)
    } else {
        (0, 0)
    };

    AttackResult {
        roll,
        target_number,
        hit,
        range,
        damage,
        in_range: true,
        has_los: true,
        overheat_bonus,
        overheat_heat: overheat_points,
        crit: None,
    }
}

/// Apply `amount` damage to a unit: Armor absorbs first, the remainder spills
/// into Structure. Structure is floored at zero (the unit is then destroyed).
pub fn apply_damage(target: &mut UnitState, amount: u32) {
    if amount <= target.cur_armor {
        target.cur_armor -= amount;
    } else {
        let overflow = amount - target.cur_armor;
        target.cur_armor = 0;
        target.cur_structure = target.cur_structure.saturating_sub(overflow);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dice::ScriptedDice;
    use crate::unit::{DamageValues, UnitCard};

    fn card() -> UnitCard {
        UnitCard {
            name: "Test".into(),
            mv_inches: 10.0,
            tmm: 2,
            damage: DamageValues {
                short: 3,
                medium: 2,
                long: 1,
            },
            armor: 4,
            structure: 3,
            pilot_skill: 4,
            overheat: 2,
            ..Default::default()
        }
    }

    fn unit() -> UnitState {
        UnitState::new(0, 0, card(), "FF0000", (0.0, 0.0))
    }

    #[test]
    fn range_brackets_respect_boundaries() {
        assert_eq!(range_of(0.0), Range::Short);
        assert_eq!(range_of(6.0), Range::Short);
        assert_eq!(range_of(6.01), Range::Medium);
        assert_eq!(range_of(24.0), Range::Medium);
        assert_eq!(range_of(24.01), Range::Long);
        assert_eq!(range_of(42.0), Range::Long);
        assert_eq!(range_of(42.01), Range::OutOfRange);
    }

    #[test]
    fn to_hit_sums_every_modifier() {
        let attacker = unit();
        let mut target = unit();
        target.mode = MovementMode::Ground; // moved -> TMM 2 applies
        // pilot 4 + target TMM 2 + medium 2 + attacker stationary -1 + woods 1
        let tn = to_hit_number(&attacker, &target, Range::Medium, Cover::Woods);
        assert_eq!(tn, 8);
    }

    #[test]
    fn attack_hits_when_roll_meets_target_number() {
        let attacker = unit();
        let target = unit();
        // TN = pilot 4 + attacker stationary -1 = 3 at short range, stationary target.
        let mut dice = ScriptedDice::new([3]);
        let r = resolve_attack(&attacker, &target, 3.0, Cover::None, false, &mut dice);
        assert!(r.hit);
        assert_eq!(r.range, Range::Short);
        assert_eq!(r.damage, 3);
    }

    #[test]
    fn attack_misses_below_target_number() {
        let attacker = unit();
        let target = unit();
        let mut dice = ScriptedDice::new([2]); // below TN 3
        let r = resolve_attack(&attacker, &target, 3.0, Cover::None, false, &mut dice);
        assert!(!r.hit);
        assert_eq!(r.damage, 0);
    }

    #[test]
    fn out_of_range_makes_no_roll() {
        let attacker = unit();
        let target = unit();
        let mut dice = ScriptedDice::new([]); // must not be consulted
        let r = resolve_attack(&attacker, &target, 100.0, Cover::None, false, &mut dice);
        assert!(!r.in_range);
        assert!(!r.hit);
    }

    #[test]
    fn damage_depletes_armor_then_structure() {
        let mut target = unit(); // armor 4, structure 3
        apply_damage(&mut target, 2);
        assert_eq!(target.cur_armor, 2);
        assert_eq!(target.cur_structure, 3);
        apply_damage(&mut target, 3); // 2 absorbed by armor, 1 spills
        assert_eq!(target.cur_armor, 0);
        assert_eq!(target.cur_structure, 2);
    }
}
