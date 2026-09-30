//! The palette. Kept close to `entropylab`'s so the two native wallets read
//! as siblings, not as unrelated apps.

use platform::draw::Color;

#[derive(Clone, Copy)]
pub struct Theme {
    pub background: Color,
    pub surface: Color,
    pub foreground: Color,
    pub muted: Color,
    pub faint: Color,
    pub border: Color,
    pub accent: Color,
    pub danger: Color,
    pub ok: Color,
}

impl Theme {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            background: Color::rgb(0x00, 0x00, 0x00),
            surface: Color::rgb(0x14, 0x14, 0x14),
            foreground: Color::rgb(0xee, 0xee, 0xee),
            muted: Color::rgb(0xa3, 0xa3, 0xa3),
            faint: Color::rgb(0x73, 0x73, 0x73),
            border: Color::rgb(0x33, 0x33, 0x33),
            accent: Color::rgb(0xd8, 0x89, 0x2b),
            danger: Color::rgb(0xff, 0x44, 0x38),
            ok: Color::rgb(0x22, 0xc5, 0x5e),
        }
    }
}

impl Default for Theme {
    fn default() -> Self {
        Self::new()
    }
}
