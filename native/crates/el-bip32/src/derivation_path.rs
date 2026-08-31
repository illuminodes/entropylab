//! A BIP-32 derivation path, such as `m/84'/0'/0'`.

use core::fmt;

use crate::child_number::{ChildNumber, ChildNumberError};

#[derive(Clone, Debug, Default, PartialEq, Eq, Hash)]
pub struct DerivationPath {
    steps: Vec<ChildNumber>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DerivationPathError {
    EmptyStep,
    Step(ChildNumberError),
    TooDeep,
}

impl DerivationPath {
    /// BIP-32 gives the depth one byte, so no key can sit deeper than 255.
    pub const MAX_DEPTH: usize = 255;

    #[must_use]
    pub const fn master() -> Self {
        Self { steps: Vec::new() }
    }

    pub fn from_steps(steps: Vec<ChildNumber>) -> Result<Self, DerivationPathError> {
        if steps.len() > Self::MAX_DEPTH {
            return Err(DerivationPathError::TooDeep);
        }
        Ok(Self { steps })
    }

    #[must_use]
    pub fn steps(&self) -> &[ChildNumber] {
        &self.steps
    }

    #[must_use]
    pub const fn depth(&self) -> usize {
        self.steps.len()
    }

    #[must_use]
    pub const fn is_master(&self) -> bool {
        self.steps.is_empty()
    }

    pub fn iter(&self) -> core::slice::Iter<'_, ChildNumber> {
        self.steps.iter()
    }

    pub fn child(&self, step: ChildNumber) -> Result<Self, DerivationPathError> {
        let mut steps = self.steps.clone();
        steps.push(step);
        Self::from_steps(steps)
    }

    #[must_use]
    pub fn parent(&self) -> Option<Self> {
        let (_, head) = self.steps.split_last()?;
        Some(Self {
            steps: head.to_vec(),
        })
    }

    /// The path written with `h` for hardened steps, which is the form the
    /// descriptor exports and the address table use.
    #[must_use]
    pub fn to_display_string(&self) -> String {
        let mut text = String::from("m");
        for step in &self.steps {
            text.push('/');
            text.push_str(&step.index().to_string());
            if step.is_hardened() {
                text.push('h');
            }
        }
        text
    }
}

impl core::str::FromStr for DerivationPath {
    type Err = DerivationPathError;

    /// Accepts an optional `m` prefix and either hardened marker. A trailing
    /// slash is rejected, as it names a step that is not there.
    fn from_str(text: &str) -> Result<Self, Self::Err> {
        let body = text
            .strip_prefix("m/")
            .or_else(|| text.strip_prefix("M/"))
            .unwrap_or(text);

        if body == "m" || body == "M" || body.is_empty() {
            return Ok(Self::master());
        }

        let mut steps = Vec::new();
        for part in body.split('/') {
            if part.is_empty() {
                return Err(DerivationPathError::EmptyStep);
            }
            steps.push(part.parse().map_err(DerivationPathError::Step)?);
        }
        Self::from_steps(steps)
    }
}

/// Writes the apostrophe form, matching `ChildNumber`.
impl fmt::Display for DerivationPath {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("m")?;
        for step in &self.steps {
            write!(f, "/{step}")?;
        }
        Ok(())
    }
}

impl fmt::Display for DerivationPathError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyStep => f.write_str("the path has an empty step"),
            Self::Step(error) => write!(f, "{error}"),
            Self::TooDeep => write!(f, "the path is deeper than {}", DerivationPath::MAX_DEPTH),
        }
    }
}

impl std::error::Error for DerivationPathError {}

impl<'a> IntoIterator for &'a DerivationPath {
    type Item = &'a ChildNumber;
    type IntoIter = core::slice::Iter<'a, ChildNumber>;

    fn into_iter(self) -> Self::IntoIter {
        self.steps.iter()
    }
}

#[cfg(test)]
mod tests {
    use super::{DerivationPath, DerivationPathError};
    use crate::child_number::{ChildNumber, ChildNumberError};

