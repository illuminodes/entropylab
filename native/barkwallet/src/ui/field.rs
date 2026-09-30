//! A single-line editable bordered text field, painting a
//! [`widget::TextInput`].

use platform::draw::{Canvas, Rect};
use statusbar::TextRenderer;
use widget::TextInput;

use crate::metrics::Metrics;
use crate::theme::Theme;

pub struct FieldView<'a> {
    input: &'a TextInput,
    placeholder: &'static str,
    focused: bool,
}

impl<'a> FieldView<'a> {
    #[must_use]
    pub const fn new(input: &'a TextInput, placeholder: &'static str, focused: bool) -> Self {
        Self {
            input,
            placeholder,
            focused,
        }
    }

    pub fn paint(&self, canvas: &mut Canvas<'_>, font: &mut TextRenderer, theme: &Theme, rect: Rect) {
        canvas.fill_round_rect(rect, theme.background, Metrics::FIELD_RADIUS);
        let border = if self.focused { theme.accent } else { theme.border };
        canvas.draw_round_rect(rect, border, Metrics::FIELD_RADIUS);

        let text_y = rect.y + font.baseline_y(rect.h);
        let inset = Metrics::SPACE_CONTROL + 4;
        if self.input.is_empty() {
            font.draw_text(
                canvas,
                self.placeholder,
                rect.x + inset,
                text_y,
                theme.faint,
                theme.background,
            );
            return;
        }
        font.draw_text(
            canvas,
            self.input.text(),
            rect.x + inset,
            text_y,
            theme.foreground,
            theme.background,
        );
        if self.focused {
            let caret_x = rect.x + inset + font.measure(self.input.before_caret());
            canvas.fill_rect(
                Rect::new(caret_x, text_y - font.line_height() + 2, 2, font.line_height()),
                theme.accent,
            );
        }
    }
}
