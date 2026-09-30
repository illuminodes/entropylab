//! `entropylab` — the native window for the wallet calculator.
//!
//! The window mirrors the web app's Key Derivation screen: the site header,
//! the workspace tabs, the pitch card, and the key card with its script
//! tabs, account fields, and address table.
//!
//! # Layering
//!
//! ```text
//! el_bip32::Account     derivation           (no UI at all)
//! report::Report        the content to show  (no pixels at all)
//! theme::Theme          the palette          (no window at all)
//! ui::page::Page        the layout           (no compositor at all)
//! window::WalletWindow  paint and keys       (the only Wayland part)
//! ```

mod options;
mod report;
mod theme;
mod ui;
mod window;

use bitcoin::bip32::Xpriv;
use bitcoin::Network;
use platform::toplevel::{LogicalSize, Toplevel, WindowEvent};

use el_bip32::Account;
use options::Options;
use report::Report;
use theme::Theme;
use ui::fonts::Fonts;
use ui::page::Page;
use window::WalletWindow;

struct App;

impl App {
    const WIDTH: i32 = 1040;
    const HEIGHT: i32 = 900;
    const FRAME_WAIT_MS: i32 = 16;
    const VERSION: &'static str = concat!("v", env!("CARGO_PKG_VERSION"));

    fn run() -> Result<(), Box<dyn std::error::Error>> {
        let options = Options::parse(std::env::args().skip(1));
        let master = Self::master(&options)?;
        let account = Account::derive(&master, options.script(), options.account())?;
        let report = Report::for_account(&account)?;

        if options.headless() {
            for line in report.plain_lines() {
                println!("{line}");
            }
            return Ok(());
        }

        let theme = Theme::new();
        let wake = waker::Waker::new()?;
        let mut window = Toplevel::new(
            "entropylab",
            "EntropyLab",
            LogicalSize {
                width: Self::WIDTH,
                height: Self::HEIGHT,
            },
            wake,
            theme.background,
        )?;

        let scale = f32::from(i16::try_from(window.scale()).unwrap_or(1));
        let fonts = Fonts::load(scale);
        let page = Page::new(report, Self::VERSION);
        let mut painter = WalletWindow::new(page, theme, fonts);

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