    fn path(text: &str) -> DerivationPath {
        text.parse().expect("the test path must parse")
    }

    #[test]
    fn the_master_path_has_no_steps() {
        for text in ["m", "M", ""] {
            let parsed = path(text);
            assert!(parsed.is_master());
            assert_eq!(parsed.depth(), 0);
            assert_eq!(parsed.to_string(), "m");
        }
    }

    #[test]
    fn the_account_paths_of_every_script_type_parse() {
        for (text, purpose) in [
            ("m/44'/0'/0'", 44),
            ("m/49'/0'/0'", 49),
            ("m/84'/0'/0'", 84),
            ("m/86'/0'/0'", 86),
        ] {
            let parsed = path(text);
            assert_eq!(parsed.depth(), 3);
            assert_eq!(parsed.steps()[0], ChildNumber::Hardened(purpose));
            assert_eq!(parsed.to_string(), text);
        }
    }

    #[test]
    fn the_prefix_is_optional_and_case_insensitive() {
        for text in ["m/84'/0'/0'", "M/84'/0'/0'", "84'/0'/0'"] {
            assert_eq!(path(text), path("m/84'/0'/0'"));
        }
    }

    #[test]
    fn both_hardened_markers_give_the_same_path() {
        assert_eq!(path("m/84h/0h/0h"), path("m/84'/0'/0'"));
    }

    #[test]
    fn display_writes_apostrophes_and_the_display_form_writes_h() {
        let parsed = path("m/84'/0'/0'/0/1");
        assert_eq!(parsed.to_string(), "m/84'/0'/0'/0/1");
        assert_eq!(parsed.to_display_string(), "m/84h/0h/0h/0/1");
    }

    #[test]
    fn the_display_form_round_trips_through_parsing() {
        let parsed = path("m/49'/0'/0'/1/7");
        assert_eq!(path(&parsed.to_display_string()), parsed);
    }

    #[test]
    fn a_mixed_path_keeps_each_step_hardening() {
        let parsed = path("m/84'/0'/0'/0/1");
        let hardening: Vec<bool> = parsed.steps().iter().map(|s| s.is_hardened()).collect();
        assert_eq!(hardening, [true, true, true, false, false]);
    }

    #[test]
    fn child_appends_and_parent_removes_one_step() {
        let account = path("m/84'/0'/0'");
        let receive = account.child(ChildNumber::Normal(0)).unwrap();
        assert_eq!(receive.to_string(), "m/84'/0'/0'/0");
        assert_eq!(receive.parent().unwrap(), account);
        assert_eq!(path("m").parent(), None);
    }

    #[test]
    fn a_malformed_path_is_rejected() {
        assert_eq!(
            "m/84'//0'".parse::<DerivationPath>(),
            Err(DerivationPathError::EmptyStep)
        );
        assert_eq!(
            "m/84'/".parse::<DerivationPath>(),
            Err(DerivationPathError::EmptyStep)
        );
        assert_eq!(
            "m/84'/x".parse::<DerivationPath>(),
            Err(DerivationPathError::Step(ChildNumberError::NotANumber))
        );
        assert_eq!(
            "m/2147483648".parse::<DerivationPath>(),
            Err(DerivationPathError::Step(ChildNumberError::IndexTooLarge))
        );
    }

    #[test]
    fn a_path_deeper_than_the_depth_byte_is_rejected() {
        let steps = vec![ChildNumber::Normal(0); DerivationPath::MAX_DEPTH];
        let deepest = DerivationPath::from_steps(steps).unwrap();
        assert_eq!(deepest.depth(), DerivationPath::MAX_DEPTH);
        assert_eq!(
            deepest.child(ChildNumber::Normal(0)),
            Err(DerivationPathError::TooDeep)
        );
    }

    #[test]
    fn iteration_yields_every_step_in_order() {
        let parsed = path("m/84'/0'/0'");
        let collected: Vec<u32> = parsed.into_iter().map(|s| s.index()).collect();
        assert_eq!(collected, [84, 0, 0]);
    }
}
