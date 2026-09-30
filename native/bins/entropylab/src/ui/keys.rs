//! The keys card: the heading, the master fingerprint, the script tabs, and
//! the account fields.

use platform::draw::{Canvas, Rect};

use crate::report::Report;
use crate::theme::Theme;
use crate::ui::card::Card;
use crate::ui::field::FieldView;
use crate::ui::fonts::Fonts;
use crate::ui::metrics::Metrics;
use crate::ui::tabs::TabStrip;

pub struct KeysCard<'a> {
    report: &'a Report,
    scripts: &'a [String],
    active_script: usize,
}

impl<'a> KeysCard<'a> {
    const HEADING: &'static str = "Keys";

    #[must_use]
    pub const fn new(report: &'a Report, scripts: &'a [String], active_script: usize) -> Self {
        Self {
            report,
            scripts,
            active_script,
        }
    }

    /// The height the card occupies for `fonts`.
    pub fn height(&self, fonts: &Fonts) -> i32 {
        let fields: i32 = self
            .report
            .fields()
            .iter()
            .map(|field| FieldView::new(field).height(fonts) + Metrics::SPACE_COMPONENT)
            .sum();
        Metrics::CARD_PAD * 2
            + fonts.heading.line_height()
            + Metrics::SPACE_CONTROL
            + fonts.mono_small.line_height()
            + Metrics::SPACE_COMPONENT
            + Metrics::TAB_HEIGHT
            + Metrics::SPACE_SECTION
            + fields
            - Metrics::SPACE_COMPONENT
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

        let fingerprint = format!("Master fingerprint  {}", self.report.fingerprint());
        fonts
            .mono_small
            .draw_text(canvas, &fingerprint, inner.x, y, theme.faint, theme.surface);
        y += fonts.mono_small.line_height() + Metrics::SPACE_COMPONENT;

        TabStrip::new(self.scripts, self.active_script).paint(
            canvas,
            &mut fonts.small,
            theme,
            Rect::new(inner.x, y, inner.w, Metrics::TAB_HEIGHT),
        );
        y += Metrics::TAB_HEIGHT + Metrics::SPACE_SECTION;

        for field in self.report.fields() {
            y = FieldView::new(field).paint(
                canvas,
                fonts,
                theme,
                Rect::new(inner.x, y, inner.w, 0),
            );
            y += Metrics::SPACE_COMPONENT;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::KeysCard;
    use crate::report::Report;
    use crate::ui::fonts::Fonts;
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
    fn the_card_is_tall_enough_for_every_field() {
        let report = report();
        let scripts = vec!["Legacy".to_owned(), "SegWit".to_owned()];
        let fonts = Fonts::load(1.0);
        let card = KeysCard::new(&report, &scripts, 1);
        let height = card.height(&fonts);
        assert!(height > 0);
        assert!(i32::try_from(report.fields().len()).unwrap() * 40 < height);
    }
}
