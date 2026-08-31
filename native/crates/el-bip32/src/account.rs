//! Account keys derived from a master seed.
//!
//! The wallet derives one account for each script type, at
//! `m/purpose'/coin_type'/account'`. Each account carries its extended keys,
//! its SLIP-132 forms, and the receive and change descriptors.

use bitcoin::bip32::{ChildNumber, DerivationPath, Xpriv, Xpub};
use bitcoin::Network;
use secp256k1::Secp256k1;

use crate::descriptor::{Chain, Descriptor, DescriptorError};
use crate::script_type::ScriptType;
use crate::slip132::{Slip132Error, Slip132Network};

pub struct Account {
    script: ScriptType,
    path: DerivationPath,
    xpub: Xpub,
    xpriv: Option<Xpriv>,
    receive: Descriptor,
    change: Descriptor,
}

#[derive(Debug)]
pub enum AccountError {
    Bip32(bitcoin::bip32::Error),
    Descriptor(DescriptorError),
    Slip132(Slip132Error),
    UncompressedKey,
}

impl Account {
    /// Derives the account for `script` under `master`.
    ///
    /// The coin type follows the network: 0 for mainnet and 1 for every test
    /// network, as SLIP-44 registers them.
    pub fn derive(master: &Xpriv, script: ScriptType, account: u32) -> Result<Self, AccountError> {
        let secp = Secp256k1::new();
        let network = Slip132Network::from(master.network);
        let coin_type = network.coin_type();
        let path = Self::account_path(script, coin_type, account)?;
        let xpriv = master.derive_priv(&secp, &path)?;
        let xpub = Xpub::from_priv(&secp, &xpriv);
        let fingerprint = master.fingerprint(&secp).to_string();
        let key = xpub.to_string();

        let receive = Descriptor::for_account(
            &fingerprint,
            script,
            coin_type,
            account,
            &key,
            Chain::Receive,
        )?;
        let change = Descriptor::for_account(
            &fingerprint,
            script,
            coin_type,
            account,
            &key,
            Chain::Change,
        )?;

        Ok(Self {
            script,
            path,
            xpub,
            xpriv: Some(xpriv),
            receive,
            change,
        })
    }

    fn account_path(
        script: ScriptType,
        coin_type: u32,
        account: u32,
    ) -> Result<DerivationPath, AccountError> {
        Ok(DerivationPath::from(vec![
            ChildNumber::from_hardened_idx(script.purpose())?,
            ChildNumber::from_hardened_idx(coin_type)?,
            ChildNumber::from_hardened_idx(account)?,
        ]))
    }

    #[must_use]
    pub const fn script(&self) -> ScriptType {
        self.script
    }

    #[must_use]
    pub const fn path(&self) -> &DerivationPath {
        &self.path
    }

    /// The account path in the form the wallet shows, with the `m/` prefix
    /// and the `h` marker.
    ///
    /// `DerivationPath` omits the prefix and writes the apostrophe marker
    /// under both format flags. `ChildNumber` writes `h` under the alternate
    /// flag, so this renders each step.
    #[must_use]
    pub fn display_path(&self) -> String {
        let steps: Vec<String> = self
            .path
            .into_iter()
            .map(|child| format!("{child:#}"))
            .collect();
        format!("m/{}", steps.join("/"))
    }

    #[must_use]
    pub const fn xpub(&self) -> &Xpub {
        &self.xpub
    }

    #[must_use]
    pub const fn xpriv(&self) -> Option<&Xpriv> {
        self.xpriv.as_ref()
    }

    /// The account public key under the SLIP-132 prefix of its script.
    pub fn slip132_xpub(&self) -> Result<String, AccountError> {
        let network = Slip132Network::from(self.xpub.network);
        Ok(self
            .script
            .key_version(network, false)
            .encode_public(&self.xpub)?)
    }

    /// The account private key under the SLIP-132 prefix of its script.
    pub fn slip132_xpriv(&self) -> Option<Result<String, AccountError>> {
        let xpriv = self.xpriv.as_ref()?;
        let network = Slip132Network::from(xpriv.network);
        Some(
            self.script
                .key_version(network, true)
                .encode_private(xpriv)
                .map_err(AccountError::from),
        )
    }

