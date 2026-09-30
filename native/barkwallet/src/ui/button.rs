//! A single rounded, labelled push button.

use platform::draw::{Canvas, Rect};
use statusbar::TextRenderer;

use crate::metrics::Metrics;
use crate::theme::Theme;

pub struct Button {
    rect: Rect,
    label: &'static str,
    enabled: bool,
}

impl Button {
    #[must_use]
    pub const fn new(rect: Rect, label: &'static str, enabled: bool) -> Self {
        Self {
            rect,
            label,
            enabled,
        }
    }

    #[must_use]
    pub const fn contains(&self, x: i32, y: i32) -> bool {
        x >= self.rect.x
            && x < self.rect.x + self.rect.w
            && y >= self.rect.y
            && y < self.rect.y + self.rect.h
    }

    #[must_use]
    pub const fn is_enabled(&self) -> bool {
        self.enabled
    }

    pub fn paint(&self, canvas: &mut Canvas<'_>, font: &mut TextRenderer, theme: &Theme) {
        let fill = if self.enabled { theme.accent } else { theme.surface };
        canvas.fill_round_rect(self.rect, fill, Metrics::BUTTON_RADIUS);
        canvas.draw_round_rect(self.rect, theme.border, Metrics::BUTTON_RADIUS);

        let text_colour = if self.enabled {
            theme.background
        } else {
            theme.faint
        };
        let width = font.measure(self.label);
        let x = self.rect.x + (self.rect.w - width) / 2;
        let y = self.rect.y + font.baseline_y(self.rect.h);
        font.draw_text(canvas, self.label, x, y, text_colour, fill);
    }
}

#[cfg(test)]
mod tests {
    use super::Button;
    use platform::draw::Rect;

    #[test]
    fn contains_matches_the_rectangle_bounds() {
        let button = Button::new(Rect::new(10, 10, 100, 40), "Go", true);
        assert!(button.contains(10, 10));
        assert!(button.contains(109, 49));
        assert!(!button.contains(9, 10));
        assert!(!button.contains(110, 10));
        assert!(!button.contains(10, 50));
    }
}
