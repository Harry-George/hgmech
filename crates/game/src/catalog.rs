//! The master unit list: the pool of units players may pick from.
//!
//! The raw source is an Alpha Strike CSV export (~3900 rows, ~1.9 MB) under
//! `data/`. Rather than embed and parse that text at runtime, the build script
//! ([`build.rs`]) parses it once at build time — filtering to `BM` (BattleMech)
//! rows, the only type the introductory rules this engine implements cover —
//! serializes the resulting cards to JSON, and DEFLATE-compresses them. Here we
//! simply inflate and deserialize that small embedded blob, so neither the CSV
//! text nor a CSV parser ships in the binary.
//!
//! The parser lives in [`crate::catalog_csv`] (shared with the build script);
//! this module only sees the finished [`UnitCard`]s.

use crate::unit::UnitCard;

/// The parsed BattleMech catalog: JSON produced by `build.rs`, DEFLATE-compressed
/// and embedded at compile time from `OUT_DIR`.
const CATALOG_DEFLATE: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/catalog.deflate"));

/// The full list of pickable units. Cards carry a default pilot skill of 4
/// (Regular); the force builder adjusts skill per unit.
///
/// The catalog is baked into the binary, so decoding is infallible in practice;
/// a failure here means a corrupt build artifact and is a bug, not a runtime
/// condition to handle.
pub fn available_units() -> Vec<UnitCard> {
    let json = miniz_oxide::inflate::decompress_to_vec(CATALOG_DEFLATE)
        .expect("embedded catalog is valid DEFLATE");
    serde_json::from_slice(&json).expect("embedded catalog is valid JSON")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::unit::{DamageValues, Size};

    #[test]
    fn catalog_is_large_and_all_battlemechs() {
        let units = available_units();
        assert!(
            units.len() > 100,
            "expected a large catalog, got {}",
            units.len()
        );
        assert!(units.iter().all(|u| !u.name.is_empty()));
        assert!(units.iter().all(|u| u.pilot_skill == 4));
    }

    #[test]
    fn parses_a_known_unit() {
        let units = available_units();
        let flea = units
            .iter()
            .find(|u| u.name == "Flea FLE-14")
            .expect("Flea FLE-14 present");
        assert_eq!(flea.size, Size::Light);
        assert_eq!(flea.mv_inches, 18.0);
        assert_eq!(flea.jump_inches, 8.0);
        assert_eq!(
            flea.damage,
            DamageValues {
                short: 1,
                medium: 1,
                long: 0
            }
        );
        assert_eq!(flea.armor, 1);
        assert_eq!(flea.structure, 1);
        assert!(flea.ene);
        assert_eq!(flea.pv, 14);
    }
}
