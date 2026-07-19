//! Unit card stats (rules.md §1 "The Unit Card").
//!
//! Most card fields are exercised through their downstream effects (TMM via the
//! to-hit math, Armor/Structure via damage, Skill via the target number, etc.).
//! The one piece of intrinsic card data with its own numeric rule is **Size**:
//! the SZ weight class is 1 Light / 2 Medium / 3 Heavy / 4 Assault.

use game::unit::Size;

#[test]
fn size_weight_classes_map_to_one_through_four() {
    // rules §1: "Size (SZ): 1 Light, 2 Medium, 3 Heavy, 4 Assault."
    assert_eq!(Size::Light.value(), 1);
    assert_eq!(Size::Medium.value(), 2);
    assert_eq!(Size::Heavy.value(), 3);
    assert_eq!(Size::Assault.value(), 4);
}
