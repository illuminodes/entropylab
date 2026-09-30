//! [`App`] -- the whole wallet's state, independent of pixels and Wayland.
//!
//! Owns the two screens' state and the worker handle, and translates raw
//! input (a click at a point, a typed key) into the state transitions.
//! Painting reads this; nothing here reads a `Canvas`.

use platform::draw::Rect;
use platform::toplevel::{Key, KeyPress};

use crate::metrics::Metrics;
use crate::ui::onboard::{OnboardScreen, OnboardState};
use crate::ui::pay::PayState;
use crate::ui::tabs::{Screen, Tabs};
use crate::wallet_worker::{WalletRequest, WalletResponse, WalletWorker};

pub struct App {
    pub screen: Screen,
    pub onboard: OnboardState,
    pub pay: PayState,
    pub connected: Option<Result<(), String>>,
    worker: WalletWorker,
}

impl App {
    #[must_use]
    pub fn new() -> Self {
        Self {
            screen: Screen::Onboard,
            onboard: OnboardState::default(),
            pay: PayState::default(),
            connected: None,
            worker: WalletWorker::spawn(),
        }
    }

    /// Drain any wallet replies that arrived since the last frame.
    pub fn poll(&mut self) {
        for response in self.worker.poll() {
            match response {
                WalletResponse::Connected(result) => self.connected = Some(result),
                WalletResponse::OnboardAddress(result) => self.onboard.on_address(result),
                WalletResponse::PayInvoice(result) => self.pay.on_paid(result),
            }
        }
    }

    const fn is_connected(&self) -> bool {
        matches!(self.connected, Some(Ok(())))
    }

    /// The screen content area beneath the header and tab strip.
    #[must_use]
    pub const fn content_area(width: i32, height: i32) -> Rect {
        let y = Metrics::MARGIN + Metrics::HEADER_HEIGHT + Metrics::SPACE_SECTION
            + Metrics::TAB_HEIGHT
            + Metrics::SPACE_SECTION;
        Rect::new(
            Metrics::MARGIN,
            y,
            width - Metrics::MARGIN * 2,
            height - y - Metrics::MARGIN,
        )
    }

    #[must_use]
    pub const fn tabs_area(width: i32) -> Rect {
        Rect::new(
            Metrics::MARGIN,
            Metrics::MARGIN + Metrics::HEADER_HEIGHT + Metrics::SPACE_SECTION,
            width - Metrics::MARGIN * 2,
            Metrics::TAB_HEIGHT,
        )
    }

    pub fn on_click(&mut self, x: i32, y: i32, width: i32, height: i32) {
        if let Some(screen) = Tabs::new(Self::tabs_area(width)).hit(x, y) {
            self.screen = screen;
            return;
        }
        if !self.is_connected() {
            return;
        }
        let area = Self::content_area(width, height);
        match self.screen {
            Screen::Onboard => {
                if OnboardScreen::button(area).contains(x, y) && !self.onboard.is_requesting() {
                    self.onboard.begin_request();
                    self.worker.send(WalletRequest::OnboardAddress);
                }
            }
            Screen::Pay => {
                let button = crate::ui::pay::PayScreen::new(&self.pay).button(area);
                if button.contains(x, y) && button.is_enabled() {
                    self.pay.begin_pay();
                    self.worker
                        .send(WalletRequest::PayInvoice(self.pay.invoice.text().to_owned()));
                }
            }
        }
    }

    pub fn on_key(&mut self, press: &KeyPress) {
        if self.screen != Screen::Pay {
            return;
        }
        match press.key {
            Key::Char(c) => {
                self.pay.invoice.insert(c);
            }
            Key::Backspace => {
                self.pay.invoice.backspace();
            }
            Key::Delete => {
                self.pay.invoice.delete();
            }
            Key::Left => {
                self.pay.invoice.move_left();
            }
            Key::Right => {
                self.pay.invoice.move_right();
            }
            Key::Home => {
                self.pay.invoice.move_home();
            }
            Key::End => {
                self.pay.invoice.move_end();
            }
            _ => {}
        }
    }
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}
