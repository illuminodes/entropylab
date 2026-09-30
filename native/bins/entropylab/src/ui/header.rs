//! The fixed site header: the logo mark, the wordmark, the version, and the
//! network status tag.

use platform::draw::{Canvas, Color, Rect};

use crate::theme::Theme;
use crate::ui::fonts::Fonts;
use crate::ui::lockup::Lockup;
use crate::ui::logo::Logo;
use crate::ui::metrics::Metrics;

pub struct Header {
    version: String,
    online: bool,
}

impl Header {
    const TITLE: &'static str = "EntropyLab";

    #[must_use]
    pub fn new(version: impl Into<String>, online: bool) -> Self {
        Self {
            version: version.into(),
            online,
        }
    }

    /// The status word and its colour. Offline is the safe state, so it is
    /// the green one.
    const fn status(&self, theme: &Theme) -> (&'static str, Color) {
        if self.online {
            ("ONLINE", theme.danger)
        } else {
            ("OFFLINE", theme.ok)
        }
    }

    pub fn paint(&self, canvas: &mut Canvas<'_>, fonts: &mut Fonts, theme: &Theme, width: i32) {
        let lockup = Lockup::new(Metrics::HEADER_HEIGHT);
        canvas.fill_rect(
            Rect::new(0, 0, width, Metrics::HEADER_HEIGHT),
            theme.background,
        );

        let rule_colour = if self.online {
            theme.danger
        } else {
            theme.border
        };
        canvas.fill_rect(lockup.rule(width), rule_colour);

        let x = Metrics::content_x(width);
        let right = x + Metrics::content_width(width);
        Logo::new(lockup.logo(x)).paint(canvas, theme);

        let title = lockup.title(x, fonts.display.line_height());
        let title_end = fonts.display.draw_text(
            canvas,
            Self::TITLE,
            title.x,
            title.y,
            theme.foreground,
            theme.background,
        );

        let version = lockup.version(title_end, fonts.small.line_height());
        fonts.small.draw_text(
            canvas,
            &self.version,
            version.x,
            version.y,
            theme.accent,
            theme.background,
        );

        let (word, colour) = self.status(theme);
        let word_width = fonts.small.measure(word);
        let status = lockup.status(right, word_width, fonts.small.line_height());
        fonts
            .small
            .draw_text(canvas, word, status.x, status.y, colour, theme.background);
    }
}

#[cfg(test)]
mod tests {
    use super::Header;
    use crate::theme::Theme;

    #[test]
    fn an_online_machine_shows_the_danger_colour() {
        let theme = Theme::new();
        let (word, colour) = Header::new("v1.0.0", true).status(&theme);
        assert_eq!(word, "ONLINE");
        assert_eq!(colour, theme.danger);
    }

    #[test]
    fn an_offline_machine_shows_the_safe_colour() {
        let theme = Theme::new();
        let (word, colour) = Header::new("v1.0.0", false).status(&theme);
        assert_eq!(word, "OFFLINE");
        assert_eq!(colour, theme.ok);
    }
}
