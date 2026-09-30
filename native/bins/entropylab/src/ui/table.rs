//! The address table: the web app's `#`, `Path`, and `Address` columns in a
//! bordered, rounded scroll region.

use platform::draw::{Canvas, Rect};

use crate::report::AddressRow;
use crate::theme::Theme;
use crate::ui::fit::TextFit;
use crate::ui::fonts::Fonts;
use crate::ui::metrics::Metrics;

pub struct AddressTable<'a> {
    rows: &'a [AddressRow],
}

/// The three column widths, resolved for the table's own width.
struct Columns {
    index: i32,
    path: i32,
    address: i32,
}

impl Columns {
    const INDEX_WIDTH: i32 = 44;
    const PATH_WIDTH: i32 = 150;

    fn resolve(width: i32) -> Self {
        let index = Self::INDEX_WIDTH;
        let path = Self::PATH_WIDTH;
        Self {
            index,
            path,
            address: (width - index - path - Metrics::TABLE_CELL_PAD).max(0),
        }
    }

    /// The width a cell's text may occupy, once its padding is removed.
    const fn text_width(column: i32) -> i32 {
        let usable = column - Metrics::TABLE_CELL_PAD * 2;
        if usable > 0 {
            usable
        } else {
            0
        }
    }

    const fn path_x(&self, x: i32) -> i32 {
        x + self.index
    }

    const fn address_x(&self, x: i32) -> i32 {
        x + self.index + self.path
    }

    /// Whether the address column has room to draw at all.
    const fn shows_addresses(&self) -> bool {
        self.address > 0
    }
}

impl<'a> AddressTable<'a> {
    #[must_use]
    pub const fn new(rows: &'a [AddressRow]) -> Self {
        Self { rows }
    }

    /// The height the whole table occupies, header row included.
    pub fn height(&self) -> i32 {
        let count = i32::try_from(self.rows.len()).unwrap_or(0);
        Metrics::TABLE_ROW_HEIGHT * (count + 1)
    }

    pub fn paint(&self, canvas: &mut Canvas<'_>, fonts: &mut Fonts, theme: &Theme, at: Rect) {
        let height = self.height();
        let frame = Rect::new(at.x, at.y, at.w, height);
        canvas.fill_round_rect(frame, theme.background, Metrics::TABLE_RADIUS);

        let columns = Columns::resolve(at.w);
        let pad = Metrics::TABLE_CELL_PAD;
        let head = Rect::new(at.x, at.y, at.w, Metrics::TABLE_ROW_HEIGHT);
        canvas.fill_round_rect(head, theme.surface, Metrics::TABLE_RADIUS);
        canvas.fill_rect(
            Rect::new(
                head.x,
                head.bottom() - Metrics::TABLE_RADIUS,
                head.w,
                Metrics::TABLE_RADIUS,
            ),
            theme.surface,
        );

        let head_y = at.y + fonts.sans_bold.baseline_y(Metrics::TABLE_ROW_HEIGHT);
        for (label, x) in [
            ("#", at.x + pad),
            ("Path", columns.path_x(at.x) + pad),
            ("Address", columns.address_x(at.x) + pad),
        ] {
            fonts
                .sans_bold
                .draw_text(canvas, label, x, head_y, theme.muted, theme.surface);
        }

        let mut y = at.y + Metrics::TABLE_ROW_HEIGHT;
        for row in self.rows {
            canvas.fill_rect(Rect::new(at.x, y, at.w, 1), theme.border);
            let text_y = y + fonts.mono_small.baseline_y(Metrics::TABLE_ROW_HEIGHT);
            let index = row.index().to_string();
            fonts.mono_small.draw_text(
                canvas,
                &index,
                at.x + pad,
                text_y,
                theme.muted,
                theme.background,
            );

            let path_width = Columns::text_width(columns.path);
            let path = TextFit::new(&mut fonts.mono_small, path_width).tail(row.path());
            fonts.mono_small.draw_text(
                canvas,
                &path,
                columns.path_x(at.x) + pad,
                text_y,
                theme.muted,
                theme.background,
            );

            if columns.shows_addresses() {
                let address_width = Columns::text_width(columns.address);
                let address =
                    TextFit::new(&mut fonts.mono_small, address_width).middle(row.address());
                fonts.mono_small.draw_text(
                    canvas,
                    &address,
                    columns.address_x(at.x) + pad,
                    text_y,
                    theme.foreground,
                    theme.background,
                );
            }
            y += Metrics::TABLE_ROW_HEIGHT;
        }

        canvas.draw_round_rect(frame, theme.border, Metrics::TABLE_RADIUS);
    }
}

#[cfg(test)]
mod tests {
    use super::{AddressTable, Columns};
    use crate::ui::metrics::Metrics;

    #[test]
    fn an_empty_table_still_shows_its_header() {
        assert_eq!(AddressTable::new(&[]).height(), Metrics::TABLE_ROW_HEIGHT);
    }

    #[test]
    fn the_columns_run_left_to_right_without_overlap() {
        let columns = Columns::resolve(800);
        assert!(columns.path_x(0) >= columns.index);
        assert!(columns.address_x(0) > columns.path_x(0));
        assert!(columns.address > 0);
    }

    #[test]
    fn a_narrow_table_never_gives_a_column_a_negative_width() {
        let columns = Columns::resolve(60);
        assert_eq!(columns.address, 0);
        assert!(!columns.shows_addresses());
        assert!(Columns::resolve(800).shows_addresses());
    }

    /// The bug this clipping fixes: an address must never be drawn wider
    /// than the column that holds it.
    #[test]
    fn every_address_is_shortened_to_its_column() {
        let mut fonts = crate::ui::fonts::Fonts::load(1.0);
        let address = "bc1qcr8te4kr609gcawutmrza0j4xv80jy8z306fyu";
        for table_width in [260, 320, 420, 560, 700] {
            let columns = Columns::resolve(table_width);
            let room = Columns::text_width(columns.address);
            let drawn = crate::ui::fit::TextFit::new(&mut fonts.mono_small, room).middle(address);
            assert!(
                fonts.mono_small.measure(&drawn) <= room,
                "the address overflows at table width {table_width}"
            );
        }
    }

    #[test]
    fn the_text_width_never_goes_negative() {
        assert_eq!(Columns::text_width(4), 0);
        assert!(Columns::text_width(200) > 0);
    }
}
