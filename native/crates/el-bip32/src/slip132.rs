//! SLIP-132 version bytes for extended keys.
//!
//! `rust-bitcoin` serializes every extended key with the BIP-32 version
//! bytes, so an account key always reads as `xpub` or `tpub`. SLIP-132
//! registers further version bytes that also name the script type. This
//! module converts between the two, as `bitcoin::bip32` implements neither.

use bitcoin::base58;
use bitcoin::bip32::{Xpriv, Xpub};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Slip132Network {
    Mainnet,
    Testnet,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Slip132Family {
    P2pkh,
    P2shP2wpkh,
    P2wpkh,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Slip132Scope {
    Singlesig,
    Multisig,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct KeyVersion {
    raw: u32,
    prefix: &'static str,
    network: Slip132Network,
    family: Slip132Family,
    scope: Slip132Scope,
    private: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Slip132Error {
    NotBase58,
    UnknownVersion(u32),
    WrongLength(usize),
    WrongKeyKind,
    MalformedKey,
}

impl KeyVersion {
    /// The serialized length of an extended key, without the checksum.
    pub const PAYLOAD_LEN: usize = 78;

    const fn new(
        raw: u32,
        prefix: &'static str,
        network: Slip132Network,
        family: Slip132Family,
        scope: Slip132Scope,
        private: bool,
    ) -> Self {
        Self {
            raw,
            prefix,
            network,
            family,
            scope,
            private,
        }
    }

    /// Every version registered in SLIP-0132 for Bitcoin.
    pub const ALL: [Self; 20] = [
        Self::new(
            0x0488_b21e,
            "xpub",
            Slip132Network::Mainnet,
            Slip132Family::P2pkh,
            Slip132Scope::Singlesig,
            false,
        ),
        Self::new(
            0x0488_ade4,
            "xprv",
            Slip132Network::Mainnet,
            Slip132Family::P2pkh,
            Slip132Scope::Singlesig,
            true,
        ),
        Self::new(
            0x049d_7cb2,
            "ypub",
            Slip132Network::Mainnet,
            Slip132Family::P2shP2wpkh,
            Slip132Scope::Singlesig,
            false,
        ),
        Self::new(
            0x049d_7878,
            "yprv",
            Slip132Network::Mainnet,
            Slip132Family::P2shP2wpkh,
            Slip132Scope::Singlesig,
            true,
        ),
        Self::new(
            0x04b2_4746,
            "zpub",
            Slip132Network::Mainnet,
            Slip132Family::P2wpkh,
            Slip132Scope::Singlesig,
            false,
        ),
        Self::new(
            0x04b2_430c,
            "zprv",
            Slip132Network::Mainnet,
            Slip132Family::P2wpkh,
            Slip132Scope::Singlesig,
            true,
        ),
        Self::new(
            0x0295_b43f,
            "Ypub",
            Slip132Network::Mainnet,
            Slip132Family::P2shP2wpkh,
            Slip132Scope::Multisig,
            false,
        ),
        Self::new(
            0x0295_b005,
            "Yprv",
            Slip132Network::Mainnet,
            Slip132Family::P2shP2wpkh,
            Slip132Scope::Multisig,
            true,
        ),
        Self::new(
            0x02aa_7ed3,
            "Zpub",
            Slip132Network::Mainnet,
            Slip132Family::P2wpkh,
            Slip132Scope::Multisig,
            false,
        ),
        Self::new(
            0x02aa_7a99,
            "Zprv",
            Slip132Network::Mainnet,
            Slip132Family::P2wpkh,
            Slip132Scope::Multisig,
            true,
        ),
        Self::new(
            0x0435_87cf,
            "tpub",
            Slip132Network::Testnet,
            Slip132Family::P2pkh,
            Slip132Scope::Singlesig,
            false,
        ),
        Self::new(
            0x0435_8394,
            "tprv",
            Slip132Network::Testnet,
            Slip132Family::P2pkh,
            Slip132Scope::Singlesig,
            true,
        ),
        Self::new(
            0x044a_5262,
            "upub",
            Slip132Network::Testnet,
            Slip132Family::P2shP2wpkh,
            Slip132Scope::Singlesig,
            false,
        ),
        Self::new(
            0x044a_4e28,
            "uprv",
            Slip132Network::Testnet,
            Slip132Family::P2shP2wpkh,
            Slip132Scope::Singlesig,
            true,
        ),
        Self::new(
            0x045f_1cf6,
            "vpub",
            Slip132Network::Testnet,
            Slip132Family::P2wpkh,
            Slip132Scope::Singlesig,
            false,
        ),
        Self::new(
            0x045f_18bc,
            "vprv",
            Slip132Network::Testnet,
            Slip132Family::P2wpkh,
            Slip132Scope::Singlesig,
            true,
        ),
        Self::new(
            0x0242_89ef,
            "Upub",
            Slip132Network::Testnet,
            Slip132Family::P2shP2wpkh,
            Slip132Scope::Multisig,
            false,
        ),
        Self::new(
            0x0242_85b5,
            "Uprv",
            Slip132Network::Testnet,
            Slip132Family::P2shP2wpkh,
            Slip132Scope::Multisig,
            true,
        ),
        Self::new(
            0x0257_5483,
            "Vpub",
            Slip132Network::Testnet,
            Slip132Family::P2wpkh,
            Slip132Scope::Multisig,
            false,
        ),
        Self::new(
            0x0257_5048,
            "Vprv",
            Slip132Network::Testnet,
            Slip132Family::P2wpkh,
            Slip132Scope::Multisig,
            true,
        ),
    ];

    #[must_use]
    pub const fn raw(self) -> u32 {
        self.raw
    }

    #[must_use]
    pub const fn prefix(self) -> &'static str {
        self.prefix
    }

    #[must_use]
    pub const fn network(self) -> Slip132Network {
        self.network
    }

    #[must_use]
    pub const fn family(self) -> Slip132Family {
        self.family
    }

    #[must_use]
    pub const fn scope(self) -> Slip132Scope {
        self.scope
    }

    #[must_use]
    pub const fn is_private(self) -> bool {
        self.private
    }

    #[must_use]
    pub fn from_raw(raw: u32) -> Option<Self> {
        Self::ALL.into_iter().find(|entry| entry.raw == raw)
    }

    #[must_use]
    pub fn from_prefix(prefix: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|entry| entry.prefix == prefix)
    }

    /// The version bytes `bitcoin::bip32` writes for the same network and
    /// key kind, which is what `Xpub` and `Xpriv` accept.
    #[must_use]
    pub const fn bip32_raw(self) -> u32 {
        match (self.network, self.private) {
            (Slip132Network::Mainnet, false) => 0x0488_b21e,
            (Slip132Network::Mainnet, true) => 0x0488_ade4,
            (Slip132Network::Testnet, false) => 0x0435_87cf,
            (Slip132Network::Testnet, true) => 0x0435_8394,
        }
    }

    fn payload(text: &str) -> Result<Vec<u8>, Slip132Error> {
        let bytes = base58::decode_check(text).map_err(|_| Slip132Error::NotBase58)?;
        if bytes.len() == Self::PAYLOAD_LEN {
            Ok(bytes)
        } else {
            Err(Slip132Error::WrongLength(bytes.len()))
        }
    }

    fn split(text: &str) -> Result<(Self, Vec<u8>), Slip132Error> {
        let mut bytes = Self::payload(text)?;
        let raw = u32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);
        let version = Self::from_raw(raw).ok_or(Slip132Error::UnknownVersion(raw))?;
        bytes[..4].copy_from_slice(&version.bip32_raw().to_be_bytes());
        Ok((version, bytes))
    }

    /// Reads any registered public key form and returns the plain `Xpub`.
    pub fn decode_public(text: &str) -> Result<(Self, Xpub), Slip132Error> {
        let (version, bytes) = Self::split(text)?;
        if version.private {
            return Err(Slip132Error::WrongKeyKind);
        }
        let key = Xpub::decode(&bytes).map_err(|_| Slip132Error::MalformedKey)?;
        Ok((version, key))
    }

    /// Reads any registered private key form and returns the plain `Xpriv`.
    pub fn decode_private(text: &str) -> Result<(Self, Xpriv), Slip132Error> {
        let (version, bytes) = Self::split(text)?;
        if !version.private {
            return Err(Slip132Error::WrongKeyKind);
        }
        let key = Xpriv::decode(&bytes).map_err(|_| Slip132Error::MalformedKey)?;
        Ok((version, key))
    }

    pub fn encode_public(self, key: &Xpub) -> Result<String, Slip132Error> {
        if self.private {
            return Err(Slip132Error::WrongKeyKind);
        }
        let mut bytes = key.encode();
        bytes[..4].copy_from_slice(&self.raw.to_be_bytes());
        Ok(base58::encode_check(&bytes))
    }

    pub fn encode_private(self, key: &Xpriv) -> Result<String, Slip132Error> {
        if !self.private {
            return Err(Slip132Error::WrongKeyKind);
        }
        let mut bytes = key.encode();
        bytes[..4].copy_from_slice(&self.raw.to_be_bytes());
        Ok(base58::encode_check(&bytes))
    }
}

impl core::fmt::Display for Slip132Error {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::NotBase58 => f.write_str("the key is not valid base58check"),
            Self::UnknownVersion(raw) => write!(f, "unknown extended key version {raw:#010x}"),
            Self::WrongLength(len) => write!(
                f,
                "the key payload is {len} bytes, not {}",
                KeyVersion::PAYLOAD_LEN
            ),
            Self::WrongKeyKind => f.write_str("the key is public where a private key is needed"),
            Self::MalformedKey => f.write_str("the key payload is malformed"),
        }
    }
}

