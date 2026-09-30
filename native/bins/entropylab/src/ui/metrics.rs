//! The spacing scale, taken from the web app's custom properties.

pub struct Metrics;

impl Metrics {
    pub const SPACE_CONTROL: i32 = 8;
    pub const SPACE_COMPONENT: i32 = 16;
    pub const SPACE_SECTION: i32 = 20;

    pub const HEADER_HEIGHT: i32 = 52;

    /// The web app caps `.wrap` at 1000px because a browser window is often
    /// far wider than a reading column. A native window is sized for this
    /// app alone, so the content uses the width it is given up to a ceiling
    /// that keeps a very wide monitor from stretching the cards absurdly.
    pub const WRAP_MAX_WIDTH: i32 = 1800;
    pub const WRAP_PAD: i32 = 24;

    pub const CARD_RADIUS: i32 = 20;
    pub const CARD_PAD: i32 = 20;

    pub const TAB_HEIGHT: i32 = 34;
    pub const TAB_RADIUS: i32 = 8;
    pub const TAB_PAD: i32 = 14;

    pub const FIELD_HEIGHT: i32 = 38;
    pub const FIELD_RADIUS: i32 = 12;

    pub const TABLE_RADIUS: i32 = 9;
    pub const TABLE_ROW_HEIGHT: i32 = 30;
    pub const TABLE_CELL_PAD: i32 = 10;

    /// The x where the centred content column starts for a window `width`
    /// wide, mirroring `.wrap`'s `max-width` with `margin: 0 auto`.
    #[must_use]
    pub const fn content_x(width: i32) -> i32 {
        let usable = width - Self::WRAP_PAD * 2;
        if usable <= Self::WRAP_MAX_WIDTH {
            Self::WRAP_PAD
        } else {
            (width - Self::WRAP_MAX_WIDTH) / 2
        }
    }

    /// The width of the centred content column.
    #[must_use]
    pub const fn content_width(width: i32) -> i32 {
        let usable = width - Self::WRAP_PAD * 2;
        if usable <= Self::WRAP_MAX_WIDTH {
            usable
        } else {
            Self::WRAP_MAX_WIDTH
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Metrics;

    #[test]
    fn a_narrow_window_keeps_the_side_padding() {
        assert_eq!(Metrics::content_x(600), Metrics::WRAP_PAD);
        assert_eq!(Metrics::content_width(600), 600 - Metrics::WRAP_PAD * 2);
    }

    #[test]
    fn a_wide_window_centres_the_column_at_its_maximum() {
        let width = Metrics::WRAP_MAX_WIDTH + 600;
        assert_eq!(Metrics::content_width(width), Metrics::WRAP_MAX_WIDTH);
        assert_eq!(Metrics::content_x(width), 300);
        assert_eq!(
            Metrics::content_x(width) * 2 + Metrics::content_width(width),
            width
        );
    }

    /// The window is sized for this app, so an ordinary window must not sit
    /// in a narrow ribbon with wasted space at both sides.
    #[test]
    fn an_ordinary_window_spends_almost_all_of_its_width() {
        for width in [1040, 1280, 1440, 1600] {
            let used = Metrics::content_width(width);
            assert!(
                used >= width - Metrics::WRAP_PAD * 2,
                "the column wastes space at {width}: {used}"
            );
        }
    }

    #[test]
    fn the_column_never_exceeds_the_window() {
        for width in [320, 700, 1000, 1032, 1033, 2560] {
            let total = Metrics::content_x(width) * 2 + Metrics::content_width(width);
            assert!(total <= width, "the column overflows at {width}");
            assert!(Metrics::content_width(width) > 0);
        }
    }
}
