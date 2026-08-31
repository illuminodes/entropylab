//! The four single-signature script types the wallet derives.

use crate::slip132::{KeyVersion, Slip132Family, Slip132Network};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ScriptType {
    P2pkh,
    P2shP2wpkh,
    P2wpkh,
    P2tr,
}

impl ScriptType {
    pub const ALL: [Self; 4] = [Self::P2pkh, Self::P2shP2wpkh, Self::P2wpkh, Self::P2tr];

    #[must_use]
    pub const fn bip(self) -> &'static str {
        match self {
            Self::P2pkh => "BIP44",
            Self::P2shP2wpkh => "BIP49",
            Self::P2wpkh => "BIP84",
            Self::P2tr => "BIP86",
        }
    }

    #[must_use]
    pub const fn purpose(self) -> u32 {
        match self {
            Self::P2pkh => 44,
            Self::P2shP2wpkh => 49,
            Self::P2wpkh => 84,
            Self::P2tr => 86,
        }
    }

    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::P2pkh => "Legacy",
            Self::P2shP2wpkh => "Nested SegWit",
            Self::P2wpkh => "Native SegWit",
            Self::P2tr => "Taproot",
        }
    }

    /// Wraps a key expression in the descriptor function of this script.
    #[must_use]
    pub fn descriptor_body(self, key: &str) -> String {
        match self {
            Self::P2pkh => format!("pkh({key})"),
            Self::P2shP2wpkh => format!("sh(wpkh({key}))"),
            Self::P2wpkh => format!("wpkh({key})"),
            Self::P2tr => format!("tr({key})"),
        }
    }

    /// The SLIP-132 family this script advertises. Taproot has no registered
    /// prefix, so BIP-86 keys stay in the `xpub` family.
    #[must_use]
    pub const fn slip132_family(self) -> Slip132Family {
        match self {
            Self::P2pkh | Self::P2tr => Slip132Family::P2pkh,
            Self::P2shP2wpkh => Slip132Family::P2shP2wpkh,
            Self::P2wpkh => Slip132Family::P2wpkh,
        }
    }

    #[must_use]
    pub fn key_version(self, network: Slip132Network, private: bool) -> KeyVersion {
        let family = self.slip132_family();
        KeyVersion::ALL
            .into_iter()
            .find(|entry| {
                entry.network() == network
                    && entry.family() == family
                    && entry.is_private() == private
                    && entry.scope() == crate::slip132::Slip132Scope::Singlesig
            })
            .expect("every singlesig family is registered for both networks")
    }
}

#[cfg(test)]
mod tests {
    use super::ScriptType;
    use crate::slip132::{Slip132Family, Slip132Network};

    #[test]
    fn the_purposes_match_their_bips() {
        for (script, purpose, bip) in [
            (ScriptType::P2pkh, 44, "BIP44"),
            (ScriptType::P2shP2wpkh, 49, "BIP49"),
            (ScriptType::P2wpkh, 84, "BIP84"),
            (ScriptType::P2tr, 86, "BIP86"),
        ] {
            assert_eq!(script.purpose(), purpose);
            assert_eq!(script.bip(), bip);
        }
    }

    #[test]
    fn the_descriptor_bodies_match_the_javascript_forms() {
        assert_eq!(ScriptType::P2pkh.descriptor_body("KEY"), "pkh(KEY)");
        assert_eq!(
            ScriptType::P2shP2wpkh.descriptor_body("KEY"),
            "sh(wpkh(KEY))"
        );
        assert_eq!(ScriptType::P2wpkh.descriptor_body("KEY"), "wpkh(KEY)");
        assert_eq!(ScriptType::P2tr.descriptor_body("KEY"), "tr(KEY)");
    }

    /// Taproot has no SLIP-132 prefix of its own, so BIP-86 exports an
    /// xpub. The first table in app.js gave it `v`, which the file later
    /// reassigned to `x`.
    #[test]
    fn taproot_stays_in_the_xpub_family() {
        assert_eq!(ScriptType::P2tr.slip132_family(), Slip132Family::P2pkh);
        let version = ScriptType::P2tr.key_version(Slip132Network::Mainnet, false);
        assert_eq!(version.prefix(), "xpub");
    }

    #[test]
    fn every_script_maps_to_the_expected_prefix() {
        for (script, mainnet_pub, testnet_pub) in [
            (ScriptType::P2pkh, "xpub", "tpub"),
            (ScriptType::P2shP2wpkh, "ypub", "upub"),
            (ScriptType::P2wpkh, "zpub", "vpub"),
            (ScriptType::P2tr, "xpub", "tpub"),
        ] {
            assert_eq!(
                script.key_version(Slip132Network::Mainnet, false).prefix(),
                mainnet_pub
            );
            assert_eq!(
                script.key_version(Slip132Network::Testnet, false).prefix(),
                testnet_pub
            );
        }
    }

    #[test]
    fn every_script_has_a_private_version_on_both_networks() {
        for script in ScriptType::ALL {
            for network in [Slip132Network::Mainnet, Slip132Network::Testnet] {
                let private = script.key_version(network, true);
                let public = script.key_version(network, false);
                assert!(private.is_private());
                assert!(!public.is_private());
                assert_eq!(private.network(), network);
                assert_eq!(private.family(), public.family());
            }
        }
    }
}
