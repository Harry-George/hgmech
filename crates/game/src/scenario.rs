//! The battlefield map, loaded from embedded JSON under `data/`.
//!
//! The scenario layout (terrain zones) lives in `data/maps/<name>.json`, baked
//! into the binary at compile time with `include_str!` so loading stays
//! synchronous and needs no backend. Units are no longer placed here — players
//! build their forces from [`crate::catalog`] and deploy them on the board
//! during the Deployment phase (see [`crate::state`]). All coordinates and
//! distances are in board **inches**; the UI multiplies by a pixel scale for
//! rendering.

use serde::Deserialize;

use crate::terrain::TerrainFeature;

/// Width of the playable board, in inches.
pub const BOARD_W: f64 = 60.0;
/// Height of the playable board, in inches.
pub const BOARD_H: f64 = 48.0;

/// A scenario map: just the terrain zones (units are deployed, not preset).
#[derive(Deserialize)]
struct Map {
    terrain: Vec<TerrainFeature>,
}

/// The demo battlefield's terrain (a wood and a rubble field), from
/// `data/maps/demo.json`.
pub fn demo_terrain() -> Vec<TerrainFeature> {
    const MAP_JSON: &str = include_str!("../../../data/maps/demo.json");
    let map: Map = serde_json::from_str(MAP_JSON).expect("valid demo map");
    map.terrain
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::terrain::TerrainKind;

    #[test]
    fn demo_terrain_loads_from_the_embedded_map() {
        let terrain = demo_terrain();
        assert_eq!(terrain.len(), 2);
        assert_eq!(terrain[0].kind, TerrainKind::Woods);
        assert_eq!(terrain[1].kind, TerrainKind::Cover);
    }
}
