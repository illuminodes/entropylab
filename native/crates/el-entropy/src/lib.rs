//! Entropy input transforms for `EntropyLab`.
//!
//! This crate converts user-supplied transcripts into BIP39 entropy. It
//! generates no randomness of its own.

pub mod dice;
pub mod seed_length;

pub use dice::{DiceError, DiceMethod, DiceTranscript};
pub use seed_length::SeedLength;
