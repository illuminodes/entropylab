//! The three faces the web app uses: a serif display face for the wordmark
//! and headings, a sans face for chrome, and a mono face for key material.

use statusbar::TextRenderer;

pub struct Fonts {
    pub display: TextRenderer,
    pub heading: TextRenderer,
    pub sans: TextRenderer,
    pub sans_bold: TextRenderer,
    pub small: TextRenderer,
    pub mono: TextRenderer,
    pub mono_small: TextRenderer,
}

/// One family, resolved to the first path that exists on this machine.
struct Family {
    candidates: &'static [&'static str],
}

impl Family {
    const DISPLAY: Self = Self {
        candidates: &[
            "/usr/share/fonts/noto/NotoSerif-SemiBold.ttf",
            "/usr/share/fonts/noto/NotoSerif-Bold.ttf",
            "/usr/share/fonts/noto/NotoSerif-Medium.ttf",
            "/usr/share/fonts/noto/NotoSerif-Regular.ttf",
        ],
    };

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
    /// Loads every face at `scale`, the window's integer output scale.
    #[must_use]
    pub fn load(scale: f32) -> Self {
        Self {
            display: Family::DISPLAY.load(22.0 * scale),
            heading: Family::DISPLAY.load(19.0 * scale),
            sans: Family::SANS.load(13.0 * scale),
            sans_bold: Family::SANS_BOLD.load(13.0 * scale),
            small: Family::SANS.load(11.0 * scale),
            mono: Family::MONO.load(12.5 * scale),
            mono_small: Family::MONO.load(11.5 * scale),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Family;

    #[test]
    fn every_family_resolves_to_a_file_that_exists() {
        for family in [
            &Family::DISPLAY,
            &Family::SANS,
            &Family::SANS_BOLD,
            &Family::MONO,
        ] {
            if let Some(path) = family.resolve() {
                assert!(std::path::Path::new(path).is_file());
            }
        }
    }

    #[test]
    fn the_display_and_mono_families_are_distinct() {
        assert_ne!(Family::DISPLAY.candidates[0], Family::MONO.candidates[0]);
    }

    /// The wordmark and the status tag must both fit the header bar.
    #[test]
    fn the_header_faces_fit_the_header_bar() {
        let fonts = super::Fonts::load(1.0);
        let bar = crate::ui::metrics::Metrics::HEADER_HEIGHT;
        assert!(
            fonts.display.line_height() <= bar,
            "the wordmark is {}px in a {bar}px bar",
            fonts.display.line_height()
        );
        assert!(fonts.small.line_height() <= bar);
    }
}
