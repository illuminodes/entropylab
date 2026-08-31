//! The painter: the only part that touches the compositor.

use platform::draw::{Canvas, Color, Rect};
use platform::toplevel::{Key, KeyPress, Painter, PhysicalSize};
use statusbar::TextRenderer;

use crate::report::Report;
use crate::theme::Theme;

pub struct WalletWindow {
    report: Report,
    theme: Theme,
    text: TextRenderer,
    quit: bool,
    frames: u32,
}

impl WalletWindow {
    const MARGIN: i32 = 24;
    const LINE_HEIGHT: i32 = 22;
    const RULE_WIDTH: i32 = 2;

    pub const fn new(report: Report, theme: Theme, text: TextRenderer) -> Self {
        Self {
            report,
            theme,
            text,
            quit: false,
            frames: 0,
        }
    }

    pub const fn quit(&self) -> bool {
        self.quit
    }

    pub const fn frames(&self) -> u32 {
        self.frames
    }
}

impl Painter for WalletWindow {
    fn background(&self) -> Color {
        self.theme.background
    }

    fn on_key(&mut self, press: KeyPress) {
        if press.key == Key::Escape {
            self.quit = true;
        }
    }

    fn paint(&mut self, canvas: &mut Canvas<'_>, size: PhysicalSize) {
        self.frames += 1;
        canvas.fill_rect(
            Rect::new(0, 0, size.width, size.height),
            self.theme.background,
        );
        canvas.fill_rect(
            Rect::new(0, 0, Self::RULE_WIDTH, size.height),
            self.theme.accent,
        );

        let mut y = Self::MARGIN;
        for line in self.report.lines() {
            if y > size.height {
                break;
            }
            self.text.draw_text(
                canvas,
                line.text(),
                Self::MARGIN,
                y,
                self.theme.for_kind(line.kind()),
                self.theme.background,
            );
            y += Self::LINE_HEIGHT;
        }
    }
}
