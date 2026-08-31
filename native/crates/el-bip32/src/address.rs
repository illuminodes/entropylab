//! Addresses derived under an account key.
//!
//! Each row is one address at `account_path/chain/index`, with its public
//! key and, when the account holds private material, its WIF.

use bitcoin::key::{CompressedPublicKey, PrivateKey, UntweakedPublicKey};
use bitcoin::secp256k1::Secp256k1;
use bitcoin::{Address, Network, NetworkKind};

use crate::account::AccountError;
use crate::descriptor::Chain;
use crate::script_type::ScriptType;

pub struct AddressRow {
    index: u32,
    chain: Chain,
    path: String,
    address: Address,
    public_key: bitcoin::PublicKey,
    wif: Option<String>,
}

impl ScriptType {
    /// Builds the address for `key` under this script.
    ///
    /// Taproot applies the BIP-86 key path tweak, which commits to an empty
    /// script tree. An untweaked output key would be unspendable.
    pub fn address(
        self,
        key: &bitcoin::PublicKey,
        network: Network,
    ) -> Result<Address, AccountError> {
        let compressed =
            CompressedPublicKey::try_from(*key).map_err(|_| AccountError::UncompressedKey)?;
        Ok(match self {
            Self::P2pkh => Address::p2pkh(compressed, network),
            Self::P2shP2wpkh => Address::p2shwpkh(&compressed, network),
            Self::P2wpkh => Address::p2wpkh(&compressed, network),
            Self::P2tr => {
                let secp = Secp256k1::verification_only();
                let internal = UntweakedPublicKey::from(key.inner);
                Address::p2tr(&secp, internal, None, network)
            }
        })
    }
}

impl AddressRow {
    #[must_use]
    pub const fn index(&self) -> u32 {
        self.index
    }

    #[must_use]
    pub const fn chain(&self) -> Chain {
        self.chain
    }

    #[must_use]
    pub fn path(&self) -> &str {
        &self.path
    }

    #[must_use]
    pub const fn address(&self) -> &Address {
        &self.address
    }

    #[must_use]
    pub const fn public_key(&self) -> &bitcoin::PublicKey {
        &self.public_key
    }

    #[must_use]
    pub fn wif(&self) -> Option<&str> {
        self.wif.as_deref()
    }
}

impl crate::account::Account {
    /// Derives `count` addresses on `chain`, starting at `start`.
    pub fn addresses(
        &self,
        chain: Chain,
        start: u32,
        count: u32,
    ) -> Result<Vec<AddressRow>, AccountError> {
        (start..start.saturating_add(count))
            .map(|index| self.address_row(chain, index))
            .collect()
    }

    /// Derives one address at `chain/index` under the account key.
    pub fn address_row(&self, chain: Chain, index: u32) -> Result<AddressRow, AccountError> {
        use bitcoin::bip32::ChildNumber;

        let secp = Secp256k1::new();
        let steps = [
            ChildNumber::from_normal_idx(chain.index())?,
            ChildNumber::from_normal_idx(index)?,
        ];
        let network = self.xpub().network;

        let (public_key, wif) = match self.xpriv() {
            Some(xpriv) => {
                let child = xpriv.derive_priv(&secp, &steps)?;
                let private = PrivateKey::new(child.private_key, network);
                (private.public_key(&secp), Some(private.to_wif()))
            }
            None => (self.xpub().derive_pub(&secp, &steps)?.to_pub().into(), None),
        };

        let address = self
            .script()
            .address(&public_key, Self::address_network(network))?;

        Ok(AddressRow {
            index,
            chain,
            path: format!("{}/{}/{index}", self.display_path(), chain.index()),
            address,
            public_key,
            wif,
        })
    }

