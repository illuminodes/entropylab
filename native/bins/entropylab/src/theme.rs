//! The palette, taken from the web app's `:root` block in `src/css/styles.css`.

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
    pub selection_accent: Color,
    pub selection_fg: Color,
    pub danger: Color,
    pub blue: Color,
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
            selection_accent: Color::rgb(0xff, 0x99, 0x00),
            selection_fg: Color::rgb(0x00, 0x00, 0x00),
            danger: Color::rgb(0xff, 0x44, 0x38),
            blue: Color::rgb(0x6f, 0x9f, 0xca),
            ok: Color::rgb(0x22, 0xc5, 0x5e),
        }
    }
}

impl Default for Theme {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::Theme;

    #[test]
    fn the_label_and_value_colours_differ() {
        let theme = Theme::new();
        assert_ne!(theme.muted, theme.blue);
        assert_ne!(theme.foreground, theme.muted);
    }

    #[test]
    fn the_palette_matches_the_web_app_root_block() {
        let theme = Theme::new();
        assert_eq!(theme.background, platform::draw::Color::rgb(0, 0, 0));
        assert_eq!(
            theme.selection_accent,
            platform::draw::Color::rgb(0xff, 0x99, 0x00)
        );
        assert_eq!(theme.surface, platform::draw::Color::rgb(0x14, 0x14, 0x14));
    }
}
