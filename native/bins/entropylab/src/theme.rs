//! The palette, taken from the cozy design guide.

use platform::draw::Color;

use crate::report::LineKind;

#[derive(Clone, Copy)]
pub struct Theme {
    pub background: Color,
    pub title: Color,
    pub label: Color,
    pub value: Color,
    pub address: Color,
    pub accent: Color,
}

impl Theme {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            background: Color::rgb(0x0d, 0x0f, 0x12),
            title: Color::rgb(0xd8, 0xde, 0xe9),
            label: Color::rgb(0x69, 0x72, 0x7f),
            value: Color::rgb(0x7d, 0xcf, 0xff),
            address: Color::rgb(0xd8, 0xde, 0xe9),
            accent: Color::rgb(0x9e, 0xce, 0x6a),
        }
    }

    #[must_use]
    pub const fn for_kind(&self, kind: LineKind) -> Color {
        match kind {
            LineKind::Title => self.title,
            LineKind::Label => self.label,
            LineKind::Value => self.value,
            LineKind::Address => self.address,
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
    use crate::report::LineKind;

    #[test]
    fn every_line_kind_has_its_own_colour() {
        let theme = Theme::new();
        assert_eq!(theme.for_kind(LineKind::Title), theme.title);
        assert_eq!(theme.for_kind(LineKind::Label), theme.label);
        assert_eq!(theme.for_kind(LineKind::Value), theme.value);
        assert_ne!(theme.for_kind(LineKind::Label), theme.value);
    }
}
