//! `entropylab` — the native window for the wallet calculator.
//!
//! The window shows one account: its path, its SLIP-132 account key, its
//! receive descriptor, and the first addresses.
//!
//! # Layering
//!
//! ```text
//! el_bip32::Account     derivation           (no UI at all)
//! report::Report        the lines to show    (no pixels at all)
//! theme::Theme          the palette          (no window at all)
//! window::WalletWindow  paint and keys       (the only Wayland part)
//! ```

mod options;
mod report;
mod theme;
mod window;

use bitcoin::bip32::Xpriv;
use bitcoin::Network;
use platform::toplevel::{LogicalSize, Toplevel, WindowEvent};

use el_bip32::Account;
use options::Options;
use report::Report;
use theme::Theme;
use window::WalletWindow;

struct App;

impl App {
    const WIDTH: i32 = 900;
    const HEIGHT: i32 = 460;
    const FRAME_WAIT_MS: i32 = 16;

    fn run() -> Result<(), Box<dyn std::error::Error>> {
        let options = Options::parse(std::env::args().skip(1));
        let master = Self::master(&options)?;
        let account = Account::derive(&master, options.script(), options.account())?;
        let report = Report::for_account(&account)?;

        if options.headless() {
            for line in report.lines() {
                println!("{}", line.text());
            }
            return Ok(());
        }

        let cfg = config::Config::load().unwrap_or_else(|error| {
            eprintln!("entropylab: config error: {error} — using defaults");
            config::Config::default()
        });
        let theme = Theme::new();
        let wake = waker::Waker::new()?;
        let mut window = Toplevel::new(
            "entropylab",
            "entropylab",
            LogicalSize {
                width: Self::WIDTH,
                height: Self::HEIGHT,
            },
            wake,
            theme.background,
        )?;

        let scale = f32::from(i16::try_from(window.scale()).unwrap_or(1));
        let text = statusbar::TextRenderer::load_with_fallbacks(
            cfg.font.path.as_deref(),
            &[cfg.font.emoji_path.as_deref()],
            cfg.font.px_size() * scale,
        );
        let mut painter = WalletWindow::new(report, theme, text);

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
            if options.reached_frame_limit(painter.frames()) {
                println!("painted {} frames", painter.frames());
                break;
            }
            window.request_redraw();
            window.wait(Self::FRAME_WAIT_MS);
        }

        window.close()?;
        Ok(())
    }

    fn master(options: &Options) -> Result<Xpriv, Box<dyn std::error::Error>> {
        let mnemonic =
            bip39::Mnemonic::parse_in_normalized(bip39::Language::English, options.phrase())
                .map_err(|error| format!("the seed phrase is not valid: {error}"))?;
        Ok(Xpriv::new_master(
            Network::Bitcoin,
            &mnemonic.to_seed_normalized(options.passphrase()),
        )?)
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    App::run()
}