impl std::error::Error for Slip132Error {}

#[cfg(test)]
mod tests {
    use super::{KeyVersion, Slip132Error, Slip132Family, Slip132Network, Slip132Scope};

    const XPUB: &str = "xpub6BosfCnifzxcFwrSzQiqu2DBVTshkCXacvNsWGYJVVhhawA7d4R5WSWGFNbi8Aw6ZRc1brxMyWMzG3DSSSSoekkudhUd9yLb6qx39T9nMdj";
    const YPUB: &str = "ypub6Ww3ibxVfGzLrAH1PNcjyAWenMTbbAosGNB6VvmSEgytSER9azLDWCxoJwW7Ke7icmizBMXrzBx9979FfaHxHcrArf3zbeJJJUZPf663zsP";
    const ZPUB: &str = "zpub6rFR7y4Q2AijBEqTUquhVz398htDFrtymD9xYYfG1m4wAcvPhXNfE3EfH1r1ADqtfSdVCToUG868RvUUkgDKf31mGDtKsAYz2oz2AGutZYs";
    const ZPRV: &str = "zprvAdG4iTXWBoARxkkzNpNh8r6Qag3irQB8PzEMkAFeTRXxHpbF9z4QgEvBRmfvqWvGp42t42nvgGpNgYSJA9iefm1yYNZKEm7z6qUWCroSQnE";

