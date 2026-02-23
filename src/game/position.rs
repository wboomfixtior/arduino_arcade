pub type Position = GenericPosition<1>;

#[derive(Copy, Clone, Eq, PartialEq)]
pub struct GenericPosition<const ROW_BITS: u8>(u8);

impl<const ROW_BITS: u8> GenericPosition<ROW_BITS> {
    pub const COLUMN_MASK: u8 = 0x0f;
    pub const ROW_MASK: u8 = {
        assert!(ROW_BITS < 8);
        u8::MAX >> (u8::BITS as u8 - ROW_BITS)
    };

    pub fn new(column: u8, row: u8) -> Self {
        Self((column & Self::COLUMN_MASK) << ROW_BITS | row & Self::ROW_MASK)
    }

    pub fn column(self) -> u8 {
        self.0 >> ROW_BITS
    }

    pub fn row(self) -> u8 {
        self.0 & Self::ROW_MASK
    }

    #[must_use]
    pub fn with_row(self, row: u8) -> Self {
        Self(self.0 & !Self::ROW_MASK | row & Self::ROW_MASK)
    }

    #[must_use]
    pub fn with_column(self, column: u8) -> Self {
        Self((column & Self::COLUMN_MASK) << ROW_BITS | self.0 & Self::ROW_MASK)
    }

    #[must_use]
    pub fn nudge_column_saturating(self, offset: i8) -> Self {
        self.with_column(
            self.column()
                .saturating_add_signed(offset)
                .clamp(0, Self::COLUMN_MASK),
        )
    }

    #[must_use]
    pub fn nudge_column_overflowing(self, offset: i8) -> (Self, bool) {
        (
            self.nudge_column_saturating(offset),
            self.column().saturating_add_signed(offset) > Self::COLUMN_MASK
                || offset < 0 && self.column() < offset.abs_diff(0),
        )
    }
}
