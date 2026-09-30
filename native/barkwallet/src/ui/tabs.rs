//! The two-tab strip: Onboard | Pay invoice.

use platform::draw::{Canvas, Rect};
use statusbar::TextRenderer;

use crate::metrics::Metrics;
use crate::theme::Theme;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Screen {
    Onboard,
    Pay,
}

impl Screen {
    const ALL: [Self; 2] = [Self::Onboard, Self::Pay];

    const fn label(self) -> &'static str {
        match self {
            Self::Onboard => "Onboard",
            Self::Pay => "Pay invoice",
        }
    }
}

pub struct Tabs {
    rect: Rect,
}

impl Tabs {
    #[must_use]
    pub const fn new(rect: Rect) -> Self {
        Self { rect }
    }

    fn slot(&self, index: usize) -> Rect {
        let width = self.rect.w / i32::try_from(Screen::ALL.len()).unwrap_or(1);
        Rect::new(
            self.rect.x + width * i32::try_from(index).unwrap_or(0),
            self.rect.y,
            width,
            self.rect.h,
        )
    }

    /// Which tab, if any, contains `(x, y)`.
    #[must_use]
    pub fn hit(&self, x: i32, y: i32) -> Option<Screen> {
        for (index, screen) in Screen::ALL.into_iter().enumerate() {
            let slot = self.slot(index);
            if x >= slot.x && x < slot.x + slot.w && y >= slot.y && y < slot.y + slot.h {
                return Some(screen);
            }
        }
        None
    }

    pub fn paint(&self, canvas: &mut Canvas<'_>, font: &mut TextRenderer, theme: &Theme, active: Screen) {
        for (index, screen) in Screen::ALL.into_iter().enumerate() {
            let slot = self.slot(index);
            let is_active = screen == active;
            let fill = if is_active { theme.surface } else { theme.background };
            canvas.fill_round_rect(slot, fill, Metrics::TAB_RADIUS);
            if is_active {
                canvas.draw_round_rect(slot, theme.accent, Metrics::TAB_RADIUS);
            }
            let label = screen.label();
            let width = font.measure(label);
            let x = slot.x + (slot.w - width) / 2;
            let y = slot.y + font.baseline_y(slot.h);
            let colour = if is_active { theme.foreground } else { theme.muted };
            font.draw_text(canvas, label, x, y, colour, fill);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Screen, Tabs};
    use platform::draw::Rect;

    #[test]
    fn a_hit_in_the_left_half_selects_onboard() {
        let tabs = Tabs::new(Rect::new(0, 0, 200, 40));
        assert_eq!(tabs.hit(10, 10), Some(Screen::Onboard));
    }

    #[test]
    fn a_hit_in_the_right_half_selects_pay() {
        let tabs = Tabs::new(Rect::new(0, 0, 200, 40));
        assert_eq!(tabs.hit(150, 10), Some(Screen::Pay));
    }

    #[test]
    fn a_hit_outside_the_strip_selects_nothing() {
        let tabs = Tabs::new(Rect::new(0, 0, 200, 40));
        assert_eq!(tabs.hit(10, 100), None);
    }
}