    #[test]
    fn every_registered_version_is_distinct() {
        for (offset, entry) in KeyVersion::ALL.iter().enumerate() {
            for other in &KeyVersion::ALL[offset + 1..] {
                assert_ne!(entry.raw(), other.raw());
                assert_ne!(entry.prefix(), other.prefix());
            }
        }
    }

    #[test]
    fn every_version_is_found_by_its_raw_value_and_prefix() {
        for entry in KeyVersion::ALL {
            assert_eq!(KeyVersion::from_raw(entry.raw()), Some(entry));
            assert_eq!(KeyVersion::from_prefix(entry.prefix()), Some(entry));
        }
        assert_eq!(KeyVersion::from_raw(0), None);
        assert_eq!(KeyVersion::from_prefix("qpub"), None);
    }

    /// The decimal constants that `src/js/app.js` carries, so the port and
    /// the page agree on every version.
    #[test]
    fn the_versions_match_the_javascript_table() {
        let expected = [
            ("xpub", 76_067_358),
            ("xprv", 76_066_276),
            ("ypub", 77_429_938),
            ("yprv", 77_428_856),
            ("zpub", 78_792_518),
            ("zprv", 78_791_436),
            ("tpub", 70_617_039),
            ("tprv", 70_615_956),
            ("upub", 71_979_618),
            ("uprv", 71_978_536),
            ("vpub", 73_342_198),
            ("vprv", 73_341_116),
            ("Ypub", 43_365_439),
            ("Yprv", 43_364_357),
            ("Zpub", 44_728_019),
            ("Zprv", 44_726_937),
            ("Upub", 37_915_119),
            ("Uprv", 37_914_037),
            ("Vpub", 39_277_699),
            ("Vprv", 39_276_616),
        ];
        for (prefix, raw) in expected {
            let entry = KeyVersion::from_prefix(prefix).expect("the prefix must be registered");
            assert_eq!(entry.raw(), raw, "{prefix} version");
        }
    }

    /// `vpub` is testnet P2WPKH. The first table in app.js wrongly gave it
    /// to mainnet Taproot before the file reassigned the table.
    #[test]
    fn vpub_is_testnet_native_segwit() {
        let entry = KeyVersion::from_prefix("vpub").unwrap();
        assert_eq!(entry.network(), Slip132Network::Testnet);
        assert_eq!(entry.family(), Slip132Family::P2wpkh);
        assert_eq!(entry.scope(), Slip132Scope::Singlesig);
        assert!(!entry.is_private());
    }

