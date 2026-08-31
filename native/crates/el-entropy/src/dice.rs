//! Hashed-dice entropy: SHA-256 over a transcript of six-sided rolls.
//!
//! Ported from `hodlDiceEntropy` / `hodlIanColemanDiceString` in
//! `src/js/app.js`. This module never generates a roll; it only transforms a
//! transcript the user supplies.

use bitcoin_hashes::{sha256, Hash};

use crate::seed_length::SeedLength;

/// Which published transcript convention to hash.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DiceMethod {
    /// COLDCARD / `SeedSigner`: hash the original digits, faces `1..=6`.
    Coldcard,
    /// Ian Coleman / Keystone: map face `6` to `0`, then hash.
    Coleman,
}

impl DiceMethod {
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Coldcard => "coldcard-sha256",
            Self::Coleman => "ian-coleman-dice-sha256",
        }
    }

    fn transcript(self, rolls: &[u8]) -> String {
        rolls
            .iter()
            .map(|face| match (self, face) {
                (Self::Coleman, 6) => '0',
                _ => char::from(b'0' + face),
            })
            .collect()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DiceError {
    Empty,
    UnexpectedCharacter(char),
}

/// A validated transcript of fair six-sided die rolls.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DiceTranscript {
    rolls: Vec<u8>,
}

impl DiceTranscript {
    /// Parses `input`, skipping the separators the UI allows (whitespace,
    /// `,`, `;`, `|`) and rejecting any other character.
    pub fn parse(input: &str) -> Result<Self, DiceError> {
        let mut rolls = Vec::new();
        for character in input.chars() {
            if character.is_whitespace() || matches!(character, ',' | ';' | '|') {
                continue;
            }
            match character {
                '1'..='6' => rolls.push(character as u8 - b'0'),
                other => return Err(DiceError::UnexpectedCharacter(other)),
            }
        }
        if rolls.is_empty() {
            return Err(DiceError::Empty);
        }
        Ok(Self { rolls })
    }

    #[must_use]
    pub const fn len(&self) -> usize {
        self.rolls.len()
    }

    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.rolls.is_empty()
    }

    #[must_use]
    pub fn rolls(&self) -> &[u8] {
        &self.rolls
    }

    /// Shannon entropy of the transcript, assuming fair independent rolls.
    #[must_use]
    pub fn source_bits(&self) -> f64 {
        let rolls = u32::try_from(self.rolls.len()).unwrap_or(u32::MAX);
        f64::from(rolls) * 6f64.log2()
    }

    /// True when the transcript is shorter than the published recommendation
    /// for `length`. The derivation stays deterministic either way.
    #[must_use]
    pub const fn is_below_recommendation(&self, length: SeedLength) -> bool {
        self.rolls.len() < length.hash_rolls()
    }

    /// Hashes the whole transcript and truncates to the entropy width of
    /// `length`. Every roll is hashed, including any beyond the
    /// recommendation.
    #[must_use]
    pub fn entropy(&self, method: DiceMethod, length: SeedLength) -> Vec<u8> {
        let digest = sha256::Hash::hash(method.transcript(&self.rolls).as_bytes());
        digest.as_byte_array()[..length.bytes()].to_vec()
    }
}

#[cfg(test)]
mod tests {
    use super::{DiceError, DiceMethod, DiceTranscript};
    use crate::seed_length::SeedLength;

    fn transcript(len: usize) -> DiceTranscript {
        let text: String = (0..len)
            .map(|i| char::from(b'1' + u8::try_from(i % 6).expect("modulo six fits u8")))
            .collect();
        DiceTranscript::parse(&text).expect("valid transcript")
    }

    #[test]
    fn separators_are_skipped_and_faces_kept_in_order() {
        let parsed = DiceTranscript::parse("1 2,3;4|5\n6").expect("valid");
        assert_eq!(parsed.rolls(), &[1, 2, 3, 4, 5, 6]);
    }

    #[test]
    fn out_of_range_faces_are_rejected() {
        assert_eq!(
            DiceTranscript::parse("123407"),
            Err(DiceError::UnexpectedCharacter('0'))
        );
        assert_eq!(DiceTranscript::parse("   "), Err(DiceError::Empty));
    }

    #[test]
    fn coleman_maps_six_to_zero_and_coldcard_does_not() {
        let parsed = DiceTranscript::parse("16").expect("valid");
        assert_eq!(DiceMethod::Coleman.transcript(parsed.rolls()), "10");
        assert_eq!(DiceMethod::Coldcard.transcript(parsed.rolls()), "16");
    }

    #[test]
    fn entropy_width_matches_the_requested_seed_length() {
        let parsed = transcript(99);
        for length in SeedLength::ALL {
            for method in [DiceMethod::Coldcard, DiceMethod::Coleman] {
                assert_eq!(parsed.entropy(method, length).len(), length.bytes());
            }
        }
    }

    #[test]
    fn shorter_lengths_are_prefixes_of_longer_ones() {
        let parsed = transcript(99);
        let widest = parsed.entropy(DiceMethod::Coldcard, SeedLength::Words24);
        for length in SeedLength::ALL {
            let entropy = parsed.entropy(DiceMethod::Coldcard, length);
            assert_eq!(entropy, widest[..length.bytes()]);
        }
    }

    #[test]
    fn the_two_methods_disagree_when_a_six_is_present() {
        let parsed = DiceTranscript::parse("666666").expect("valid");
        assert_ne!(
            parsed.entropy(DiceMethod::Coldcard, SeedLength::Words24),
            parsed.entropy(DiceMethod::Coleman, SeedLength::Words24)
        );
    }

    #[test]
    fn derivation_is_deterministic() {
        let parsed = transcript(99);
        assert_eq!(
            parsed.entropy(DiceMethod::Coleman, SeedLength::Words12),
            parsed.entropy(DiceMethod::Coleman, SeedLength::Words12)
        );
    }

    #[test]
    fn recommendation_flag_follows_the_published_roll_count() {
        for length in SeedLength::ALL {
            assert!(transcript(length.hash_rolls() - 1).is_below_recommendation(length));
            assert!(!transcript(length.hash_rolls()).is_below_recommendation(length));
        }
    }

    #[test]
    fn known_vector_matches_sha256_of_the_digit_string() {
        let parsed = DiceTranscript::parse("123456").expect("valid");
        let entropy = parsed.entropy(DiceMethod::Coldcard, SeedLength::Words24);
        assert_eq!(
            entropy,
            hex_bytes("8d969eef6ecad3c29a3a629280e686cf0c3f5d5a86aff3ca12020c923adc6c92")
        );
    }

    fn hex_bytes(text: &str) -> Vec<u8> {
        (0..text.len())
            .step_by(2)
            .map(|i| u8::from_str_radix(&text[i..i + 2], 16).expect("hex"))
            .collect()
    }
}
