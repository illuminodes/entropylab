//! The painter: the only part that touches the compositor.

use platform::draw::{Canvas, Color, Rect};
use platform::toplevel::{Key, KeyPress, Painter, PhysicalSize};

use crate::theme::Theme;
use crate::ui::fonts::Fonts;
use crate::ui::page::Page;

pub struct WalletWindow {
    page: Page,
    theme: Theme,
    fonts: Fonts,
    quit: bool,
    frames: u32,
}

impl WalletWindow {
    pub const fn new(page: Page, theme: Theme, fonts: Fonts) -> Self {
        Self {
            page,
            theme,
            fonts,
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
        self.page.paint(
            canvas,
            &mut self.fonts,
            &self.theme,
            Rect::new(0, 0, size.width, size.height),
        );
    }
}
