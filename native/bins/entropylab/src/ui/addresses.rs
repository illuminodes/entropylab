//! The addresses card: the heading, the chain note, and the address table.

use platform::draw::{Canvas, Rect};

use crate::report::Report;
use crate::theme::Theme;
use crate::ui::card::Card;
use crate::ui::fonts::Fonts;
use crate::ui::metrics::Metrics;
use crate::ui::table::AddressTable;

pub struct AddressesCard<'a> {
    report: &'a Report,
}

impl<'a> AddressesCard<'a> {
    const HEADING: &'static str = "Addresses";
    const NOTE: &'static str = "Receive chain \u{b7} external";

    #[must_use]
    pub const fn new(report: &'a Report) -> Self {
        Self { report }
    }

    /// The height the card occupies for `fonts`.
    pub fn height(&self, fonts: &Fonts) -> i32 {
        Metrics::CARD_PAD * 2
            + fonts.heading.line_height()
            + Metrics::SPACE_CONTROL
            + fonts.mono_small.line_height()
            + Metrics::SPACE_COMPONENT
            + AddressTable::new(self.report.rows()).height()
    }

    pub fn paint(&self, canvas: &mut Canvas<'_>, fonts: &mut Fonts, theme: &Theme, at: Rect) {
        let card = Card::new(Rect::new(at.x, at.y, at.w, self.height(fonts)));
        card.paint(canvas, theme);

        let inner = card.inner();
        let mut y = inner.y;
        fonts.heading.draw_text(
            canvas,
            Self::HEADING,
            inner.x,
            y,
            theme.foreground,
            theme.surface,
        );
        y += fonts.heading.line_height() + Metrics::SPACE_CONTROL;

        fonts
            .mono_small
            .draw_text(canvas, Self::NOTE, inner.x, y, theme.faint, theme.surface);
        y += fonts.mono_small.line_height() + Metrics::SPACE_COMPONENT;

        AddressTable::new(self.report.rows()).paint(
            canvas,
            fonts,
            theme,
            Rect::new(inner.x, y, inner.w, 0),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::AddressesCard;
    use crate::report::Report;
    use crate::ui::fonts::Fonts;
    use crate::ui::table::AddressTable;
    use bitcoin::bip32::Xpriv;
    use bitcoin::Network;
    use el_bip32::{Account, ScriptType};

    const PHRASE: &str = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";

    fn report() -> Report {
        let mnemonic =
            bip39::Mnemonic::parse_in_normalized(bip39::Language::English, PHRASE).unwrap();
        let master = Xpriv::new_master(Network::Bitcoin, &mnemonic.to_seed_normalized("")).unwrap();
        let account = Account::derive(&master, ScriptType::P2wpkh, 0).unwrap();
        Report::for_account(&account).unwrap()
    }

    #[test]
    fn the_card_wraps_the_table_with_room_for_the_heading() {
        let report = report();
        let fonts = Fonts::load(1.0);
        let table = AddressTable::new(report.rows()).height();
        assert!(AddressesCard::new(&report).height(&fonts) > table);
    }
}