    const fn address_network(kind: NetworkKind) -> Network {
        match kind {
            NetworkKind::Main => Network::Bitcoin,
            NetworkKind::Test => Network::Testnet,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::account::Account;
    use crate::descriptor::Chain;
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

    fn account(script: ScriptType, network: Network) -> Account {
        Account::derive(&master(network), script, 0).unwrap()
    }

    /// BIP-84 publishes the first two receive addresses and the first
    /// change address for this phrase.
    #[test]
    fn the_bip84_addresses_match_the_published_vectors() {
        let account = account(ScriptType::P2wpkh, Network::Bitcoin);
        let receive = account.addresses(Chain::Receive, 0, 2).unwrap();
        assert_eq!(
            receive[0].address().to_string(),
            "bc1qcr8te4kr609gcawutmrza0j4xv80jy8z306fyu"
        );
        assert_eq!(
            receive[1].address().to_string(),
            "bc1qnjg0jd8228aq7egyzacy8cys3knf9xvrerkf9g"
        );
        let change = account.addresses(Chain::Change, 0, 1).unwrap();
        assert_eq!(
            change[0].address().to_string(),
            "bc1q8c6fshw2dlwun7ekn9qwf37cu2rn755upcp6el"
        );
    }

    /// BIP-86 publishes the first two Taproot receive addresses. They are
    /// correct only with the key path tweak applied.
    #[test]
    fn the_bip86_addresses_match_the_published_vectors() {
        let account = account(ScriptType::P2tr, Network::Bitcoin);
        let receive = account.addresses(Chain::Receive, 0, 2).unwrap();
        assert_eq!(
            receive[0].address().to_string(),
            "bc1p5cyxnuxmeuwuvkwfem96lqzszd02n6xdcjrs20cac6yqjjwudpxqkedrcr"
        );
        assert_eq!(
            receive[1].address().to_string(),
            "bc1p4qhjn9zdvkux4e44uhx8tc55attvtyu358kutcqkudyccelu0was9fqzwh"
        );
        let change = account.addresses(Chain::Change, 0, 1).unwrap();
        assert_eq!(
            change[0].address().to_string(),
            "bc1p3qkhfews2uk44qtvauqyr2ttdsw7svhkl9nkm9s9c3x4ax5h60wqwruhk7"
        );
    }

    /// BIP-341 requires the key path tweak. An untweaked output key gives a
    /// different address, and the coins would be unspendable.
    #[test]
    fn the_taproot_output_key_is_tweaked() {
        use bitcoin::key::TapTweak;
        use bitcoin::secp256k1::Secp256k1;

        let account = account(ScriptType::P2tr, Network::Bitcoin);
        let row = &account.addresses(Chain::Receive, 0, 1).unwrap()[0];

        let internal = bitcoin::key::UntweakedPublicKey::from(row.public_key().inner);
        assert_eq!(
            internal.to_string(),
            "cc8a4bc64d897bddc5fbc2f670f7a8ba0b386779106cf1223c6fc5d7cd6fc115"
        );

        let (output, _) = internal.tap_tweak(&Secp256k1::new(), None);
        assert_eq!(
            output.to_x_only_public_key().to_string(),
            "a60869f0dbcf1dc659c9cecbaf8050135ea9e8cdc487053f1dc6880949dc684c"
        );
        assert_ne!(
            internal.to_string(),
            output.to_x_only_public_key().to_string()
        );
        assert!(row.address().to_string().contains("5cyxnuxmeuwuvkwfem96"));
    }

    /// BIP-49 publishes the first testnet nested segwit address and its WIF.
    #[test]
    fn the_bip49_address_and_wif_match_the_published_vector() {
        let account = account(ScriptType::P2shP2wpkh, Network::Testnet);
        let row = &account.addresses(Chain::Receive, 0, 1).unwrap()[0];
        assert_eq!(
            row.address().to_string(),
            "2Mww8dCYPUpKHofjgcXcBCEGmniw9CoaiD2"
        );
        assert_eq!(
            row.wif().unwrap(),
            "cULrpoZGXiuC19Uhvykx7NugygA3k86b3hmdCeyvHYQZSxojGyXJ"
        );
    }

    /// BIP-44 publishes the first legacy address for this phrase.
    #[test]
    fn the_bip44_address_matches_the_published_vector() {
        let account = account(ScriptType::P2pkh, Network::Bitcoin);
        let row = &account.addresses(Chain::Receive, 0, 1).unwrap()[0];
        assert_eq!(
            row.address().to_string(),
            "1LqBGSKuX5yYUonjxT5qGfpUsXKYYWeabA"
        );
    }

    #[test]
    fn every_script_starts_with_its_expected_prefix() {
        for (script, prefix) in [
            (ScriptType::P2pkh, "1"),
            (ScriptType::P2shP2wpkh, "3"),
            (ScriptType::P2wpkh, "bc1q"),
            (ScriptType::P2tr, "bc1p"),
        ] {
            let account = account(script, Network::Bitcoin);
            let row = &account.addresses(Chain::Receive, 0, 1).unwrap()[0];
            assert!(
                row.address().to_string().starts_with(prefix),
                "{} is not a {prefix} address",
                row.address()
            );
        }
    }

    #[test]
    fn the_path_records_the_chain_and_index() {
        let account = account(ScriptType::P2wpkh, Network::Bitcoin);
        let receive = &account.addresses(Chain::Receive, 0, 1).unwrap()[0];
        assert_eq!(receive.path(), "m/84h/0h/0h/0/0");
        let change = &account.addresses(Chain::Change, 5, 1).unwrap()[0];
        assert_eq!(change.path(), "m/84h/0h/0h/1/5");
        assert_eq!(change.index(), 5);
    }

    #[test]
    fn the_start_index_offsets_the_rows() {
        let account = account(ScriptType::P2wpkh, Network::Bitcoin);
        let all = account.addresses(Chain::Receive, 0, 5).unwrap();
        let tail = account.addresses(Chain::Receive, 3, 2).unwrap();
        assert_eq!(tail.len(), 2);
        assert_eq!(all[3].address(), tail[0].address());
        assert_eq!(all[4].address(), tail[1].address());
    }

    #[test]
    fn the_receive_and_change_chains_differ() {
        let account = account(ScriptType::P2wpkh, Network::Bitcoin);
        let receive = account.addresses(Chain::Receive, 0, 1).unwrap();
        let change = account.addresses(Chain::Change, 0, 1).unwrap();
        assert_ne!(receive[0].address(), change[0].address());
    }

    /// A watch-only account has no private material, so it has no WIF.
    #[test]
    fn every_row_carries_a_wif_when_the_account_holds_private_keys() {
        for script in ScriptType::ALL {
            let account = account(script, Network::Bitcoin);
            for row in account.addresses(Chain::Receive, 0, 3).unwrap() {
                assert!(row.wif().is_some());
                assert!(row.public_key().compressed);
            }
        }
    }

    /// Every derived address must parse back on its own network.
    #[test]
    fn every_address_is_valid_for_its_network() {
        for script in ScriptType::ALL {
            for network in [Network::Bitcoin, Network::Testnet] {
                let account = account(script, network);
                for row in account.addresses(Chain::Receive, 0, 2).unwrap() {
                    let parsed = row
                        .address()
                        .to_string()
                        .parse::<bitcoin::Address<_>>()
                        .expect("the address parses");
                    assert!(parsed.is_valid_for_network(network));
                }
            }
        }
    }
}
