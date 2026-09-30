//! `barkwallet` -- an extremely simple Ark/Lightning wallet window: an
//! onboard address, and pay-a-lightning-invoice. Nothing else, to start.
//!
//! # Layering
//!
//! ```text
//! woof::ArkPos              the wallet                (no UI, no window)
//! wallet_worker::WalletWorker  async bridge thread     (no pixels at all)
//! app::App                  screen state + input rules (no pixels at all)
//! theme::Theme               the palette                (no window at all)
//! window::BarkWalletWindow  paint and keys              (the only Wayland part)
//! ```

mod app;
mod metrics;
mod theme;
mod ui;
mod wallet_worker;
mod window;

use app::App;
use platform::toplevel::{LogicalSize, Toplevel, WindowEvent};
use theme::Theme;
use ui::fonts::Fonts;
use window::BarkWalletWindow;

struct Runner;

impl Runner {
    const WIDTH: i32 = 720;
    const HEIGHT: i32 = 560;
    const FRAME_WAIT_MS: i32 = 100;

    fn run() -> Result<(), Box<dyn std::error::Error>> {
        let theme = Theme::new();
        let wake = waker::Waker::new()?;
        let mut window = Toplevel::new(
            "barkwallet",
            "BarkWallet",
            LogicalSize {
                width: Self::WIDTH,
                height: Self::HEIGHT,
            },
            wake,
            theme.background,
        )?;

        let scale = f32::from(i16::try_from(window.scale()).unwrap_or(1));
        let fonts = Fonts::load(scale);
        let app = App::new();
        let mut painter = BarkWalletWindow::new(app, theme, fonts);

        loop {
            window.tick(&mut painter)?;
            while let Some(event) = window.next_event() {
                if matches!(event, WindowEvent::CloseRequested) {
                    window.close()?;
                    return Ok(());
                }
            }
            if window.is_closing() || painter.quit() {
                break;
            }
            // Any wallet reply that arrived since the last paint needs a
            // frame to show it, even with no compositor event to trigger one.
            window.request_redraw();
            window.wait(Self::FRAME_WAIT_MS);
        }

        window.close()?;
        Ok(())
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    Runner::run()
}
