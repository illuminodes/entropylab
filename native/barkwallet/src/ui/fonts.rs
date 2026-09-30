//! The two faces the window needs: a sans face for chrome and a mono face
//! for addresses/invoices.

use statusbar::TextRenderer;

pub struct Fonts {
    pub heading: TextRenderer,
    pub sans: TextRenderer,
    pub mono: TextRenderer,
}

struct Family {
    candidates: &'static [&'static str],
}

impl Family {
    const SANS: Self = Self {
        candidates: &[
            "/usr/share/fonts/noto/NotoSans-Regular.ttf",
            "/usr/share/fonts/noto/NotoSans-Medium.ttf",
        ],
    };

    const SANS_BOLD: Self = Self {
        candidates: &[
            "/usr/share/fonts/noto/NotoSans-Bold.ttf",
            "/usr/share/fonts/noto/NotoSans-Medium.ttf",
        ],
    };

    const MONO: Self = Self {
        candidates: &[
            "/usr/share/fonts/noto/NotoSansMono-Regular.ttf",
            "/usr/share/fonts/noto/NotoSansMono-Medium.ttf",
        ],
    };

    fn resolve(&self) -> Option<&'static str> {
        self.candidates
            .iter()
            .copied()
            .find(|path| std::path::Path::new(path).is_file())
    }

    fn load(&self, px_size: f32) -> TextRenderer {
        TextRenderer::load(self.resolve(), px_size)
    }
}

impl Fonts {
    #[must_use]
    pub fn load(scale: f32) -> Self {
        Self {
            heading: Family::SANS_BOLD.load(20.0 * scale),
            sans: Family::SANS.load(14.0 * scale),
            mono: Family::MONO.load(13.0 * scale),
        }
    }
}
