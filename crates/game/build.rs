//! Build-time preparation of the unit catalog.
//!
//! The master Alpha Strike unit list ships as a ~1.9 MB CSV. Embedding it raw
//! with `include_str!` bloated the WASM bundle by that whole amount and forced a
//! full CSV parse on every startup. Instead we parse it here, once, at build
//! time; keep only the BattleMech rows and the fields the game actually uses;
//! serialize the result to JSON; and DEFLATE-compress it. At runtime
//! `catalog::available_units` just inflates + deserializes the (much smaller)
//! embedded blob — no CSV text and no parser ship in the binary.
//!
//! The CSV parser is shared verbatim with the crate via `include!` of
//! `src/catalog_csv.rs`, so the build-time parsing and the parsing exercised by
//! the crate's own unit tests cannot drift apart.

use std::path::Path;

include!("src/catalog_csv.rs");

/// The master unit list, relative to the crate manifest directory.
const CSV_FILE: &str = "../../data/unit_list_20230906-001811.csv";

fn main() {
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR");
    let csv_path = Path::new(&manifest_dir).join(CSV_FILE);

    println!("cargo:rerun-if-changed={}", csv_path.display());
    println!("cargo:rerun-if-changed=src/catalog_csv.rs");
    println!("cargo:rerun-if-changed=build.rs");

    let csv = std::fs::read_to_string(&csv_path)
        .unwrap_or_else(|e| panic!("failed to read {}: {e}", csv_path.display()));

    let records = parse_records(&csv);
    let json = serde_json::to_vec(&records).expect("serialize catalog to JSON");
    // Raw DEFLATE (level 10, max). `catalog::available_units` inflates this with
    // the matching `miniz_oxide::inflate::decompress_to_vec`.
    let compressed = miniz_oxide::deflate::compress_to_vec(&json, 10);

    let out_dir = std::env::var("OUT_DIR").expect("OUT_DIR");
    let out_path = Path::new(&out_dir).join("catalog.deflate");
    std::fs::write(&out_path, &compressed)
        .unwrap_or_else(|e| panic!("failed to write {}: {e}", out_path.display()));
}
