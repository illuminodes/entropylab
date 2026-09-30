//! The `.segmented-control` strip: joined tabs where the active one takes
//! the orange selection fill.

use platform::draw::{Canvas, Rect};
use statusbar::TextRenderer;

use crate::theme::Theme;
use crate::ui::metrics::Metrics;

pub struct TabStrip<'a> {
    labels: &'a [String],
    active: usize,
}

/// Where one tab sits and how its corners are rounded.
struct Slot {
    rect: Rect,
    first: bool,
    last: bool,
}

impl Slot {
    fn paint(
        &self,
        canvas: &mut Canvas<'_>,
        font: &mut TextRenderer,
        theme: &Theme,
        label: &str,
        active: bool,
    ) {
        let radius = Metrics::TAB_RADIUS;
        let (fill, ink) = if active {
            (theme.selection_accent, theme.selection_fg)
        } else {
            (theme.background, theme.muted)
        };
        canvas.fill_round_rect(self.rect, fill, radius);
        if active {
            canvas.draw_round_rect(self.rect, theme.selection_accent, radius);
        } else {
            canvas.draw_round_rect(self.rect, theme.border, radius);
        }
        // The joined look comes from square inner corners: repaint the seam
        // side as a plain rect so neighbours meet without a gap.
        if !self.first {
            let seam = Rect::new(self.rect.x, self.rect.y + 1, radius, self.rect.h - 2);
            canvas.fill_rect(seam, fill);
            canvas.fill_rect(Rect::new(seam.x, self.rect.y, radius, 1), theme.border);
            canvas.fill_rect(
                Rect::new(seam.x, self.rect.bottom() - 1, radius, 1),
                theme.border,
            );
        }
        if !self.last {
            let seam = Rect::new(
                self.rect.right() - radius,
                self.rect.y + 1,
                radius,
                self.rect.h - 2,
            );
            canvas.fill_rect(seam, fill);
            canvas.fill_rect(Rect::new(seam.x, self.rect.y, radius, 1), theme.border);
            canvas.fill_rect(
                Rect::new(seam.x, self.rect.bottom() - 1, radius, 1),
                theme.border,
            );
        }
        let text_width = font.measure(label);
        let x = self.rect.x + (self.rect.w - text_width) / 2;
        let y = self.rect.y + font.baseline_y(self.rect.h);
        font.draw_text(canvas, label, x, y, ink, fill);
    }
}

impl<'a> TabStrip<'a> {
    #[must_use]
    pub const fn new(labels: &'a [String], active: usize) -> Self {
        Self { labels, active }
    }

    /// The width the strip needs for `font`, sized to its widest label so
    /// every tab is equal, as a segmented control is.
    #[cfg(test)]
    fn width(&self, font: &mut TextRenderer) -> i32 {
        self.tab_width(font) * i32::try_from(self.labels.len()).unwrap_or(0)
    }

    fn tab_width(&self, font: &mut TextRenderer) -> i32 {
        let widest = self
            .labels
            .iter()
            .map(|label| font.measure(label))
            .max()
            .unwrap_or(0);
        widest + Metrics::TAB_PAD * 2
    }

    pub fn paint(&self, canvas: &mut Canvas<'_>, font: &mut TextRenderer, theme: &Theme, at: Rect) {
        let width = self.tab_width(font);
        let last = self.labels.len().saturating_sub(1);
        for (index, label) in self.labels.iter().enumerate() {
            let offset = i32::try_from(index).unwrap_or(0) * (width - 1);
            let slot = Slot {
                rect: Rect::new(at.x + offset, at.y, width, Metrics::TAB_HEIGHT),
                first: index == 0,
                last: index == last,
            };
            slot.paint(canvas, font, theme, label, index == self.active);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::TabStrip;

    fn labels() -> Vec<String> {
        ["Legacy", "Nested SegWit", "Native SegWit", "Taproot"]
            .into_iter()
            .map(str::to_owned)
            .collect()
    }

    #[test]
    fn the_strip_holds_every_label() {
        let labels = labels();
        let strip = TabStrip::new(&labels, 2);
        assert_eq!(strip.labels.len(), 4);
        assert_eq!(strip.active, 2);
    }

    #[test]
    fn an_empty_strip_has_no_width() {
        let empty: Vec<String> = Vec::new();
        let strip = TabStrip::new(&empty, 0);
        let mut font = statusbar::TextRenderer::load(None, 13.0);
        assert_eq!(strip.width(&mut font), 0);
    }

    #[test]
    fn the_width_grows_with_the_tab_count() {
        let labels = labels();
        let mut font = statusbar::TextRenderer::load(None, 13.0);
        let one = TabStrip::new(&labels[..1], 0).width(&mut font);
        let four = TabStrip::new(&labels, 0).width(&mut font);
        assert!(four > one);
    }
}