    #[must_use]
    pub const fn receive_descriptor(&self) -> &Descriptor {
        &self.receive
    }

    #[must_use]
    pub const fn change_descriptor(&self) -> &Descriptor {
        &self.change
    }

    #[must_use]
    pub const fn descriptor(&self, chain: Chain) -> &Descriptor {
        match chain {
            Chain::Receive => &self.receive,
            Chain::Change => &self.change,
        }
    }
}

impl Slip132Network {
    #[must_use]
    pub const fn coin_type(self) -> u32 {
        match self {
            Self::Mainnet => 0,
            Self::Testnet => 1,
        }
    }
}

impl From<Network> for Slip132Network {
    fn from(network: Network) -> Self {
        match network {
            Network::Bitcoin => Self::Mainnet,
            _ => Self::Testnet,
        }
    }
}

impl From<bitcoin::NetworkKind> for Slip132Network {
    fn from(kind: bitcoin::NetworkKind) -> Self {
        match kind {
            bitcoin::NetworkKind::Main => Self::Mainnet,
            bitcoin::NetworkKind::Test => Self::Testnet,
        }
    }
}

impl From<bitcoin::bip32::Error> for AccountError {
    fn from(error: bitcoin::bip32::Error) -> Self {
        Self::Bip32(error)
    }
}

impl From<DescriptorError> for AccountError {
    fn from(error: DescriptorError) -> Self {
        Self::Descriptor(error)
    }
}

impl From<Slip132Error> for AccountError {
    fn from(error: Slip132Error) -> Self {
        Self::Slip132(error)
    }
}

impl core::fmt::Display for AccountError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Bip32(error) => write!(f, "the key derivation failed: {error}"),
            Self::Descriptor(error) => write!(f, "{error}"),
            Self::Slip132(error) => write!(f, "{error}"),
            Self::UncompressedKey => f.write_str("an address needs a compressed public key"),
        }
    }
}

impl std::error::Error for AccountError {}

#[cfg(test)]
mod tests {
    use super::Account;
    use crate::script_type::ScriptType;
    use bitcoin::bip32::Xpriv;
    use bitcoin::Network;

    const PHRASE: &str =
        "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";

    fn master(network: Network) -> Xpriv {
        let mnemonic =
            bip39::Mnemonic::parse_in_normalized(bip39::Language::English, PHRASE).unwrap();
        Xpriv::new_master(network, &mnemonic.to_seed_normalized("")).unwrap()
    }

    /// The BIP-84 test vectors publish the account key for this phrase.
    #[test]
    fn the_bip84_account_key_matches_the_published_vector() {
        let account = Account::derive(&master(Network::Bitcoin), ScriptType::P2wpkh, 0).unwrap();
        assert_eq!(
            account.slip132_xpub().unwrap(),
            "zpub6rFR7y4Q2AijBEqTUquhVz398htDFrtymD9xYYfG1m4wAcvPhXNfE3EfH1r1ADqtfSdVCToUG868RvUUkgDKf31mGDtKsAYz2oz2AGutZYs"
        );
        assert_eq!(
            account.slip132_xpriv().unwrap().unwrap(),
            "zprvAdG4iTXWBoARxkkzNpNh8r6Qag3irQB8PzEMkAFeTRXxHpbF9z4QgEvBRmfvqWvGp42t42nvgGpNgYSJA9iefm1yYNZKEm7z6qUWCroSQnE"
        );
    }

    /// BIP-49 publishes its account key on testnet, in the SLIP-132 upub
    /// and uprv forms.
    #[test]
    fn the_bip49_testnet_account_key_matches_the_published_vector() {
        let account =
            Account::derive(&master(Network::Testnet), ScriptType::P2shP2wpkh, 0).unwrap();
        assert_eq!(
            account.slip132_xpub().unwrap(),
            "upub5EFU65HtV5TeiSHmZZm7FUffBGy8UKeqp7vw43jYbvZPpoVsgU93oac7Wk3u6moKegAEWtGNF8DehrnHtv21XXEMYRUocHqguyjknFHYfgY"
        );
        assert_eq!(
            account.slip132_xpriv().unwrap().unwrap(),
            "uprv91G7gZkzehuMVxDJTYE6tLivdF8e4rvzSu1LFfKw3b2Qx1Aj8vpoFnHdfUZ3hmi9jsvPifmZ24RTN2KhwB8BfMLTVqaBReibyaFFcTP1s9n"
        );
    }

