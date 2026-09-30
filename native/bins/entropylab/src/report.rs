//! The wallet content the window shows, built without any Wayland or pixels.
//!
//! Keeping the text model separate from the painter means the wallet output
//! is testable without a compositor.

use el_bip32::{Account, AccountError, Chain, ScriptType};

/// One labelled read-only value, drawn as a field in the account card.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Field {
    label: String,
    value: String,
    note: Option<String>,
}

/// One row of the address table: the `#`, `Path`, and `Address` columns of
/// the web app's table.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AddressRow {
    index: u32,
    path: String,
    address: String,
}

pub struct Report {
    script: ScriptType,
    fingerprint: String,
    fields: Vec<Field>,
    rows: Vec<AddressRow>,
}

impl Field {
    #[must_use]
    pub fn label(&self) -> &str {
        &self.label
    }

    #[must_use]
    pub fn value(&self) -> &str {
        &self.value
    }

    #[must_use]
    pub fn note(&self) -> Option<&str> {
        self.note.as_deref()
    }
}

impl AddressRow {
    #[must_use]
    pub const fn index(&self) -> u32 {
        self.index
    }

    #[must_use]
    pub fn path(&self) -> &str {
        &self.path
    }

    #[must_use]
    pub fn address(&self) -> &str {
        &self.address
    }
}

impl Report {
    pub const ADDRESS_COUNT: u32 = 8;

    /// Builds the report for one account.
    pub fn for_account(account: &Account) -> Result<Self, AccountError> {
        let script = account.script();
        let account_path = account.display_path();
        let descriptor = account.receive_descriptor().to_string();
        let fields = vec![
            Field {
                label: "Script type".to_owned(),
                value: script.label().to_owned(),
                note: Some(format!("{} · {}", script.bip(), script.short())),
            },
            Field {
                label: "Derivation path".to_owned(),
                value: format!("{account_path}/0/0"),
                note: Some("Exact BIP32 address path".to_owned()),
            },
            Field {
                label: format!("Account extended public key ({})", account.slip132_prefix()),
                value: account.slip132_xpub()?,
                note: None,
            },
            Field {
                label: "Receive descriptor".to_owned(),
                value: descriptor,
                note: Some("Import into Bitcoin Core as watch-only".to_owned()),
            },
        ];

        let rows = account
            .addresses(Chain::Receive, 0, Self::ADDRESS_COUNT)?
            .into_iter()
            .map(|row| AddressRow {
                index: row.index(),
                path: row.path().to_string(),
                address: row.address().to_string(),
            })
            .collect();

        Ok(Self {
            script,
            fingerprint: account.fingerprint().to_owned(),
            fields,
            rows,
        })
    }

    #[must_use]
    pub const fn script(&self) -> ScriptType {
        self.script
    }

    #[must_use]
    pub fn fingerprint(&self) -> &str {
        &self.fingerprint
    }

    #[must_use]
    pub fn fields(&self) -> &[Field] {
        &self.fields
    }

    #[must_use]
    pub fn rows(&self) -> &[AddressRow] {
        &self.rows
    }

    /// The plain-text form, used by `--print`.
    #[must_use]
    pub fn plain_lines(&self) -> Vec<String> {
        let mut lines = vec![self.script.label().to_owned()];
        for field in &self.fields {
            lines.push(field.label.clone());
            lines.push(field.value.clone());
        }
        lines.push("addresses".to_owned());
        for row in &self.rows {
            lines.push(format!("{}  {}  {}", row.index, row.path, row.address));
        }
        lines
    }
}

#[cfg(test)]
mod tests {
    use super::Report;
    use bitcoin::bip32::Xpriv;
    use bitcoin::Network;
    use el_bip32::{Account, ScriptType};

    const PHRASE: &str =
        "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";

    fn account(script: ScriptType) -> Account {
        let mnemonic =
            bip39::Mnemonic::parse_in_normalized(bip39::Language::English, PHRASE).unwrap();
        let master = Xpriv::new_master(Network::Bitcoin, &mnemonic.to_seed_normalized("")).unwrap();
        Account::derive(&master, script, 0).unwrap()
    }

    fn report(script: ScriptType) -> Report {
        Report::for_account(&account(script)).unwrap()
    }

    #[test]
    fn the_report_shows_the_published_bip84_account_key() {
        let report = report(ScriptType::P2wpkh);
        let path = report
            .fields()
            .iter()
            .find(|field| field.label() == "Derivation path")
            .expect("the derivation path field");
        assert_eq!(path.value(), "m/84h/0h/0h/0/0");
        let key = report
            .fields()
            .iter()
            .find(|field| field.label().starts_with("Account extended"))
            .expect("the account key field");
        assert_eq!(
            key.value(),
            "zpub6rFR7y4Q2AijBEqTUquhVz398htDFrtymD9xYYfG1m4wAcvPhXNfE3EfH1r1ADqtfSdVCToUG868RvUUkgDKf31mGDtKsAYz2oz2AGutZYs"
        );
    }

    #[test]
    fn the_table_rows_carry_the_index_path_and_address() {
        let report = report(ScriptType::P2wpkh);
        assert_eq!(report.rows().len(), Report::ADDRESS_COUNT as usize);
        let first = &report.rows()[0];
        assert_eq!(first.index(), 0);
        assert_eq!(
            first.address(),
            "bc1qcr8te4kr609gcawutmrza0j4xv80jy8z306fyu"
        );
        assert!(first.path().ends_with("/0/0"));
        assert_eq!(report.rows()[7].index(), 7);
    }

    #[test]
    fn the_fingerprint_is_the_published_master_fingerprint() {
        assert_eq!(report(ScriptType::P2wpkh).fingerprint(), "73c5da0a");
    }

    #[test]
    fn every_script_builds_a_report() {
        for script in ScriptType::ALL {
            let report = report(script);
            assert_eq!(report.script(), script);
            assert!(report.fields().iter().all(|f| !f.value().is_empty()));
            assert_eq!(report.rows().len(), Report::ADDRESS_COUNT as usize);
        }
    }

    #[test]
    fn the_descriptor_field_carries_a_checksum() {
        let report = report(ScriptType::P2wpkh);
        let descriptor = report
            .fields()
            .iter()
            .find(|field| field.value().starts_with("wpkh("))
            .expect("a descriptor field");
        assert!(descriptor.value().contains('#'));
        assert!(descriptor.value().contains("/84h/0h/0h]"));
    }

    #[test]
    fn the_plain_form_lists_every_field_and_row() {
        let report = report(ScriptType::P2wpkh);
        let lines = report.plain_lines();
        assert_eq!(lines[0], "Native SegWit");
        assert!(lines.iter().any(|l| l.contains("bc1qcr8te4kr609")));
        assert_eq!(lines.len(), 1 + report.fields().len() * 2 + 1 + 8);
    }
}
