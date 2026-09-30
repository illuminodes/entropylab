//! The logo mark: a green sprout over an orange vessel that carries a die.
//!
//! The source asset is an SVG of bezier curves. This is a plain-rectangle
//! reading of the same shapes, which is what the canvas primitives draw.

use platform::draw::{Canvas, Color, Rect};

use crate::theme::Theme;

pub struct Logo {
    rect: Rect,
}

impl Logo {
    const GREEN: Color = Color::rgb(0x0e, 0x9a, 0x4d);
    const ORANGE: Color = Color::rgb(0xf7, 0x91, 0x00);

    #[must_use]
    pub const fn new(rect: Rect) -> Self {
        Self { rect }
    }

    /// The vessel occupies the bottom of the mark, the sprout the top.
    const fn vessel(&self) -> Rect {
        let h = self.rect.h * 45 / 100;
        Rect::new(self.rect.x, self.rect.bottom() - h, self.rect.w, h)
    }

    const fn stem(&self) -> Rect {
        let w = if self.rect.w / 9 > 2 {
            self.rect.w / 9
        } else {
            2
        };
        let top = self.rect.y + self.rect.h * 30 / 100;
        Rect::new(
            self.rect.x + (self.rect.w - w) / 2,
            top,
            w,
            self.vessel().y - top + 1,
        )
    }

    const fn leaf(&self) -> Rect {
        let w = self.rect.w * 52 / 100;
        let h = self.rect.h * 30 / 100;
        Rect::new(
            self.rect.x + self.rect.w - w,
            self.rect.y + self.rect.h * 4 / 100,
            w,
            h,
        )
    }

    pub fn paint(&self, canvas: &mut Canvas<'_>, theme: &Theme) {
        let vessel = self.vessel();
        canvas.fill_round_rect(vessel, Self::ORANGE, vessel.w / 3);
        canvas.fill_rect(self.stem(), Self::GREEN);
        let leaf = self.leaf();
        canvas.fill_round_rect(leaf, Self::GREEN, leaf.h / 2);

        let pip = (vessel.w / 9).max(2);
        let centre_x = vessel.x + (vessel.w - pip) / 2;
        let centre_y = vessel.y + (vessel.h - pip) / 2;
        let step = pip * 2;
        for (dx, dy) in [(-step, -step), (0, 0), (step, step)] {
            canvas.fill_rect(
                Rect::new(centre_x + dx, centre_y + dy, pip, pip),
                theme.surface,
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Logo;
    use platform::draw::Rect;

    fn logo() -> Logo {
        Logo::new(Rect::new(10, 10, 30, 30))
    }

    #[test]
    fn every_part_stays_inside_the_mark() {
        let logo = logo();
        for part in [logo.vessel(), logo.stem(), logo.leaf()] {
            assert!(part.x >= logo.rect.x, "{part:?} starts left of the mark");
            assert!(part.right() <= logo.rect.right(), "{part:?} runs right");
            assert!(part.y >= logo.rect.y, "{part:?} starts above the mark");
            assert!(part.bottom() <= logo.rect.bottom(), "{part:?} runs below");
        }
    }

    #[test]
    fn the_stem_reaches_from_the_leaf_down_to_the_vessel() {
        let logo = logo();
        assert!(logo.stem().bottom() >= logo.vessel().y);
        assert!(logo.stem().y < logo.vessel().y);
    }

    #[test]
    fn the_vessel_sits_below_the_leaf() {
        let logo = logo();
        assert!(logo.vessel().y > logo.leaf().bottom());
    }
}
