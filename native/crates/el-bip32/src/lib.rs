//! BIP-32 key derivation for `EntropyLab`.
//!
//! The BIP-32 primitives come from `bitcoin::bip32`. Use `Xpub`, `Xpriv`,
//! `DerivationPath`, and `ChildNumber` from there directly. The descriptor
//! checksum comes from `miniscript`. This crate adds only what those leave
//! out.
//!
//! `bitcoin::bip32::DerivationPath` parses both hardened markers, writes the
//! apostrophe form through `Display`, and writes the `h` form through the
//! alternate flag, `{:#}`. It omits the leading `m/`.

pub mod account;
pub mod descriptor;
pub mod script_type;
pub mod slip132;

pub use account::{Account, AccountError};
pub use descriptor::{Chain, Descriptor, DescriptorError};
pub use script_type::ScriptType;
pub use slip132::{KeyVersion, Slip132Error, Slip132Family, Slip132Network, Slip132Scope};

#[cfg(test)]
mod upstream_probe {
    use bitcoin::AddressType;
    use std::str::FromStr;

    /// `DerivationPath` writes the apostrophe marker under both the plain
    /// and the alternate flag, so the `h` form needs its own rendering.
    #[test]
    fn upstream_derivation_path_writes_the_apostrophe_marker() {
        let path = bitcoin::bip32::DerivationPath::from_str("m/84h/0h/0h").unwrap();
        assert_eq!(format!("{path}"), "84'/0'/0'");
        assert_eq!(format!("{path:#}"), "84'/0'/0'");

        let child = bitcoin::bip32::ChildNumber::from_hardened_idx(84).unwrap();
        assert_eq!(format!("{child}"), "84'");
        assert_eq!(format!("{child:#}"), "84h");
    }

    /// `AddressType` cannot name a nested segwit account: BIP-49 and a bare
    /// P2SH both read as `P2sh`. `ScriptType` carries that distinction.
    #[test]
    fn upstream_address_type_has_no_nested_segwit_variant() {
        assert_eq!(AddressType::from_str("p2pkh").unwrap(), AddressType::P2pkh);
        assert_eq!(AddressType::from_str("p2sh").unwrap(), AddressType::P2sh);
        assert_eq!(
            AddressType::from_str("p2wpkh").unwrap(),
            AddressType::P2wpkh
        );
        assert_eq!(AddressType::from_str("p2tr").unwrap(), AddressType::P2tr);
        assert!(AddressType::from_str("p2sh-p2wpkh").is_err());
    }
}
