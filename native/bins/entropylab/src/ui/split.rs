//! Column division: cuts a rect into side-by-side columns with a gutter.

use platform::draw::Rect;

pub struct Split {
    area: Rect,
    gutter: i32,
}

impl Split {
    #[must_use]
    pub const fn new(area: Rect, gutter: i32) -> Self {
        Self { area, gutter }
    }

    /// Cuts the area in two, the left column taking `percent` of the width
    /// that remains once the gutter is removed.
    #[must_use]
    pub const fn in_two(&self, percent: i32) -> (Rect, Rect) {
        let usable = self.area.w - self.gutter;
        let left_width = usable * percent / 100;
        let right_width = usable - left_width;
        (
            Rect::new(self.area.x, self.area.y, left_width, self.area.h),
            Rect::new(
                self.area.x + left_width + self.gutter,
                self.area.y,
                right_width,
                self.area.h,
            ),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::Split;
    use platform::draw::Rect;

    #[test]
    fn the_two_columns_and_the_gutter_fill_the_area() {
        let area = Rect::new(20, 60, 1000, 400);
        let (left, right) = Split::new(area, 24).in_two(50);
        assert_eq!(left.x, area.x);
        assert_eq!(right.right(), area.right());
        assert_eq!(right.x - left.right(), 24);
        assert_eq!(left.w + right.w + 24, area.w);
    }

    #[test]
    fn the_columns_never_overlap() {
        for percent in [10, 25, 46, 50, 75, 90] {
            let (left, right) = Split::new(Rect::new(0, 0, 900, 300), 20).in_two(percent);
            assert!(left.right() <= right.x, "columns overlap at {percent}%");
            assert!(left.w > 0 && right.w > 0, "empty column at {percent}%");
        }
    }

    #[test]
    fn a_bigger_share_widens_the_left_column() {
        let area = Rect::new(0, 0, 800, 200);
        let narrow = Split::new(area, 16).in_two(40).0.w;
        let wide = Split::new(area, 16).in_two(60).0.w;
        assert!(wide > narrow);
    }

    #[test]
    fn both_columns_keep_the_areas_vertical_extent() {
        let area = Rect::new(5, 40, 600, 250);
        let (left, right) = Split::new(area, 12).in_two(46);
        assert_eq!(left.y, area.y);
        assert_eq!(left.h, area.h);
        assert_eq!(right.y, area.y);
        assert_eq!(right.h, area.h);
    }
}
