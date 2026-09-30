//! Header geometry: places the logo, the wordmark, the version, and the
//! status tag inside the header bar.
//!
//! `draw_text` takes a cell-top origin, so every box here is a full text
//! cell, not a baseline. A box that starts below `height - line_height`
//! spills out of the bar.

use platform::draw::Rect;

pub struct Lockup {
    height: i32,
}

impl Lockup {
    pub const LOGO_SIZE: i32 = 28;
    const LOGO_GAP: i32 = 10;
    const VERSION_GAP: i32 = 8;

    #[must_use]
    pub const fn new(height: i32) -> Self {
        Self { height }
    }

    /// Centres a box `extent` tall in the bar, clamped so it never starts
    /// above the top edge.
    const fn centre(&self, extent: i32) -> i32 {
        let y = (self.height - extent) / 2;
        if y > 0 {
            y
        } else {
            0
        }
    }

    #[must_use]
    pub const fn logo(&self, x: i32) -> Rect {
        Rect::new(
            x,
            self.centre(Self::LOGO_SIZE),
            Self::LOGO_SIZE,
            Self::LOGO_SIZE,
        )
    }

    /// The wordmark cell, `line_height` tall, set beside the logo.
    #[must_use]
    pub const fn title(&self, x: i32, line_height: i32) -> Rect {
        let logo = self.logo(x);
        Rect::new(
            logo.right() + Self::LOGO_GAP,
            self.centre(line_height),
            0,
            line_height,
        )
    }

    /// The version cell, set after the wordmark's measured width.
    #[must_use]
    pub const fn version(&self, title_end: i32, line_height: i32) -> Rect {
        Rect::new(
            title_end + Self::VERSION_GAP,
            self.centre(line_height),
            0,
            line_height,
        )
    }

    /// The status cell, right-aligned against the content column.
    #[must_use]
    pub const fn status(&self, right: i32, text_width: i32, line_height: i32) -> Rect {
        Rect::new(
            right - text_width,
            self.centre(line_height),
            text_width,
            line_height,
        )
    }

    /// The rule under the bar.
    #[must_use]
    pub const fn rule(&self, width: i32) -> Rect {
        Rect::new(0, self.height - 1, width, 1)
    }
}

#[cfg(test)]
mod tests {
    use super::Lockup;
    use crate::ui::metrics::Metrics;

    const HEIGHT: i32 = Metrics::HEADER_HEIGHT;

    /// The bug this module fixes: every part must end inside the bar.
    #[test]
    fn every_part_stays_inside_the_bar() {
        let lockup = Lockup::new(HEIGHT);
        let logo = lockup.logo(24);
        let title = lockup.title(24, 26);
        let version = lockup.version(title.x + 120, 14);
        let status = lockup.status(1000, 60, 14);
        for part in [logo, title, version, status] {
            assert!(part.y >= 0, "{part:?} starts above the bar");
            assert!(
                part.bottom() <= HEIGHT,
                "{part:?} spills below the {HEIGHT}px bar"
            );
        }
    }

    /// A face taller than the bar must still not start off the top edge.
    #[test]
    fn an_oversized_face_is_clamped_to_the_top() {
        let lockup = Lockup::new(HEIGHT);
        assert_eq!(lockup.title(0, 200).y, 0);
    }

    #[test]
    fn the_parts_run_left_to_right_without_overlap() {
        let lockup = Lockup::new(HEIGHT);
        let logo = lockup.logo(24);
        let title = lockup.title(24, 26);
        assert!(title.x > logo.right());
        let version = lockup.version(title.x + 120, 14);
        assert!(version.x > title.x + 120);
    }

    #[test]
    fn the_status_tag_ends_at_the_right_edge() {
        let status = Lockup::new(HEIGHT).status(980, 60, 14);
        assert_eq!(status.right(), 980);
    }

    #[test]
    fn the_rule_sits_on_the_last_row_of_the_bar() {
        let rule = Lockup::new(HEIGHT).rule(1200);
        assert_eq!(rule.bottom(), HEIGHT);
        assert_eq!(rule.w, 1200);
    }
}
