//! The lines the window shows, built without any Wayland or pixels.
//!
//! Keeping the text model separate from the painter means the wallet output
//! is testable without a compositor.

use el_bip32::{Account, AccountError, Chain};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LineKind {
    Title,
    Label,
    Value,
    Address,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Line {
    kind: LineKind,
    text: String,
}

pub struct Report {
    lines: Vec<Line>,
}

impl Line {
    #[must_use]
    pub const fn kind(&self) -> LineKind {
        self.kind
    }

    #[must_use]
    pub fn text(&self) -> &str {
        &self.text
    }
}

impl Report {
    pub const ADDRESS_COUNT: u32 = 5;

    /// Builds the report for one account.
    pub fn for_account(account: &Account) -> Result<Self, AccountError> {
        let mut lines = vec![
            Self::line(LineKind::Title, account.script().label()),
            Self::line(LineKind::Label, "account path"),
            Self::line(LineKind::Value, account.display_path()),
            Self::line(LineKind::Label, "account xpub"),
            Self::line(LineKind::Value, account.slip132_xpub()?),
            Self::line(LineKind::Label, "receive descriptor"),
            Self::line(LineKind::Value, account.receive_descriptor().to_string()),
            Self::line(LineKind::Label, "first addresses"),
        ];

        for row in account.addresses(Chain::Receive, 0, Self::ADDRESS_COUNT)? {
            lines.push(Self::line(
                LineKind::Address,
                format!("{}  {}", row.index(), row.address()),
            ));
        }

        Ok(Self { lines })
    }

    fn line(kind: LineKind, text: impl Into<String>) -> Line {
        Line {
            kind,
            text: text.into(),
        }
    }

    #[must_use]
    pub fn lines(&self) -> &[Line] {
        &self.lines
    }
}

#[cfg(test)]
mod tests {
    use super::{LineKind, Report};
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
        let values: Vec<&str> = report
            .lines()
            .iter()
            .filter(|line| line.kind() == LineKind::Value)
            .map(super::Line::text)
            .collect();
        assert_eq!(values[0], "m/84h/0h/0h");
        assert_eq!(
            values[1],
            "zpub6rFR7y4Q2AijBEqTUquhVz398htDFrtymD9xYYfG1m4wAcvPhXNfE3EfH1r1ADqtfSdVCToUG868RvUUkgDKf31mGDtKsAYz2oz2AGutZYs"
        );
    }

    #[test]
    fn the_report_lists_five_addresses() {
        let report = report(ScriptType::P2wpkh);
        let addresses: Vec<&str> = report
            .lines()
            .iter()
            .filter(|line| line.kind() == LineKind::Address)
            .map(super::Line::text)
            .collect();
        assert_eq!(addresses.len(), Report::ADDRESS_COUNT as usize);
        assert_eq!(
            addresses[0],
            "0  bc1qcr8te4kr609gcawutmrza0j4xv80jy8z306fyu"
        );
        assert!(addresses[4].starts_with("4  bc1q"));
    }

    #[test]
    fn the_first_line_names_the_script() {
        for script in ScriptType::ALL {
            let report = report(script);
            let first = &report.lines()[0];
            assert_eq!(first.kind(), LineKind::Title);
            assert_eq!(first.text(), script.label());
        }
    }

    #[test]
    fn every_script_builds_a_report() {
        for script in ScriptType::ALL {
            let report = report(script);
            assert!(report.lines().len() > Report::ADDRESS_COUNT as usize);
            assert!(report.lines().iter().all(|line| !line.text().is_empty()));
        }
    }

    #[test]
    fn the_descriptor_line_carries_a_checksum() {
        let report = report(ScriptType::P2wpkh);
        let descriptor = report
            .lines()
            .iter()
            .find(|line| line.text().starts_with("wpkh("))
            .expect("a descriptor line");
        assert!(descriptor.text().contains('#'));
        assert!(descriptor.text().contains("/84h/0h/0h]"));
    }
}
