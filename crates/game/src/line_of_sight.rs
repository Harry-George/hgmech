//! Line of sight (rules.md §5 Step 1).
//!
//! Whether an attacker can "see" a target is decided geometrically from the
//! terrain lying between them. All terrain is modelled as axis-aligned
//! rectangular zones (see [`crate::terrain`]); a unit is modelled as a small
//! circular base of radius [`BASE_RADIUS`] centred on its position.
//!
//! Two kinds of terrain affect sight:
//!
//! * **Solid** terrain — buildings, hills ([`TerrainKind::is_solid`]) — blocks
//!   the view. We sample the target's silhouette (a segment of width
//!   `2·BASE_RADIUS` perpendicular to the line of sight) and measure how much of
//!   it hides behind solid zones:
//!     - less than **1/3** of the target visible → **LOS blocked**;
//!     - between **1/3 and 2/3** of the target hidden → LOS holds but the target
//!       gains **partial cover** (+1 to hit);
//!     - up to 1/3 hidden → clear.
//! * **Woods** are not solid: they block only when the line passes through
//!   **≥ [`WOODS_BLOCK_INCHES`]** of woods; a shorter stretch of intervening or
//!   occupied woods adds **+1** to hit instead.
//!
//! A 'Mech standing in **water** always gains partial cover from it. Base-to-base
//! (adjacent) units always have line of sight to each other. Other units are
//! never terrain and never block sight, so they are not considered here.
//!
//! Simplifications (documented in `tests/todo.txt`): the 1/3 and 2/3 fractions
//! are estimated by sampling rather than exact area, and partial cover and woods
//! each contribute +1 but do not stack — the stronger single source is reported.

use crate::terrain::{Cover, TerrainFeature};

/// Half-width of a unit's base, in inches — an abstraction of a 'Mech's physical
/// base used to give the target a silhouette to hide behind terrain.
pub const BASE_RADIUS: f64 = 0.5;

/// A line passing through this many inches of woods (or more) is blocked
/// (rules.md §5 Step 1).
pub const WOODS_BLOCK_INCHES: f64 = 6.0;

/// How many points across the target's silhouette are sampled when measuring how
/// much of it hides behind solid terrain. Higher is smoother; the exact value
/// only matters near the 1/3 and 2/3 boundaries.
const LOS_SAMPLES: usize = 32;

/// The result of tracing a line of sight from an attacker to a target.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Sighting {
    /// True if terrain blocks the view entirely (no attack is possible).
    pub blocked: bool,
    /// The cover the target enjoys when the line of sight holds. Both partial
    /// cover and intervening/occupied woods are `+1` (see
    /// [`crate::combat::terrain_modifier`]).
    pub cover: Cover,
}

impl Sighting {
    /// An unobstructed sighting with no cover.
    pub fn clear() -> Self {
        Self {
            blocked: false,
            cover: Cover::None,
        }
    }

    /// The to-hit penalty the cover imposes (0 or +1).
    pub fn to_hit_modifier(&self) -> i32 {
        crate::combat::terrain_modifier(self.cover)
    }
}

/// Trace the line of sight from `attacker` to `target` across `terrain`.
pub fn line_of_sight(
    attacker: (f64, f64),
    target: (f64, f64),
    terrain: &[TerrainFeature],
) -> Sighting {
    let dx = target.0 - attacker.0;
    let dy = target.1 - attacker.1;
    let dist = (dx * dx + dy * dy).sqrt();

    // A 'Mech in water always has partial cover from it, regardless of sight.
    let in_water = terrain
        .iter()
        .any(|t| t.kind.is_water() && t.contains(target));

    // Base-to-base units always have line of sight to each other.
    if dist <= 2.0 * BASE_RADIUS {
        return Sighting {
            blocked: false,
            cover: if in_water { Cover::Partial } else { Cover::None },
        };
    }

    // --- Solid terrain: how much of the target's silhouette is hidden? -------
    // Sample points across a segment perpendicular to the line of sight, and
    // count those whose sightline crosses a solid zone before reaching them.
    let (ux, uy) = (dx / dist, dy / dist);
    let (perp_x, perp_y) = (-uy, ux);
    let mut hidden = 0usize;
    for i in 0..LOS_SAMPLES {
        let frac = i as f64 / (LOS_SAMPLES - 1) as f64; // 0.0 ..= 1.0
        let offset = (frac * 2.0 - 1.0) * BASE_RADIUS; // -r ..= +r
        let sample = (target.0 + perp_x * offset, target.1 + perp_y * offset);
        let occluded = terrain
            .iter()
            .any(|t| t.kind.is_solid() && overlap_length(attacker, sample, t) > f64::EPSILON);
        if occluded {
            hidden += 1;
        }
    }
    let visible_frac = 1.0 - hidden as f64 / LOS_SAMPLES as f64;
    let solid_blocked = visible_frac < 1.0 / 3.0;
    let solid_partial = !solid_blocked && visible_frac < 2.0 / 3.0;

    // --- Woods: how far does the centre-to-centre line run through woods? -----
    let woods_len: f64 = terrain
        .iter()
        .filter(|t| t.kind.is_woods())
        .map(|t| overlap_length(attacker, target, t))
        .sum();
    let woods_blocked = woods_len >= WOODS_BLOCK_INCHES;
    let woods_conceals = !woods_blocked && woods_len > f64::EPSILON;

    let blocked = solid_blocked || woods_blocked;
    let cover = if blocked {
        Cover::None
    } else if woods_conceals {
        Cover::Woods
    } else if solid_partial || in_water {
        Cover::Partial
    } else {
        Cover::None
    };

    Sighting { blocked, cover }
}

