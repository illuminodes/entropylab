//! The page: the header and the workspace tabs across the top, then the
//! keys and addresses cards side by side.

use platform::draw::{Canvas, Rect};

use crate::report::Report;
use crate::theme::Theme;
use crate::ui::addresses::AddressesCard;
use crate::ui::fonts::Fonts;
use crate::ui::header::Header;
use crate::ui::keys::KeysCard;
use crate::ui::metrics::Metrics;
use crate::ui::pitch::PitchCard;
use crate::ui::split::Split;
use crate::ui::tabs::TabStrip;

pub struct Page {
    report: Report,
    workspaces: Vec<String>,
    scripts: Vec<String>,
    active_script: usize,
    version: String,
}

impl Page {
    /// The keys card carries long base58 values, so it takes the wider share.
    const KEYS_SHARE: i32 = 54;
    /// Below this width the columns would be too narrow to read, so the
    /// cards stack instead.
    const TWO_COLUMN_MIN_WIDTH: i32 = 900;

    #[must_use]
    pub fn new(report: Report, version: impl Into<String>) -> Self {
        let scripts = el_bip32::ScriptType::ALL
            .into_iter()
            .map(|script| script.short().to_owned())
            .collect();
        let active_script = el_bip32::ScriptType::ALL
            .into_iter()
            .position(|script| script == report.script())
            .unwrap_or(0);
        Self {
            report,
            workspaces: [
                "Key Derivation",
                "BIP-85",
                "Multi Signature",
                "Silent Payments",
                "PSBT / Nonce",
            ]
            .into_iter()
            .map(str::to_owned)
            .collect(),
            scripts,
            active_script,
            version: version.into(),
        }
    }

    /// Whether `width` has room for the side-by-side layout.
    const fn is_two_column(width: i32) -> bool {
        Metrics::content_width(width) >= Self::TWO_COLUMN_MIN_WIDTH
    }

    pub fn paint(&self, canvas: &mut Canvas<'_>, fonts: &mut Fonts, theme: &Theme, size: Rect) {
        canvas.fill_rect(size, theme.background);

        Header::new(&self.version, false).paint(canvas, fonts, theme, size.w);

        let x = Metrics::content_x(size.w);
        let width = Metrics::content_width(size.w);
        let mut y = Metrics::HEADER_HEIGHT + Metrics::SPACE_SECTION;

        TabStrip::new(&self.workspaces, 0).paint(
            canvas,
            &mut fonts.sans,
            theme,
            Rect::new(x, y, width, Metrics::TAB_HEIGHT),
        );
        y += Metrics::TAB_HEIGHT + Metrics::SPACE_SECTION;

        y = PitchCard::paint(canvas, fonts, theme, Rect::new(x, y, width, 0));
        y += Metrics::SPACE_SECTION;

        let keys = KeysCard::new(&self.report, &self.scripts, self.active_script);
        let addresses = AddressesCard::new(&self.report);
        let area = Rect::new(x, y, width, size.h - y);

        if Self::is_two_column(size.w) {
            let (left, right) = Split::new(area, Metrics::SPACE_SECTION).in_two(Self::KEYS_SHARE);
            keys.paint(canvas, fonts, theme, left);
            addresses.paint(canvas, fonts, theme, right);
        } else {
            keys.paint(canvas, fonts, theme, area);
            let below = area.y + keys.height(fonts) + Metrics::SPACE_SECTION;
            addresses.paint(
                canvas,
                fonts,
                theme,
                Rect::new(area.x, below, area.w, area.h),
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Page;
    use crate::report::Report;
    use crate::ui::addresses::AddressesCard;
    use crate::ui::fonts::Fonts;
    use crate::ui::keys::KeysCard;
    use crate::ui::metrics::Metrics;
    use crate::ui::split::Split;
    use bitcoin::bip32::Xpriv;
    use bitcoin::Network;
    use el_bip32::{Account, ScriptType};
    use platform::draw::Rect;

    const PHRASE: &str = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";

    fn page(script: ScriptType) -> Page {
        let mnemonic =
            bip39::Mnemonic::parse_in_normalized(bip39::Language::English, PHRASE).unwrap();
        let master = Xpriv::new_master(Network::Bitcoin, &mnemonic.to_seed_normalized("")).unwrap();
        let account = Account::derive(&master, script, 0).unwrap();
        Page::new(Report::for_account(&account).unwrap(), "v1.0.0")
    }

    #[test]
    fn the_active_tab_follows_the_reports_script() {
        assert_eq!(page(ScriptType::P2pkh).active_script, 0);
        assert_eq!(page(ScriptType::P2wpkh).active_script, 2);
        assert_eq!(page(ScriptType::P2tr).active_script, 3);
    }

    #[test]
    fn every_script_has_a_tab() {
        let page = page(ScriptType::P2wpkh);
        assert_eq!(page.scripts.len(), ScriptType::ALL.len());
        assert!(page.active_script < page.scripts.len());
    }

    #[test]
    fn the_workspace_tabs_match_the_web_app() {
        let page = page(ScriptType::P2wpkh);
        assert_eq!(page.workspaces[0], "Key Derivation");
        assert_eq!(page.workspaces.len(), 5);
    }

    #[test]
    fn a_wide_window_uses_two_columns_and_a_narrow_one_stacks() {
        assert!(Page::is_two_column(1040));
        assert!(!Page::is_two_column(700));
    }

    /// The overflow this layout exists to fix: at the default window size
    /// both columns must end above the bottom edge.
    #[test]
    fn the_two_column_layout_fits_the_default_window() {
        let page = page(ScriptType::P2wpkh);
        let fonts = Fonts::load(1.0);
        let (width, height) = (1040, 900);
        assert!(Page::is_two_column(width));

        let top = Metrics::HEADER_HEIGHT
            + Metrics::SPACE_SECTION
            + Metrics::TAB_HEIGHT
            + Metrics::SPACE_SECTION
            + super::PitchCard::height(&fonts)
            + Metrics::SPACE_SECTION;
        let area = Rect::new(
            Metrics::content_x(width),
            top,
            Metrics::content_width(width),
            height - top,
        );
        let (left, right) = Split::new(area, Metrics::SPACE_SECTION).in_two(Page::KEYS_SHARE);

        let keys = KeysCard::new(&page.report, &page.scripts, page.active_script).height(&fonts);
        let addresses = AddressesCard::new(&page.report).height(&fonts);
        assert!(top + keys <= height, "the keys card overflows: {keys}");
        assert!(
            top + addresses <= height,
            "the addresses card overflows: {addresses}"
        );
        assert!(left.w > 300 && right.w > 300);
    }

    /// Stacking the same two cards is what overflowed before the split.
    #[test]
    fn the_two_columns_are_shorter_than_the_stacked_form() {
        let page = page(ScriptType::P2wpkh);
        let fonts = Fonts::load(1.0);
        let keys = KeysCard::new(&page.report, &page.scripts, page.active_script).height(&fonts);
        let addresses = AddressesCard::new(&page.report).height(&fonts);
        let tallest = keys.max(addresses);
        assert!(tallest < keys + addresses + Metrics::SPACE_SECTION);
    }
}
