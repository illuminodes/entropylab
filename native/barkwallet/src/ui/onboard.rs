//! The Onboard screen: a button that requests a fresh on-chain funding
//! address, and the address itself once it arrives.

use platform::draw::{Canvas, Rect};

use crate::metrics::Metrics;
use crate::theme::Theme;
use crate::ui::button::Button;
use crate::ui::wrap::Wrap;

#[derive(Default)]
pub struct OnboardState {
    address: Option<String>,
    requesting: bool,
    error: Option<String>,
}

impl OnboardState {
    #[must_use]
    pub const fn is_requesting(&self) -> bool {
        self.requesting
    }

    pub fn begin_request(&mut self) {
        self.requesting = true;
        self.error = None;
    }

    pub fn on_address(&mut self, result: Result<String, String>) {
        self.requesting = false;
        match result {
            Ok(address) => self.address = Some(address),
            Err(e) => self.error = Some(e),
        }
    }
}

pub struct OnboardScreen<'a> {
    state: &'a OnboardState,
}

impl<'a> OnboardScreen<'a> {
    #[must_use]
    pub const fn new(state: &'a OnboardState) -> Self {
        Self { state }
    }

    /// The generate/refresh button's rect, given the screen area -- used by
    /// both painting and hit-testing so they never disagree.
    #[must_use]
    pub const fn button_rect(area: Rect) -> Rect {
        Rect::new(area.x, area.y, 220, Metrics::BUTTON_HEIGHT)
    }

    #[must_use]
    pub fn button(area: Rect) -> Button {
        Button::new(Self::button_rect(area), "Get address", true)
    }

    pub fn paint(&self, canvas: &mut Canvas<'_>, fonts: &mut super::fonts::Fonts, theme: &Theme, area: Rect) {
        let label = if self.state.requesting {
            "Requesting..."
        } else {
            "Get address"
        };
        Button::new(Self::button_rect(area), label, !self.state.requesting)
            .paint(canvas, &mut fonts.sans, theme);

        let mut y = area.y + Metrics::BUTTON_HEIGHT + Metrics::SPACE_SECTION;

        if let Some(error) = &self.state.error {
            fonts
                .sans
                .draw_text(canvas, error, area.x, y + fonts.sans.line_height(), theme.danger, theme.background);
            return;
        }

        let Some(address) = &self.state.address else {
            fonts.sans.draw_text(
                canvas,
                "Press the button to generate a bitcoin address you can send funds to.",
                area.x,
                y + fonts.sans.line_height(),
                theme.muted,
                theme.background,
            );
            return;
        };

        fonts.sans.draw_text(
            canvas,
            "Send bitcoin to this address to board it onto your Ark balance:",
            area.x,
            y + fonts.sans.line_height(),
            theme.muted,
            theme.background,
        );
        y += fonts.sans.line_height() + Metrics::SPACE_CONTROL * 2;

        let card = Rect::new(area.x, y, area.w, Self::card_height(fonts, area.w, address));
        canvas.fill_round_rect(card, theme.surface, Metrics::CARD_RADIUS);
        canvas.draw_round_rect(card, theme.border, Metrics::CARD_RADIUS);

        let inner_width = area.w - Metrics::SPACE_SECTION * 2;
        let lines = Wrap::new(&mut fonts.mono, inner_width).lines(address);
        let mut line_y = y + Metrics::SPACE_CONTROL * 2;
        for line in &lines {
            fonts.mono.draw_text(
                canvas,
                line,
                area.x + Metrics::SPACE_SECTION,
                line_y + fonts.mono.line_height(),
                theme.foreground,
                theme.surface,
            );
            line_y += fonts.mono.line_height() + 4;
        }
    }

    fn card_height(fonts: &mut super::fonts::Fonts, width: i32, address: &str) -> i32 {
        let inner_width = width - Metrics::SPACE_SECTION * 2;
        let lines = Wrap::new(&mut fonts.mono, inner_width).lines(address);
        Metrics::SPACE_CONTROL * 4
            + i32::try_from(lines.len()).unwrap_or(1) * (fonts.mono.line_height() + 4)
    }
}
