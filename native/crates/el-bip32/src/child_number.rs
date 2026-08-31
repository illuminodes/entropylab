//! One step of a BIP-32 derivation path.

use core::fmt;

pub const HARDENED_OFFSET: u32 = 0x8000_0000;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ChildNumber {
    Normal(u32),
    Hardened(u32),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ChildNumberError {
    Empty,
    IndexTooLarge,
    NotANumber,
}

impl ChildNumber {
    pub const MAX_INDEX: u32 = HARDENED_OFFSET - 1;

    pub const fn normal(index: u32) -> Result<Self, ChildNumberError> {
        if index > Self::MAX_INDEX {
            return Err(ChildNumberError::IndexTooLarge);
        }
        Ok(Self::Normal(index))
    }

    pub const fn hardened(index: u32) -> Result<Self, ChildNumberError> {
        if index > Self::MAX_INDEX {
            return Err(ChildNumberError::IndexTooLarge);
        }
        Ok(Self::Hardened(index))
    }

    #[must_use]
    pub const fn from_raw(raw: u32) -> Self {
        if raw >= HARDENED_OFFSET {
            Self::Hardened(raw - HARDENED_OFFSET)
        } else {
            Self::Normal(raw)
        }
    }

    #[must_use]
    pub const fn raw(self) -> u32 {
        match self {
            Self::Normal(index) => index,
            Self::Hardened(index) => index + HARDENED_OFFSET,
        }
    }

    #[must_use]
    pub const fn index(self) -> u32 {
        match self {
            Self::Normal(index) | Self::Hardened(index) => index,
        }
    }

    #[must_use]
    pub const fn is_hardened(self) -> bool {
        matches!(self, Self::Hardened(_))
    }
}

impl core::str::FromStr for ChildNumber {
    type Err = ChildNumberError;

    /// Accepts `0`, `0'`, and `0h`. BIP-32 writes the apostrophe, and
    /// descriptors write `h`, so both mark a hardened step.
    fn from_str(text: &str) -> Result<Self, Self::Err> {
        let (digits, hardened) = text
            .strip_suffix(['\'', 'h', 'H'])
            .map_or((text, false), |rest| (rest, true));

        if digits.is_empty() {
            return Err(ChildNumberError::Empty);
        }
        if !digits.bytes().all(|b| b.is_ascii_digit()) {
            return Err(ChildNumberError::NotANumber);
        }

        let index: u32 = digits
            .parse()
            .map_err(|_| ChildNumberError::IndexTooLarge)?;

        if hardened {
            Self::hardened(index)
        } else {
            Self::normal(index)
        }
    }
}

/// Writes the apostrophe form, which is what BIP-32 itself uses.
impl fmt::Display for ChildNumber {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Normal(index) => write!(f, "{index}"),
            Self::Hardened(index) => write!(f, "{index}'"),
        }
    }
}

impl fmt::Display for ChildNumberError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let text = match self {
            Self::Empty => "the path step is empty",
            Self::IndexTooLarge => "the child index must be below 2^31",
            Self::NotANumber => "the path step is not a number",
        };
        f.write_str(text)
    }
}

impl std::error::Error for ChildNumberError {}

#[cfg(test)]
mod tests {
    use super::{ChildNumber, ChildNumberError, HARDENED_OFFSET};

    #[test]
    fn hardened_and_normal_split_at_the_offset() {
        assert_eq!(ChildNumber::from_raw(0), ChildNumber::Normal(0));
        assert_eq!(
            ChildNumber::from_raw(HARDENED_OFFSET),
            ChildNumber::Hardened(0)
        );
        assert_eq!(
            ChildNumber::from_raw(HARDENED_OFFSET - 1),
            ChildNumber::Normal(HARDENED_OFFSET - 1)
        );
        assert_eq!(
            ChildNumber::from_raw(u32::MAX),
            ChildNumber::Hardened(ChildNumber::MAX_INDEX)
        );
    }

    #[test]
    fn raw_round_trips_across_the_whole_range() {
        for raw in [0, 1, 44, HARDENED_OFFSET - 1, HARDENED_OFFSET, u32::MAX] {
            assert_eq!(ChildNumber::from_raw(raw).raw(), raw);
        }
    }

    #[test]
    fn an_index_at_or_above_the_offset_is_rejected() {
        assert_eq!(
            ChildNumber::normal(HARDENED_OFFSET),
            Err(ChildNumberError::IndexTooLarge)
        );
        assert_eq!(
            ChildNumber::hardened(HARDENED_OFFSET),
            Err(ChildNumberError::IndexTooLarge)
        );
        assert!(ChildNumber::normal(ChildNumber::MAX_INDEX).is_ok());
    }

    #[test]
    fn both_hardened_markers_parse_the_same() {
        for text in ["44'", "44h", "44H"] {
            assert_eq!(
                text.parse::<ChildNumber>().unwrap(),
                ChildNumber::Hardened(44)
            );
        }
        assert_eq!(
            "44".parse::<ChildNumber>().unwrap(),
            ChildNumber::Normal(44)
        );
    }

    #[test]
    fn display_uses_the_apostrophe_form() {
        assert_eq!(ChildNumber::Hardened(0).to_string(), "0'");
        assert_eq!(ChildNumber::Normal(0).to_string(), "0");
    }

    #[test]
    fn display_round_trips_through_parsing() {
        for raw in [0, 1, 84, HARDENED_OFFSET, u32::MAX] {
            let child = ChildNumber::from_raw(raw);
            assert_eq!(child.to_string().parse::<ChildNumber>().unwrap(), child);
        }
    }

    #[test]
    fn malformed_steps_are_rejected() {
        for text in ["", "'", "h", "-1", "1x", "+1", " 1", "1 ", "٤٤"] {
            assert!(
                text.parse::<ChildNumber>().is_err(),
                "{text:?} must not parse"
            );
        }
    }

    #[test]
    fn an_index_above_the_u32_range_is_rejected() {
        assert_eq!(
            "4294967296".parse::<ChildNumber>(),
            Err(ChildNumberError::IndexTooLarge)
        );
    }
}
