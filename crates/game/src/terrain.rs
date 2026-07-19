//! Terrain features and the cover they grant.
//!
//! Terrain is modelled as axis-aligned rectangular zones on the board. A unit
//! standing inside a zone receives that zone's cover, which feeds a to-hit
//! modifier in [`crate::combat`].

use serde::{Deserialize, Serialize};

/// What a terrain zone is made of.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub enum TerrainKind {
    /// Open ground — no effect.
    Open,
    /// Light cover (rubble, low walls). Movement-only terrain: it grants no
    /// line-of-sight cover under the rules (see [`crate::line_of_sight`]).
    Cover,
    /// Woods — non-solid concealment. Blocks line of sight only when the line
    /// passes through ≥ 6" of it; shorter stretches add +1 to hit.
    Woods,
    /// Solid terrain (a building or hill) that blocks line of sight when it
    /// hides enough of the target.
    Building,
    /// Water — a 'Mech standing in it gains partial cover.
    Water,
}

impl TerrainKind {
    /// Solid terrain (buildings, hills) can block line of sight, unlike woods
    /// and water (rules.md §5 Step 1).
    pub fn is_solid(&self) -> bool {
        matches!(self, TerrainKind::Building)
    }

    pub fn is_woods(&self) -> bool {
        matches!(self, TerrainKind::Woods)
    }

    pub fn is_water(&self) -> bool {
        matches!(self, TerrainKind::Water)
    }
}

/// The cover a unit currently benefits from, ordered weakest to strongest.
#[derive(Clone, Copy, PartialEq, Eq, Debug, PartialOrd, Ord)]
pub enum Cover {
    None,
    Partial,
    Woods,
}

/// A rectangular terrain zone in board coordinates. Loaded from the `terrain`
/// array of a `data/maps/<name>.json` scenario file.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct TerrainFeature {
    pub kind: TerrainKind,
    pub label: String,
    pub x: f64,
    pub y: f64,
    pub w: f64,
    pub h: f64,
}

impl TerrainFeature {
    pub fn new(kind: TerrainKind, label: impl Into<String>, x: f64, y: f64, w: f64, h: f64) -> Self {
        Self {
            kind,
            label: label.into(),
            x,
            y,
            w,
            h,
        }
    }

    pub fn contains(&self, pos: (f64, f64)) -> bool {
        let (px, py) = pos;
        px >= self.x && px <= self.x + self.w && py >= self.y && py <= self.y + self.h
    }

    /// This is the cover the unit gets by standing "in" the said terrain feature.
    pub fn cover(&self) -> Cover {
        match self.kind {
            TerrainKind::Open | TerrainKind::Building => Cover::None,
            TerrainKind::Cover | TerrainKind::Water => Cover::Partial,
            TerrainKind::Woods => Cover::Woods,
        }
    }
}

/// The strongest cover among all features containing `pos`.
pub fn cover_at(pos: (f64, f64), features: &[TerrainFeature]) -> Cover {
    features
        .iter()
        .filter(|f| f.contains(pos))
        .map(TerrainFeature::cover)
        .max()
        .unwrap_or(Cover::None)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn contains_respects_bounds() {
        let f = TerrainFeature::new(TerrainKind::Woods, "W", 10.0, 10.0, 20.0, 20.0);
        assert!(f.contains((10.0, 10.0)));
        assert!(f.contains((30.0, 30.0)));
        assert!(f.contains((20.0, 20.0)));
        assert!(!f.contains((9.9, 20.0)));
        assert!(!f.contains((31.0, 20.0)));
    }

    #[test]
    fn cover_at_returns_none_in_the_open() {
        let features = [TerrainFeature::new(TerrainKind::Woods, "W", 0.0, 0.0, 10.0, 10.0)];
        assert_eq!(cover_at((100.0, 100.0), &features), Cover::None);
    }

    #[test]
    fn cover_at_picks_strongest_overlap() {
        let features = [
            TerrainFeature::new(TerrainKind::Cover, "C", 0.0, 0.0, 50.0, 50.0),
            TerrainFeature::new(TerrainKind::Woods, "W", 0.0, 0.0, 50.0, 50.0),
        ];
        assert_eq!(cover_at((25.0, 25.0), &features), Cover::Woods);
    }
}
