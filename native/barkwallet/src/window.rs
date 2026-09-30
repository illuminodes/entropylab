//! The painter: the only part that touches the compositor.

use platform::draw::{Canvas, Color, Rect};
use platform::protocol::PointerButton;
use platform::toplevel::{KeyPress, Painter, PhysicalSize, PointerEventKind};

use crate::app::App;
use crate::metrics::Metrics;
use crate::theme::Theme;
use crate::ui::fonts::Fonts;
use crate::ui::onboard::OnboardScreen;
use crate::ui::pay::PayScreen;
use crate::ui::tabs::{Screen, Tabs};

pub struct BarkWalletWindow {
    app: App,
    theme: Theme,
    fonts: Fonts,
    quit: bool,
    size: PhysicalSize,
}

impl BarkWalletWindow {
    #[must_use]
    pub fn new(app: App, theme: Theme, fonts: Fonts) -> Self {
        Self {
            app,
            theme,
            fonts,
            quit: false,
            size: PhysicalSize {
                width: 0,
                height: 0,
            },
        }
    }

    #[must_use]
    pub const fn quit(&self) -> bool {
        self.quit
    }

    pub fn poll(&mut self) {
        self.app.poll();
    }

    fn paint_header(&mut self, canvas: &mut Canvas<'_>, width: i32) {
        let title = "Bark Wallet";
        let y = Metrics::MARGIN + self.fonts.heading.baseline_y(Metrics::HEADER_HEIGHT);
        self.fonts.heading.draw_text(
            canvas,
            title,
            Metrics::MARGIN,
            y,
            self.theme.foreground,
            self.theme.background,
        );

        let (status, colour) = match &self.app.connected {
            None => ("connecting...", self.theme.muted),
            Some(Ok(())) => ("connected", self.theme.ok),
            Some(Err(_)) => ("connection failed", self.theme.danger),
        };
        let status_width = self.fonts.sans.measure(status);
        self.fonts.sans.draw_text(
            canvas,
            status,
            width - Metrics::MARGIN - status_width,
            Metrics::MARGIN + self.fonts.sans.baseline_y(Metrics::HEADER_HEIGHT),
            colour,
            self.theme.background,
        );
    }
}

impl Painter for BarkWalletWindow {
    fn background(&self) -> Color {
        self.theme.background
    }

    fn on_key(&mut self, press: KeyPress) {
        self.app.on_key(&press);
    }

    fn on_pointer(&mut self, event: PointerEventKind) {
        if let PointerEventKind::Press {
            button: PointerButton::Left,
            at,
            ..
        } = event
        {
            self.app
                .on_click(at.x, at.y, self.size.width, self.size.height);
        }
    }

    fn paint(&mut self, canvas: &mut Canvas<'_>, size: PhysicalSize) {
        self.size = size;
        self.poll();

        canvas.fill_rect(Rect::new(0, 0, size.width, size.height), self.theme.background);
        self.paint_header(canvas, size.width);

        let tabs_area = App::tabs_area(size.width);
        Tabs::new(tabs_area).paint(canvas, &mut self.fonts.sans, &self.theme, self.app.screen);

        let content = App::content_area(size.width, size.height);
        match self.app.screen {
            Screen::Onboard => {
                OnboardScreen::new(&self.app.onboard).paint(canvas, &mut self.fonts, &self.theme, content);
            }
            Screen::Pay => {
                PayScreen::new(&self.app.pay).paint(canvas, &mut self.fonts, &self.theme, content);
            }
        }
    }
}
