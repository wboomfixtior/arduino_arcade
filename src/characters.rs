pub const SPECIAL_CHARACTERS: [[u8; 8]; 7] = [
    PLAYER,
    DIAMOND,
    HEART,
    SPADE,
    CLUB,
    TOP_SQUARE,
    BOTTOM_SQUARE,
];

#[rustfmt::skip]
pub const PLAYER: [u8; 8] = [
    0b00100,
    0b01010,
    0b00000,
    0b11011,
    0b10001,
    0b10101,
    0b01010,
    0b01010,
];

#[rustfmt::skip]
pub const DIAMOND: [u8; 8] = [
    0b00000,
    0b00000,
    0b00100,
    0b01110,
    0b11111,
    0b01110,
    0b00100,
    0b00000,
];

#[rustfmt::skip]
pub const HEART: [u8; 8] = [
    0b00000,
    0b00000,
    0b01010,
    0b11111,
    0b11111,
    0b01110,
    0b00100,
    0b00000,
];

#[rustfmt::skip]
pub const SPADE: [u8; 8] = [
    0b00000,
    0b00100,
    0b01110,
    0b11111,
    0b11111,
    0b00100,
    0b01110,
    0b00000,
];

#[rustfmt::skip]
pub const CLUB: [u8; 8] = [
    0b00000,
    0b01110,
    0b01110,
    0b10101,
    0b11111,
    0b00100,
    0b01110,
    0b00000,
];

#[rustfmt::skip]
pub const TOP_SQUARE: [u8; 8] = [
    0b11111,
    0b11111,
    0b11111,
    0b11111,
    0b00000,
    0b00000,
    0b00000,
    0b00000,
];

#[rustfmt::skip]
pub const BOTTOM_SQUARE: [u8; 8] = [
    0b00000,
    0b00000,
    0b00000,
    0b00000,
    0b11111,
    0b11111,
    0b11111,
    0b11111,
];
