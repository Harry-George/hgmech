// CSV parsing for the master unit list.
//
// This module is deliberately self-contained: it depends only on `std` and
// `serde_json`, with NO `crate::` references, so it can be `include!`d by the
// build script (`build.rs`) as well as compiled as a crate module for its own
// unit tests. (Regular `//` comments, not `//!` — the build script `include!`s
// this file mid-body, where inner doc comments are not allowed.) The build
// script runs `parse_records` over the embedded CSV once, at build time, and
// stores the compressed result; the crate never parses the CSV at runtime (see
// `crate::catalog`).
//
// Records are produced as `serde_json::Value` objects whose keys match the
// fields of `crate::unit::UnitCard`, so the runtime can deserialize the stored
// JSON straight into `Vec<UnitCard>` without this module needing to know about
// the `UnitCard` type at all.

use serde_json::{json, Value};

// Column indices in the CSV export.
const COL_NAME: usize = 0;
const COL_ROLE: usize = 2;
const COL_PV: usize = 3;
const COL_TYPE: usize = 4;
const COL_SIZE: usize = 5;
const COL_MOVE: usize = 6;
const COL_SHORT: usize = 7;
const COL_MEDIUM: usize = 8;
const COL_LONG: usize = 9;
const COL_OVERHEAT: usize = 10;
const COL_ARMOR: usize = 11;
const COL_STRUCTURE: usize = 12;
const COL_SPECIALS: usize = 13;
const COL_IMAGE: usize = 14;

/// Split raw CSV text into records of string fields. Handles RFC-4180-style
/// quoting: commas and newlines inside `"…"` are literal, and a doubled `""`
/// inside a quoted field is one literal quote.
fn parse_csv(text: &str) -> Vec<Vec<String>> {
    let mut records = Vec::new();
    let mut record = Vec::new();
    let mut field = String::new();
    let mut in_quotes = false;
    let mut chars = text.chars().peekable();

    while let Some(c) = chars.next() {
        if in_quotes {
            match c {
                '"' if chars.peek() == Some(&'"') => {
                    chars.next();
                    field.push('"');
                }
                '"' => in_quotes = false,
                _ => field.push(c),
            }
        } else {
            match c {
                '"' => in_quotes = true,
                ',' => record.push(std::mem::take(&mut field)),
                '\r' => {} // paired with the following '\n'
                '\n' => {
                    record.push(std::mem::take(&mut field));
                    records.push(std::mem::take(&mut record));
                }
                _ => field.push(c),
            }
        }
    }
    // Flush a final record that has no trailing newline.
    if !field.is_empty() || !record.is_empty() {
        record.push(field);
        records.push(record);
    }
    records
}

/// Parse the leading unsigned integer of a cell, ignoring any trailing
/// decoration such as the `*` in a `0*` (minimal-damage) value.
fn parse_u32(cell: &str) -> u32 {
    cell.trim()
        .chars()
        .take_while(|c| c.is_ascii_digit())
        .collect::<String>()
        .parse()
        .unwrap_or(0)
}

/// Parse a Move cell — e.g. `40"`, `18"/8"j`, `14"j` — into (ground, jump)
/// inches. The `j`-suffixed token is the jump value; the plain token is the
/// ground value. A lone jump token (`14"j`) also sets the ground rate.
fn parse_move(cell: &str) -> (f64, f64) {
    let mut ground = 0.0;
    let mut jump = 0.0;
    for token in cell.split('/') {
        let token = token.trim();
        if token.is_empty() {
            continue;
        }
        let is_jump = token.to_ascii_lowercase().contains('j');
        let value: f64 = token
            .chars()
            .filter(|c| c.is_ascii_digit() || *c == '.')
            .collect::<String>()
            .parse()
            .unwrap_or(0.0);
        if is_jump {
            jump = value;
        } else {
            ground = value;
        }
    }
    if ground == 0.0 && jump > 0.0 {
        ground = jump;
    }
    (ground, jump)
}

/// Map the numeric Size code (1–4) to the [`crate::unit::Size`] variant name.
/// Returned as the string serde uses for the enum, so it deserializes directly.
fn parse_size(cell: &str) -> &'static str {
    match cell.trim() {
        "1" => "Light",
        "3" => "Heavy",
        "4" => "Assault",
        _ => "Medium",
    }
}

