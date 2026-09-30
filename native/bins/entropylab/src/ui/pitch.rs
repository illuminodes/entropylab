//! The lede card: the kicker, the headline, and the claim list.

use platform::draw::{Canvas, Rect};

use crate::theme::Theme;
use crate::ui::card::Card;
use crate::ui::fonts::Fonts;
use crate::ui::metrics::Metrics;

pub struct PitchCard;

impl PitchCard {
    const KICKER: &'static str = "RUN OFFLINE \u{b7} BRING YOUR OWN ENTROPY";
    const HEADLINE: &'static str = "Hold or receive bitcoin without a signing device.";
    const CLAIMS: [&'static str; 3] = [
        "Turn dice rolls or a seed you already have into receive addresses.",
        "Export an xpub and load it into any watch-only wallet.",
        "Keep your private keys offline.",
    ];

    /// The height the card occupies for `fonts`.
    pub fn height(fonts: &Fonts) -> i32 {
        let claims = i32::try_from(Self::CLAIMS.len()).unwrap_or(0);
        Metrics::CARD_PAD * 2
            + fonts.small.line_height()
            + Metrics::SPACE_CONTROL
            + fonts.display.line_height()
            + Metrics::SPACE_COMPONENT
            + (fonts.sans.line_height() + Metrics::SPACE_CONTROL / 2) * claims
    }

    /// Draws the card at `at`, and returns the y just past its bottom edge.
    pub fn paint(canvas: &mut Canvas<'_>, fonts: &mut Fonts, theme: &Theme, at: Rect) -> i32 {
        let card = Card::new(Rect::new(at.x, at.y, at.w, Self::height(fonts)));
        card.paint(canvas, theme);

        let inner = card.inner();
        let mut y = inner.y;
        fonts.small.draw_text(
            canvas,
            Self::KICKER,
            inner.x,
            y,
            theme.accent,
            theme.surface,
        );
        y += fonts.small.line_height() + Metrics::SPACE_CONTROL;

        fonts.display.draw_text(
            canvas,
            Self::HEADLINE,
            inner.x,
            y,
            theme.foreground,
            theme.surface,
        );
        y += fonts.display.line_height() + Metrics::SPACE_COMPONENT;

        for claim in Self::CLAIMS {
            canvas.fill_rect(
                Rect::new(inner.x + 3, y + fonts.sans.line_height() / 2, 3, 3),
                theme.faint,
            );
            fonts.sans.draw_text(
                canvas,
                claim,
                inner.x + Metrics::SPACE_COMPONENT,
                y,
                theme.muted,
                theme.surface,
            );
            y += fonts.sans.line_height() + Metrics::SPACE_CONTROL / 2;
        }

        card.rect_bottom()
    }
}

#[cfg(test)]
mod tests {
    use super::PitchCard;
    use crate::ui::fonts::Fonts;
    use crate::ui::metrics::Metrics;

    #[test]
    fn the_card_is_tall_enough_for_its_padding_and_claims() {
        let fonts = Fonts::load(1.0);
        let height = PitchCard::height(&fonts);
        assert!(height > Metrics::CARD_PAD * 2);
        assert!(height > fonts.display.line_height() * 3);
    }

    #[test]
    fn the_kicker_is_upper_case_as_the_stylesheet_sets_it() {
        assert_eq!(PitchCard::KICKER, PitchCard::KICKER.to_uppercase());
    }
}
