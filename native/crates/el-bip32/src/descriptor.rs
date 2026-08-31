//! Output descriptors for an account key.
//!
//! The checksum comes from `miniscript`, which implements BIP-380. This
//! module builds the descriptor text, because the wallet writes hardened
//! steps in an origin with `h` and `miniscript` rewrites them to `'` when
//! it renders a parsed descriptor. The checksum covers the text, so the two
//! marker forms give different checksums.

use miniscript::descriptor::checksum::Engine;

use crate::script_type::ScriptType;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Chain {
    Receive,
    Change,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Descriptor {
    body: String,
    checksum: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DescriptorError {
    InvalidCharacter,
}

impl Chain {
    pub const ALL: [Self; 2] = [Self::Receive, Self::Change];

    #[must_use]
    pub const fn index(self) -> u32 {
        match self {
            Self::Receive => 0,
            Self::Change => 1,
        }
    }

    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Receive => "receive",
            Self::Change => "change",
        }
    }
}

impl Descriptor {
    /// Builds the key expression `[fingerprint/purpose/coin/account]KEY/chain/*`.
    ///
    /// The origin writes hardened steps with `h`, which is the form the
    /// wallet exports.
    #[must_use]
    pub fn key_expression(
        fingerprint: &str,
        script: ScriptType,
        coin_type: u32,
        account: u32,
        key: &str,
        chain: Chain,
    ) -> String {
        format!(
            "[{fingerprint}/{}h/{coin_type}h/{account}h]{key}/{}/*",
            script.purpose(),
            chain.index()
        )
    }

    pub fn new(body: String) -> Result<Self, DescriptorError> {
        let mut engine = Engine::new();
        engine
            .input(&body)
            .map_err(|_| DescriptorError::InvalidCharacter)?;
        let checksum = engine.checksum();
        Ok(Self { body, checksum })
    }

    pub fn for_account(
        fingerprint: &str,
        script: ScriptType,
        coin_type: u32,
        account: u32,
        key: &str,
        chain: Chain,
    ) -> Result<Self, DescriptorError> {
        let expression = Self::key_expression(fingerprint, script, coin_type, account, key, chain);
        Self::new(script.descriptor_body(&expression))
    }

    #[must_use]
    pub fn body(&self) -> &str {
        &self.body
    }

    #[must_use]
    pub fn checksum(&self) -> &str {
        &self.checksum
    }
}

impl core::fmt::Display for Descriptor {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}#{}", self.body, self.checksum)
    }
}

impl core::fmt::Display for DescriptorError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str("the descriptor has a character that BIP-380 does not allow")
    }
}

impl std::error::Error for DescriptorError {}

#[cfg(test)]
mod tests {
    use super::{Chain, Descriptor, DescriptorError};
    use crate::script_type::ScriptType;

    const XPUB: &str = "xpub6BosfCnifzxcFwrSzQiqu2DBVTshkCXacvNsWGYJVVhhawA7d4R5WSWGFNbi8Aw6ZRc1brxMyWMzG3DSSSSoekkudhUd9yLb6qx39T9nMdj";
    const FINGERPRINT: &str = "73c5da0a";

    fn descriptor(script: ScriptType, chain: Chain) -> Descriptor {
        Descriptor::for_account(FINGERPRINT, script, 0, 0, XPUB, chain)
            .expect("the descriptor must build")
    }

    /// Bitcoin Core accepts a descriptor only when the checksum matches, so
    /// `miniscript` must agree with the checksum Core computes.
    #[test]
    fn the_checksum_matches_bitcoin_core() {
        let built = Descriptor::new(format!("wpkh({XPUB}/0/*)")).unwrap();
        let parsed: miniscript::Descriptor<miniscript::DescriptorPublicKey> =
            built.body().parse().expect("the body parses");
        let rendered = parsed.to_string();
        let expected = rendered.split('#').nth(1).expect("a rendered checksum");
        assert_eq!(built.checksum(), expected);
    }

    #[test]
    fn the_origin_uses_the_h_marker() {
        let text = descriptor(ScriptType::P2wpkh, Chain::Receive).to_string();
        assert!(text.contains("[73c5da0a/84h/0h/0h]"), "{text}");
        assert!(!text.contains('\''), "{text}");
    }

    /// The apostrophe form is a different string, so BIP-380 gives it a
    /// different checksum. The wallet must export the `h` form to stay
    /// consistent with the page.
    #[test]
    fn the_marker_form_changes_the_checksum() {
        let with_h = Descriptor::new(format!("wpkh([{FINGERPRINT}/84h/0h/0h]{XPUB}/0/*)")).unwrap();
        let with_apostrophe =
            Descriptor::new(format!("wpkh([{FINGERPRINT}/84'/0'/0']{XPUB}/0/*)")).unwrap();
        assert_ne!(with_h.checksum(), with_apostrophe.checksum());
    }

    #[test]
    fn every_script_wraps_the_key_expression() {
        for (script, opening) in [
            (ScriptType::P2pkh, "pkh("),
            (ScriptType::P2shP2wpkh, "sh(wpkh("),
            (ScriptType::P2wpkh, "wpkh("),
            (ScriptType::P2tr, "tr("),
        ] {
            let text = descriptor(script, Chain::Receive).to_string();
            assert!(text.starts_with(opening), "{text}");
        }
    }

    #[test]
    fn the_purpose_follows_the_script_type() {
        for (script, purpose) in [
            (ScriptType::P2pkh, "44h"),
            (ScriptType::P2shP2wpkh, "49h"),
            (ScriptType::P2wpkh, "84h"),
            (ScriptType::P2tr, "86h"),
        ] {
            let text = descriptor(script, Chain::Receive).to_string();
            assert!(text.contains(&format!("/{purpose}/0h/0h]")), "{text}");
        }
    }

    #[test]
    fn the_chain_step_separates_receive_from_change() {
        let receive = descriptor(ScriptType::P2wpkh, Chain::Receive).to_string();
        let change = descriptor(ScriptType::P2wpkh, Chain::Change).to_string();
        assert!(receive.contains("/0/*)"), "{receive}");
        assert!(change.contains("/1/*)"), "{change}");
        assert_ne!(receive, change);
    }

    #[test]
    fn every_built_descriptor_parses_back() {
        for script in ScriptType::ALL {
            for chain in Chain::ALL {
                let built = descriptor(script, chain);
                let parsed = built
                    .body()
                    .parse::<miniscript::Descriptor<miniscript::DescriptorPublicKey>>();
                assert!(parsed.is_ok(), "{built}");
            }
        }
    }

    #[test]
    fn the_display_form_appends_the_checksum() {
        let built = descriptor(ScriptType::P2wpkh, Chain::Receive);
        assert_eq!(
            built.to_string(),
            format!("{}#{}", built.body(), built.checksum())
        );
        assert_eq!(built.checksum().len(), 8);
    }

    #[test]
    fn a_character_outside_the_alphabet_is_rejected() {
        assert_eq!(
            Descriptor::new("wpkh(\u{1f600})".to_owned()),
            Err(DescriptorError::InvalidCharacter)
        );
    }
}
