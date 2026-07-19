//! Dice abstraction.
//!
//! Combat resolution depends only on the [`Dice`] trait, so production code uses
//! a seedable PRNG ([`XorShiftDice`]) while tests inject a deterministic
//! [`ScriptedDice`]. Neither implementation touches the browser; the UI seeds
//! the PRNG from `Math::random()` at start-up.

use std::collections::VecDeque;

/// Anything that can produce a 2D6 roll (a value in `2..=12`).
pub trait Dice {
    fn roll_2d6(&mut self) -> u8;
}

/// A tiny, dependency-free xorshift64 PRNG. Deterministic for a given seed,
/// which keeps the crate free of `rand`/`getrandom` and their wasm quirks.
#[derive(Clone, Debug)]
pub struct XorShiftDice {
    state: u64,
}

impl XorShiftDice {
    pub fn new(seed: u64) -> Self {
        // Avoid the all-zero state, which xorshift cannot escape.
        Self {
            state: if seed == 0 { 0x9E37_79B9_7F4A_7C15 } else { seed },
        }
    }

    fn next_u64(&mut self) -> u64 {
        let mut x = self.state;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.state = x;
        x
    }

    fn d6(&mut self) -> u8 {
        (self.next_u64() % 6) as u8 + 1
    }
}

impl Dice for XorShiftDice {
    fn roll_2d6(&mut self) -> u8 {
        self.d6() + self.d6()
    }
}

/// A dice source that returns a predetermined sequence of 2D6 totals. Used in
/// tests to force specific hit/miss outcomes.
#[derive(Clone, Debug)]
pub struct ScriptedDice {
    rolls: VecDeque<u8>,
}

impl ScriptedDice {
    pub fn new(rolls: impl IntoIterator<Item = u8>) -> Self {
        Self {
            rolls: rolls.into_iter().collect(),
        }
    }
}

impl Dice for ScriptedDice {
    fn roll_2d6(&mut self) -> u8 {
        self.rolls
            .pop_front()
            .expect("ScriptedDice ran out of scripted rolls")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn xorshift_rolls_are_in_2d6_range() {
        let mut dice = XorShiftDice::new(12345);
        for _ in 0..10_000 {
            let r = dice.roll_2d6();
            assert!((2..=12).contains(&r), "roll {r} out of range");
        }
    }

    #[test]
    fn xorshift_is_deterministic_per_seed() {
        let mut a = XorShiftDice::new(42);
        let mut b = XorShiftDice::new(42);
        let seq_a: Vec<u8> = (0..20).map(|_| a.roll_2d6()).collect();
        let seq_b: Vec<u8> = (0..20).map(|_| b.roll_2d6()).collect();
        assert_eq!(seq_a, seq_b);
    }

    #[test]
    fn xorshift_seeds_differ() {
        let mut a = XorShiftDice::new(1);
        let mut b = XorShiftDice::new(2);
        let seq_a: Vec<u8> = (0..20).map(|_| a.roll_2d6()).collect();
        let seq_b: Vec<u8> = (0..20).map(|_| b.roll_2d6()).collect();
        assert_ne!(seq_a, seq_b);
    }

    #[test]
    fn scripted_dice_returns_queue_in_order() {
        let mut dice = ScriptedDice::new([7, 2, 12]);
        assert_eq!(dice.roll_2d6(), 7);
        assert_eq!(dice.roll_2d6(), 2);
        assert_eq!(dice.roll_2d6(), 12);
    }
}