/// A reasonable default TMM for a unit's MV, used to populate card stats. Mirrors
/// [`crate::combat::tmm_for_mv`]; duplicated here to keep this module free of
/// `crate::` dependencies so the build script can `include!` it.
fn tmm_for_mv(mv_inches: f64) -> i32 {
    match mv_inches as i32 {
        i32::MIN..=2 => 0,
        3..=6 => 1,
        7..=10 => 2,
        11..=14 => 3,
        15..=18 => 4,
        _ => 5,
    }
}

/// True if `specials` contains `name` as a whole comma-separated token.
fn has_special(specials: &str, name: &str) -> bool {
    specials
        .split(',')
        .any(|t| t.trim().eq_ignore_ascii_case(name))
}

/// Parse the master list into one JSON object per pickable unit, filtered to
/// BattleMechs. Each object's keys match the fields of
/// [`crate::unit::UnitCard`], with a default pilot skill of 4 (Regular); the
/// force builder adjusts skill per unit later.
pub fn parse_records(csv: &str) -> Vec<Value> {
    let mut records = parse_csv(csv);
    if records.is_empty() {
        return Vec::new();
    }
    records.remove(0); // header

    let mut units = Vec::new();
    for row in records {
        if row.len() <= COL_IMAGE {
            continue;
        }
        let name = row[COL_NAME].trim();
        if name.is_empty() || !row[COL_TYPE].trim().eq_ignore_ascii_case("BM") {
            continue;
        }

        let specials = row[COL_SPECIALS].trim();
        let (mv_inches, jump_inches) = parse_move(&row[COL_MOVE]);

        units.push(json!({
            "name": name,
            "size": parse_size(&row[COL_SIZE]),
            "mv_inches": mv_inches,
            "tmm": tmm_for_mv(mv_inches),
            "damage": {
                "short": parse_u32(&row[COL_SHORT]),
                "medium": parse_u32(&row[COL_MEDIUM]),
                "long": parse_u32(&row[COL_LONG]),
            },
            "armor": parse_u32(&row[COL_ARMOR]),
            "structure": parse_u32(&row[COL_STRUCTURE]),
            "pilot_skill": 4,
            "overheat": parse_u32(&row[COL_OVERHEAT]),
            "ovl": has_special(specials, "OVL"),
            "ene": has_special(specials, "ENE"),
            "case": has_special(specials, "CASE"),
            "caseii": has_special(specials, "CASEII"),
            "mel": has_special(specials, "MEL"),
            "pv": parse_u32(&row[COL_PV]),
            "role": row[COL_ROLE].trim(),
            "jump_inches": jump_inches,
            "specials": specials,
            "image_url": row[COL_IMAGE].trim(),
        }));
    }
    units
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_csv_handles_quotes_and_escapes() {
        let text = "a,b,c\n\"x,y\",\"he said \"\"hi\"\"\",z\n";
        let recs = parse_csv(text);
        assert_eq!(recs.len(), 2);
        assert_eq!(recs[1], vec!["x,y", "he said \"hi\"", "z"]);
    }

    #[test]
    fn parse_move_handles_jump_notation() {
        assert_eq!(parse_move("40\""), (40.0, 0.0));
        assert_eq!(parse_move("18\"/8\"j"), (18.0, 8.0));
        assert_eq!(parse_move("14\"j"), (14.0, 14.0));
    }

    #[test]
    fn parse_records_maps_fields_and_filters_to_battlemechs() {
        // Header + one BM row + one non-BM row. Only the BM survives.
        let csv = "\
Name,x,Role,PV,Type,Size,Move,S,M,L,OV,Armor,Struct,Specials,Image
Flea FLE-14,,Scout,14,BM,1,18\"/8\"j,1,1,0,0,1,1,\"ENE, REAR\",http://img
Some Tank,,Brawler,20,CV,2,10\",2,2,0,0,4,3,,";
        let recs = parse_records(csv);
        assert_eq!(recs.len(), 1);
        let flea = &recs[0];
        assert_eq!(flea["name"], "Flea FLE-14");
        assert_eq!(flea["size"], "Light");
        assert_eq!(flea["mv_inches"], 18.0);
        assert_eq!(flea["jump_inches"], 8.0);
        assert_eq!(flea["tmm"], 4);
        assert_eq!(flea["damage"]["short"], 1);
        assert_eq!(flea["pv"], 14);
        assert_eq!(flea["ene"], true);
        assert_eq!(flea["ovl"], false);
    }
}
