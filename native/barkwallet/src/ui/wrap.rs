//! Hard-wraps a long, unbroken string (an address, an invoice) to a pixel
//! width by chopping at whatever character boundary fits -- there are no
//! word breaks to wrap at in a bech32/bolt11 string.

use statusbar::TextRenderer;

pub struct Wrap<'a> {
    font: &'a mut TextRenderer,
    width: i32,
}

impl<'a> Wrap<'a> {
    #[must_use]
    pub const fn new(font: &'a mut TextRenderer, width: i32) -> Self {
        Self { font, width }
    }

    /// Split `text` into lines that each fit `width`, character-preserving
    /// (no ellipsis: the caller needs every character legible, e.g. to copy
    /// an address by hand).
    pub fn lines(&mut self, text: &str) -> Vec<String> {
        if self.width <= 0 {
            return vec![text.to_owned()];
        }
        let mut lines = Vec::new();
        let mut current = String::new();
        for ch in text.chars() {
            let mut candidate = current.clone();
            candidate.push(ch);
            if !current.is_empty() && self.font.measure(&candidate) > self.width {
                lines.push(current);
                current = String::new();
            }
            current.push(ch);
        }
        if !current.is_empty() {
            lines.push(current);
        }
        if lines.is_empty() {
            lines.push(String::new());
        }
        lines
    }
}

#[cfg(test)]
mod tests {
    use super::Wrap;
    use statusbar::TextRenderer;

    fn font() -> TextRenderer {
        TextRenderer::load(None, 13.0)
    }

    const ADDRESS: &str = "bc1qcr8te4kr609gcawutmrza0j4xv80jy8z306fyubc1qcr8te4kr609gcawutmrza0j4xv80jy8z306fyu";

    #[test]
    fn every_line_fits_the_given_width() {
        let mut font = font();
        for width in [60, 120, 200, 400] {
            let lines = Wrap::new(&mut font, width).lines(ADDRESS);
            for line in &lines {
                assert!(font.measure(line) <= width, "{line:?} overflows {width}");
            }
        }
    }

    #[test]
    fn no_character_is_lost_across_the_wrap() {
        let mut font = font();
        let lines = Wrap::new(&mut font, 80).lines(ADDRESS);
        assert_eq!(lines.concat(), ADDRESS);
    }

    #[test]
    fn a_short_string_stays_on_one_line() {
        let mut font = font();
        let lines = Wrap::new(&mut font, 10_000).lines("short");
        assert_eq!(lines, vec!["short".to_owned()]);
    }

    #[test]
    fn an_empty_string_yields_one_empty_line() {
        let mut font = font();
        let lines = Wrap::new(&mut font, 200).lines("");
        assert_eq!(lines, vec![String::new()]);
    }
}