    #[test]
    fn a_serialized_key_keeps_the_bip32_version_of_its_network() {
        for entry in KeyVersion::ALL {
            let expected = match (entry.network(), entry.is_private()) {
                (Slip132Network::Mainnet, false) => 0x0488_b21e,
                (Slip132Network::Mainnet, true) => 0x0488_ade4,
                (Slip132Network::Testnet, false) => 0x0435_87cf,
                (Slip132Network::Testnet, true) => 0x0435_8394,
            };
            assert_eq!(entry.bip32_raw(), expected, "{}", entry.prefix());
        }
    }

    #[test]
    fn the_slip132_test_vectors_decode() {
        for (text, prefix) in [(XPUB, "xpub"), (YPUB, "ypub"), (ZPUB, "zpub")] {
            let (version, _) = KeyVersion::decode_public(text).expect("the vector must decode");
            assert_eq!(version.prefix(), prefix);
        }
    }

    /// Every public form of one key differs only in its version bytes, so
    /// re-encoding a decoded zpub as an xpub must give the spec's xpub for
    /// that same account key.
    #[test]
    fn re_encoding_changes_only_the_prefix() {
        let (_, key) = KeyVersion::decode_public(ZPUB).unwrap();
        let as_xpub = KeyVersion::from_prefix("xpub")
            .unwrap()
            .encode_public(&key)
            .unwrap();
        assert!(as_xpub.starts_with("xpub"));

        let (_, round_tripped) = KeyVersion::decode_public(&as_xpub).unwrap();
        assert_eq!(round_tripped, key);
    }

    #[test]
    fn every_public_version_round_trips() {
        let (_, key) = KeyVersion::decode_public(ZPUB).unwrap();
        for entry in KeyVersion::ALL.into_iter().filter(|e| !e.is_private()) {
            let encoded = entry.encode_public(&key).unwrap();
            assert!(encoded.starts_with(entry.prefix()), "{encoded}");
            let (version, decoded) = KeyVersion::decode_public(&encoded).unwrap();
            assert_eq!(version, entry);
            assert_eq!(decoded.public_key, key.public_key);
            assert_eq!(decoded.chain_code, key.chain_code);
            assert_eq!(decoded.depth, key.depth);
            assert_eq!(decoded.parent_fingerprint, key.parent_fingerprint);
            assert_eq!(decoded.child_number, key.child_number);
        }
    }

    /// The version bytes carry the network, so re-encoding a mainnet key
    /// under a testnet version yields a key that reads as testnet. Only the
    /// network changes; the key material stays the same.
    #[test]
    fn a_cross_network_version_changes_the_network_field() {
        let (_, key) = KeyVersion::decode_public(ZPUB).unwrap();
        assert_eq!(key.network, bitcoin::NetworkKind::Main);

        let as_vpub = KeyVersion::from_prefix("vpub")
            .unwrap()
            .encode_public(&key)
            .unwrap();
        let (_, decoded) = KeyVersion::decode_public(&as_vpub).unwrap();

        assert_eq!(decoded.network, bitcoin::NetworkKind::Test);
        assert_eq!(decoded.public_key, key.public_key);
        assert_eq!(decoded.chain_code, key.chain_code);
    }

    #[test]
    fn a_private_key_round_trips_through_its_own_versions() {
        let (version, key) = KeyVersion::decode_private(ZPRV).unwrap();
        assert_eq!(version.prefix(), "zprv");
        assert_eq!(version.encode_private(&key).unwrap(), ZPRV);
    }

    #[test]
    fn a_public_key_is_refused_where_a_private_key_belongs() {
        assert_eq!(
            KeyVersion::decode_private(ZPUB),
            Err(Slip132Error::WrongKeyKind)
        );
        assert_eq!(
            KeyVersion::decode_public(ZPRV),
            Err(Slip132Error::WrongKeyKind)
        );
    }

    #[test]
    fn a_malformed_key_is_rejected() {
        assert_eq!(
            KeyVersion::decode_public("foobar"),
            Err(Slip132Error::NotBase58)
        );
        assert!(matches!(
            KeyVersion::decode_public("1111111111111111111114oLvT2"),
            Err(Slip132Error::WrongLength(_))
        ));
    }
}
