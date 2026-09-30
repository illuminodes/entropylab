//! The Pay screen: paste a BOLT11 invoice, pay it from the Ark balance.

use platform::draw::{Canvas, Rect};
use widget::TextInput;

use crate::metrics::Metrics;
use crate::theme::Theme;
use crate::ui::button::Button;
use crate::ui::field::FieldView;
use crate::ui::fonts::Fonts;

#[derive(Default)]
pub struct PayState {
    pub invoice: TextInput,
    paying: bool,
    result: Option<Result<(), String>>,
}

impl PayState {
    #[must_use]
    pub const fn is_paying(&self) -> bool {
        self.paying
    }

    #[must_use]
    pub fn can_pay(&self) -> bool {
        !self.paying && !self.invoice.is_empty()
    }

    pub fn begin_pay(&mut self) {
        self.paying = true;
        self.result = None;
    }

    pub fn on_paid(&mut self, result: Result<(), String>) {
        self.paying = false;
        self.result = Some(result);
    }
}

pub struct PayScreen<'a> {
    state: &'a PayState,
}

impl<'a> PayScreen<'a> {
    #[must_use]
    pub const fn new(state: &'a PayState) -> Self {
        Self { state }
    }

    #[must_use]
    pub const fn field_rect(area: Rect) -> Rect {
        Rect::new(area.x, area.y, area.w, Metrics::FIELD_HEIGHT)
    }

    #[must_use]
    pub const fn button_rect(area: Rect) -> Rect {
        Rect::new(
            area.x,
            area.y + Metrics::FIELD_HEIGHT + Metrics::SPACE_SECTION,
            160,
            Metrics::BUTTON_HEIGHT,
        )
    }

    #[must_use]
    pub fn button(&self, area: Rect) -> Button {
        Button::new(Self::button_rect(area), "Pay", self.state.can_pay())
    }

    pub fn paint(&self, canvas: &mut Canvas<'_>, fonts: &mut Fonts, theme: &Theme, area: Rect) {
        FieldView::new(&self.state.invoice, "lnbc1...", true).paint(
            canvas,
            &mut fonts.mono,
            theme,
            Self::field_rect(area),
        );

        let label = if self.state.is_paying() { "Paying..." } else { "Pay" };
        Button::new(Self::button_rect(area), label, self.state.can_pay())
            .paint(canvas, &mut fonts.sans, theme);

        let Some(result) = &self.state.result else {
            return;
        };
        let y = Self::button_rect(area).bottom() + Metrics::SPACE_SECTION;
        let (text, colour) = match result {
            Ok(()) => ("Payment settled.".to_owned(), theme.ok),
            Err(e) => (format!("Payment failed: {e}"), theme.danger),
        };
        fonts
            .sans
            .draw_text(canvas, &text, area.x, y + fonts.sans.line_height(), colour, theme.background);
    }
}