    /// BIP-49 publishes the testnet master key for this phrase.
    #[test]
    fn the_bip49_testnet_master_key_matches_the_published_vector() {
        let version = crate::slip132::KeyVersion::ALL
            .into_iter()
            .find(|entry| entry.prefix() == "uprv")
            .unwrap();
        assert_eq!(
            version.encode_private(&master(Network::Testnet)).unwrap(),
            "uprv8tXDerPXZ1QsVNjUJWTurs9kA1KGfKUAts74GCkcXtU8GwnH33GDRbNJpEqTvipfCyycARtQJhmdfWf8oKt41X9LL1zeD2pLsWmxEk3VAwd"
        );
    }

    /// BIP-44 publishes the account key for this phrase as an xpub.
    #[test]
    fn the_bip44_account_key_matches_the_published_vector() {
        let account = Account::derive(&master(Network::Bitcoin), ScriptType::P2pkh, 0).unwrap();
        assert_eq!(
            account.xpub().to_string(),
            "xpub6BosfCnifzxcFwrSzQiqu2DBVTshkCXacvNsWGYJVVhhawA7d4R5WSWGFNbi8Aw6ZRc1brxMyWMzG3DSSSSoekkudhUd9yLb6qx39T9nMdj"
        );
    }

    #[test]
    fn the_path_follows_the_script_and_network() {
        for (script, purpose) in [
            (ScriptType::P2pkh, 44),
            (ScriptType::P2shP2wpkh, 49),
            (ScriptType::P2wpkh, 84),
            (ScriptType::P2tr, 86),
        ] {
            let mainnet = Account::derive(&master(Network::Bitcoin), script, 0).unwrap();
            assert_eq!(mainnet.display_path(), format!("m/{purpose}h/0h/0h"));

            let testnet = Account::derive(&master(Network::Testnet), script, 0).unwrap();
            assert_eq!(testnet.display_path(), format!("m/{purpose}h/1h/0h"));
        }
    }

    #[test]
    fn the_account_index_reaches_the_derived_key() {
        let first = Account::derive(&master(Network::Bitcoin), ScriptType::P2wpkh, 0).unwrap();
        let second = Account::derive(&master(Network::Bitcoin), ScriptType::P2wpkh, 1).unwrap();
        assert_eq!(second.display_path(), "m/84h/0h/1h");
        assert_ne!(first.xpub(), second.xpub());
    }

    /// Taproot has no SLIP-132 prefix, so BIP-86 exports the xpub form.
    #[test]
    fn the_taproot_account_exports_an_xpub() {
        let account = Account::derive(&master(Network::Bitcoin), ScriptType::P2tr, 0).unwrap();
        assert_eq!(account.slip132_xpub().unwrap(), account.xpub().to_string());
        assert!(account.slip132_xpub().unwrap().starts_with("xpub"));
    }

    #[test]
    fn the_descriptors_carry_the_account_origin() {
        let account = Account::derive(&master(Network::Bitcoin), ScriptType::P2wpkh, 0).unwrap();
        let fingerprint = "73c5da0a";
        let receive = account.receive_descriptor().to_string();
        assert!(
            receive.starts_with(&format!("wpkh([{fingerprint}/84h/0h/0h]")),
            "{receive}"
        );
        assert!(receive.contains("/0/*)"), "{receive}");
        assert!(account.change_descriptor().to_string().contains("/1/*)"));
    }

    #[test]
    fn every_script_derives_on_both_networks() {
        for script in ScriptType::ALL {
            for network in [Network::Bitcoin, Network::Testnet] {
                let account = Account::derive(&master(network), script, 0).unwrap();
                assert!(account.xpriv().is_some());
                assert!(account.slip132_xpriv().unwrap().is_ok());
                assert!(account.slip132_xpub().is_ok());
            }
        }
    }
}
