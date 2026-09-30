//! The `.card` surface: a rounded panel on the page background.

use platform::draw::{Canvas, Rect};

use crate::theme::Theme;
use crate::ui::metrics::Metrics;

pub struct Card {
    rect: Rect,
}

impl Card {
    #[must_use]
    pub const fn new(rect: Rect) -> Self {
        Self { rect }
    }

    /// The rect inside the card's padding, where its content is drawn.
    #[must_use]
    pub const fn inner(&self) -> Rect {
        Rect::new(
            self.rect.x + Metrics::CARD_PAD,
            self.rect.y + Metrics::CARD_PAD,
            self.rect.w - Metrics::CARD_PAD * 2,
            self.rect.h - Metrics::CARD_PAD * 2,
        )
    }

    /// The y just past the card's bottom edge.
    #[must_use]
    pub const fn rect_bottom(&self) -> i32 {
        self.rect.bottom()
    }

    pub fn paint(&self, canvas: &mut Canvas<'_>, theme: &Theme) {
        canvas.fill_round_rect(self.rect, theme.surface, Metrics::CARD_RADIUS);
        canvas.draw_round_rect(self.rect, theme.border, Metrics::CARD_RADIUS);
    }
}

#[cfg(test)]
mod tests {
    use super::Card;
    use crate::ui::metrics::Metrics;
    use platform::draw::Rect;

    #[test]
    fn the_inner_rect_sits_inside_the_padding() {
        let card = Card::new(Rect::new(10, 20, 400, 200));
        let inner = card.inner();
        assert_eq!(inner.x, 10 + Metrics::CARD_PAD);
        assert_eq!(inner.y, 20 + Metrics::CARD_PAD);
        assert_eq!(inner.w, 400 - Metrics::CARD_PAD * 2);
        assert_eq!(inner.h, 200 - Metrics::CARD_PAD * 2);
    }
}