/// Length of the portion of segment `a`→`b` that lies inside `rect`, in inches
/// (0 if it misses). Uses Liang–Barsky clipping against the axis-aligned zone.
fn overlap_length(a: (f64, f64), b: (f64, f64), rect: &TerrainFeature) -> f64 {
    match clip_segment(a, b, rect) {
        Some((t0, t1)) if t1 > t0 => {
            let dx = b.0 - a.0;
            let dy = b.1 - a.1;
            (t1 - t0) * (dx * dx + dy * dy).sqrt()
        }
        _ => 0.0,
    }
}

/// Clip segment `a`→`b` to the axis-aligned rectangle, returning the parameter
/// range `[t0, t1] ⊆ [0, 1]` that lies inside it (Liang–Barsky), or `None` if
/// the segment misses the rectangle entirely.
fn clip_segment(a: (f64, f64), b: (f64, f64), rect: &TerrainFeature) -> Option<(f64, f64)> {
    let (x0, y0) = a;
    let dx = b.0 - a.0;
    let dy = b.1 - a.1;
    let (xmin, xmax) = (rect.x, rect.x + rect.w);
    let (ymin, ymax) = (rect.y, rect.y + rect.h);

    let p = [-dx, dx, -dy, dy];
    let q = [x0 - xmin, xmax - x0, y0 - ymin, ymax - y0];
    let mut t0 = 0.0f64;
    let mut t1 = 1.0f64;
    for i in 0..4 {
        if p[i].abs() < f64::EPSILON {
            // Segment is parallel to this edge; reject if it starts outside it.
            if q[i] < 0.0 {
                return None;
            }
        } else {
            let r = q[i] / p[i];
            if p[i] < 0.0 {
                if r > t1 {
                    return None;
                }
                if r > t0 {
                    t0 = r;
                }
            } else {
                if r < t0 {
                    return None;
                }
                if r < t1 {
                    t1 = r;
                }
            }
        }
    }
    Some((t0, t1))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::terrain::TerrainKind;

    fn feature(kind: TerrainKind, x: f64, y: f64, w: f64, h: f64) -> TerrainFeature {
        TerrainFeature::new(kind, "T", x, y, w, h)
    }

    #[test]
    fn open_ground_is_a_clear_sighting() {
        let s = line_of_sight((0.0, 0.0), (20.0, 0.0), &[]);
        assert!(!s.blocked);
        assert_eq!(s.cover, Cover::None);
    }

    #[test]
    fn a_solid_wall_covering_the_target_blocks_los() {
        let wall = feature(TerrainKind::Building, 9.0, -5.0, 2.0, 10.0);
        let s = line_of_sight((0.0, 0.0), (20.0, 0.0), &[wall]);
        assert!(s.blocked);
    }

    #[test]
    fn a_solid_wall_hiding_half_the_target_gives_partial_cover() {
        // Wall top edge at y = 0 (the line-of-sight height); it hides the lower
        // half of the target silhouette.
        let wall = feature(TerrainKind::Building, 9.0, -5.0, 2.0, 5.0);
        let s = line_of_sight((0.0, 0.0), (20.0, 0.0), &[wall]);
        assert!(!s.blocked);
        assert_eq!(s.cover, Cover::Partial);
        assert_eq!(s.to_hit_modifier(), 1);
    }

    #[test]
    fn a_solid_wall_out_of_the_sightline_gives_no_cover() {
        // Wall well below the target; no sightline reaches it.
        let wall = feature(TerrainKind::Building, 9.0, -20.0, 2.0, 15.0);
        let s = line_of_sight((0.0, 0.0), (20.0, 0.0), &[wall]);
        assert!(!s.blocked);
        assert_eq!(s.cover, Cover::None);
    }

    #[test]
    fn thin_woods_conceal_but_do_not_block() {
        let woods = feature(TerrainKind::Woods, 9.0, -10.0, 4.0, 20.0); // 4" across
        let s = line_of_sight((0.0, 0.0), (20.0, 0.0), &[woods]);
        assert!(!s.blocked);
        assert_eq!(s.cover, Cover::Woods);
        assert_eq!(s.to_hit_modifier(), 1);
    }

    #[test]
    fn six_inches_of_woods_block_los() {
        let woods = feature(TerrainKind::Woods, 7.0, -10.0, 6.0, 20.0); // 6" across
        let s = line_of_sight((0.0, 0.0), (20.0, 0.0), &[woods]);
        assert!(s.blocked);
    }

    #[test]
    fn standing_in_water_gives_partial_cover() {
        let water = feature(TerrainKind::Water, 15.0, -5.0, 10.0, 10.0);
        let s = line_of_sight((0.0, 0.0), (20.0, 0.0), &[water]);
        assert!(!s.blocked);
        assert_eq!(s.cover, Cover::Partial);
    }

    #[test]
    fn adjacent_units_always_have_los_through_a_wall() {
        let wall = feature(TerrainKind::Building, -5.0, -5.0, 10.0, 10.0);
        let s = line_of_sight((0.0, 0.0), (0.5, 0.0), &[wall]);
        assert!(!s.blocked);
    }
}
