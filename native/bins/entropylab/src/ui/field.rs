//! The `.field` control: a label, a bordered read-only value box, and an
//! optional note under it.

use platform::draw::{Canvas, Rect};

use crate::report::Field;
use crate::theme::Theme;
use crate::ui::fit::TextFit;
use crate::ui::fonts::Fonts;
use crate::ui::metrics::Metrics;

pub struct FieldView<'a> {
    field: &'a Field,
}

impl<'a> FieldView<'a> {
    #[must_use]
    pub const fn new(field: &'a Field) -> Self {
        Self { field }
    }

    /// The height this field occupies, label and note included.
    pub fn height(&self, fonts: &Fonts) -> i32 {
        let mut height = fonts.sans_bold.line_height() + Metrics::SPACE_CONTROL;
        height += Metrics::FIELD_HEIGHT;
        if self.field.note().is_some() {
            height += Metrics::SPACE_CONTROL / 2 + fonts.small.line_height();
        }
        height
    }

    /// Draws the field at `at`, and returns the y just past it.
    pub fn paint(
        &self,
        canvas: &mut Canvas<'_>,
        fonts: &mut Fonts,
        theme: &Theme,
        at: Rect,
    ) -> i32 {
        let mut y = at.y;
        fonts.sans_bold.draw_text(
            canvas,
            self.field.label(),
            at.x,
            y,
            theme.foreground,
            theme.surface,
        );
        y += fonts.sans_bold.line_height() + Metrics::SPACE_CONTROL;

        let box_rect = Rect::new(at.x, y, at.w, Metrics::FIELD_HEIGHT);
        canvas.fill_round_rect(box_rect, theme.background, Metrics::FIELD_RADIUS);
        canvas.draw_round_rect(box_rect, theme.border, Metrics::FIELD_RADIUS);

        let text_y = y + fonts.mono.baseline_y(Metrics::FIELD_HEIGHT);
        let inner_width = at.w - Metrics::SPACE_COMPONENT;
        let value = TextFit::new(&mut fonts.mono, inner_width).middle(self.field.value());
        fonts.mono.draw_text(
            canvas,
            &value,
            at.x + Metrics::SPACE_CONTROL + 4,
            text_y,
            theme.blue,
            theme.background,
        );
        y += Metrics::FIELD_HEIGHT;

        if let Some(note) = self.field.note() {
            y += Metrics::SPACE_CONTROL / 2;
            fonts
                .small
                .draw_text(canvas, note, at.x, y, theme.faint, theme.surface);
            y += fonts.small.line_height();
        }
        y
    }
}

#[cfg(test)]
mod tests {
    use crate::ui::fit::TextFit;
    use crate::ui::fonts::Fonts;

    #[test]
    fn a_short_value_is_left_untouched() {
        let mut fonts = Fonts::load(1.0);
        assert_eq!(
            TextFit::new(&mut fonts.mono, 10_000).middle("zpub6r"),
            "zpub6r"
        );
    }

    #[test]
    fn a_long_value_is_cut_to_the_field_width() {
        let mut fonts = Fonts::load(1.0);
        let long = "z".repeat(400);
        let fitted = TextFit::new(&mut fonts.mono, 120).middle(&long);
        assert!(fitted.contains('\u{2026}'));
        assert!(fitted.chars().count() < long.chars().count());
        assert!(fonts.mono.measure(&fitted) <= 120);
    }
}
