//! Text fitting: shortens a string until it fits a pixel width.

use statusbar::TextRenderer;

pub struct TextFit<'a> {
    font: &'a mut TextRenderer,
    width: i32,
}

impl<'a> TextFit<'a> {
    const ELLIPSIS: &'static str = "\u{2026}";

    #[must_use]
    pub const fn new(font: &'a mut TextRenderer, width: i32) -> Self {
        Self { font, width }
    }

    /// Whether `text` already fits.
    pub fn fits(&mut self, text: &str) -> bool {
        self.font.measure(text) <= self.width
    }

    /// Drops characters from the end and appends an ellipsis.
    pub fn tail(&mut self, text: &str) -> String {
        if self.fits(text) {
            return text.to_owned();
        }
        let budget = self.width - self.font.measure(Self::ELLIPSIS);
        let mut kept = String::new();
        for ch in text.chars() {
            let mut candidate = kept.clone();
            candidate.push(ch);
            if self.font.measure(&candidate) > budget {
                break;
            }
            kept = candidate;
        }
        kept.push_str(Self::ELLIPSIS);
        kept
    }

    /// Removes characters from the middle, keeping both ends. An address is
    /// checked by its first and last characters, so both must stay legible.
    pub fn middle(&mut self, text: &str) -> String {
        if self.fits(text) {
            return text.to_owned();
        }
        let chars: Vec<char> = text.chars().collect();
        let budget = self.width - self.font.measure(Self::ELLIPSIS);
        let mut keep = 0;
        loop {
            let next = keep + 1;
            if next * 2 >= chars.len() {
                break;
            }
            let head: String = chars[..next].iter().collect();
            let tail: String = chars[chars.len() - next..].iter().collect();
            if self.font.measure(&format!("{head}{tail}")) > budget {
                break;
            }
            keep = next;
        }
        if keep == 0 {
            return self.tail(text);
        }
        let head: String = chars[..keep].iter().collect();
        let tail: String = chars[chars.len() - keep..].iter().collect();
        format!("{head}{}{tail}", Self::ELLIPSIS)
    }
}

#[cfg(test)]
mod tests {
    use super::TextFit;
    use statusbar::TextRenderer;

    fn font() -> TextRenderer {
        TextRenderer::load(None, 13.0)
    }

    const ADDRESS: &str = "bc1qcr8te4kr609gcawutmrza0j4xv80jy8z306fyu";

    #[test]
    fn a_short_string_is_left_untouched() {
        let mut font = font();
        assert_eq!(TextFit::new(&mut font, 10_000).tail(ADDRESS), ADDRESS);
        assert_eq!(TextFit::new(&mut font, 10_000).middle(ADDRESS), ADDRESS);
    }

    #[test]
    fn the_tail_form_fits_the_width_and_is_marked() {
        let mut font = font();
        let fitted = TextFit::new(&mut font, 120).tail(ADDRESS);
        assert!(fitted.ends_with('\u{2026}'));
        assert!(font.measure(&fitted) <= 120);
    }

    #[test]
    fn the_middle_form_keeps_both_ends() {
        let mut font = font();
        let fitted = TextFit::new(&mut font, 150).middle(ADDRESS);
        assert!(fitted.contains('\u{2026}'));
        assert!(fitted.starts_with("bc1"));
        assert!(fitted.ends_with("fyu"));
        assert!(font.measure(&fitted) <= 150);
    }

    #[test]
    fn every_width_produces_a_string_that_fits() {
        let mut font = font();
        for width in [20, 40, 60, 90, 130, 200, 300] {
            let tail = TextFit::new(&mut font, width).tail(ADDRESS);
            let middle = TextFit::new(&mut font, width).middle(ADDRESS);
            assert!(font.measure(&tail) <= width, "tail overflows at {width}");
            assert!(
                font.measure(&middle) <= width,
                "middle overflows at {width}"
            );
        }
    }

    #[test]
    fn a_hopeless_width_still_returns_something_short() {
        let mut font = font();
        let fitted = TextFit::new(&mut font, 1).middle(ADDRESS);
        assert!(fitted.chars().count() < ADDRESS.chars().count());
    }
}
