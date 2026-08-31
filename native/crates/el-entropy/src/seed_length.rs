//! The five BIP39 phrase lengths and the constants each one implies.
//!
//! Ported from `hodlSeedLengths` in `src/js/app.js`.

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum SeedLength {
    Words12,
    Words15,
    Words18,
    Words21,
    #[default]
    Words24,
}

impl SeedLength {
    pub const ALL: [Self; 5] = [
        Self::Words12,
        Self::Words15,
        Self::Words18,
        Self::Words21,
        Self::Words24,
    ];

    #[must_use]
    pub const fn from_words(words: u16) -> Option<Self> {
        match words {
            12 => Some(Self::Words12),
            15 => Some(Self::Words15),
            18 => Some(Self::Words18),
            21 => Some(Self::Words21),
            24 => Some(Self::Words24),
            _ => None,
        }
    }

    #[must_use]
    pub const fn words(self) -> u16 {
        match self {
            Self::Words12 => 12,
            Self::Words15 => 15,
            Self::Words18 => 18,
            Self::Words21 => 21,
            Self::Words24 => 24,
        }
    }

    #[must_use]
    pub const fn bits(self) -> usize {
        match self {
            Self::Words12 => 128,
            Self::Words15 => 160,
            Self::Words18 => 192,
            Self::Words21 => 224,
            Self::Words24 => 256,
        }
    }

    #[must_use]
    pub const fn bytes(self) -> usize {
        self.bits() / 8
    }

    #[must_use]
    pub const fn hex_chars(self) -> usize {
        self.bits() / 4
    }

    /// The published fair-die roll recommendation, `round(bits / log2(6))`.
    ///
    /// Every length takes `ceil(bits / log2(6))` except `Words24`, which
    /// ships 99 where the ceiling is 100: 99 rolls carry ~255.91 bits, so
    /// the recommendation sits 0.09 bits under the 256-bit target. Upstream
    /// ships the same 99 and asserts only that 98 rolls fall short, so this
    /// port keeps the value rather than silently strengthening it.
    #[must_use]
    pub const fn hash_rolls(self) -> usize {
        match self {
            Self::Words12 => 50,
            Self::Words15 => 62,
            Self::Words18 => 75,
            Self::Words21 => 87,
            Self::Words24 => 99,
        }
    }

    #[must_use]
    pub const fn partial_words(self) -> u16 {
        self.words() - 1
    }

    #[must_use]
    pub const fn candidates(self) -> u16 {
        match self {
            Self::Words12 => 128,
            Self::Words15 => 64,
            Self::Words18 => 32,
            Self::Words21 => 16,
            Self::Words24 => 8,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::SeedLength;

    fn roll_bits(count: usize) -> f64 {
        f64::from(u32::try_from(count).expect("roll count fits u32")) * 6f64.log2()
    }

    fn target_bits(length: SeedLength) -> f64 {
        f64::from(u32::try_from(length.bits()).expect("bit width fits u32"))
    }

    /// The least roll count whose entropy reaches the target, found by
    /// integer search so no float drives the loop.
    fn ceiling_rolls(length: SeedLength) -> usize {
        (1..=256)
            .find(|&rolls| roll_bits(rolls) >= target_bits(length))
            .expect("256 rolls exceed every supported target")
    }

    #[test]
    fn one_roll_below_the_recommendation_always_falls_short() {
        for length in SeedLength::ALL {
            assert!(roll_bits(length.hash_rolls() - 1) < target_bits(length));
        }
    }

    #[test]
    fn recommendation_is_the_ceiling_roll_count_except_for_24_words() {
        for length in SeedLength::ALL {
            let ceiling = ceiling_rolls(length);
            let expected = if length == SeedLength::Words24 {
                ceiling - 1
            } else {
                ceiling
            };
            assert_eq!(length.hash_rolls(), expected);
        }
    }

    #[test]
    fn only_the_24_word_recommendation_rounds_below_its_target() {
        for length in SeedLength::ALL {
            let reaches = roll_bits(length.hash_rolls()) >= target_bits(length);
            assert_eq!(reaches, length != SeedLength::Words24);
        }
    }

    #[test]
    fn published_roll_counts_match_upstream() {
        let expected = [50, 62, 75, 87, 99];
        for (length, want) in SeedLength::ALL.into_iter().zip(expected) {
            assert_eq!(length.hash_rolls(), want);
        }
    }

    #[test]
    fn byte_and_hex_widths_follow_the_bit_length() {
        for length in SeedLength::ALL {
            assert_eq!(length.bytes() * 8, length.bits());
            assert_eq!(length.hex_chars(), length.bytes() * 2);
        }
    }

    #[test]
    fn word_counts_round_trip_through_from_words() {
        for length in SeedLength::ALL {
            assert_eq!(SeedLength::from_words(length.words()), Some(length));
        }
        assert_eq!(SeedLength::from_words(13), None);
    }
}
