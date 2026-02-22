use core::{mem, num::NonZeroU8};

use crate::{game::position::Position, LCD};

#[derive(Copy, Clone, Eq, PartialEq)]
pub struct Card(NonZeroU8);

impl Card {
    pub fn new(number: Number, suit: Suit) -> Self {
        // SAFETY: `suit as u8` is always nonzero
        Self(unsafe { NonZeroU8::new_unchecked(number as u8 | suit as u8) })
    }

    pub fn number(self) -> Number {
        // SAFETY: The 4 most significant bits must have been set from a Number
        unsafe { mem::transmute::<u8, Number>(self.0.get() & 0xf0) }
    }

    pub fn suit(self) -> Suit {
        // SAFETY: The 4 least significant bits must have been set from a Suit
        unsafe { mem::transmute::<u8, Suit>(self.0.get() & 0x0f) }
    }

    pub fn draw_at(self, lcd: &mut LCD, column: u8) {
        lcd.set_cursor(Position::new(column, 0));
        lcd.write(self.suit().character());
        lcd.set_cursor(Position::new(column, 1));
        lcd.write(self.number().character());
    }
}

/// NOTE: Only occupies the 4 most significant bits
#[derive(Copy, Clone, Eq, PartialEq)]
#[repr(u8)]
pub enum Number {
    Ace = 0x00,
    Number1 = 0x10,
    Number2 = 0x20,
    Number3 = 0x30,
    Number4 = 0x40,
    Number5 = 0x50,
    Number6 = 0x60,
    Number7 = 0x70,
    Number8 = 0x80,
    Number9 = 0x90,
    Number10 = 0xa0,
    Jack = 0xb0,
    Queen = 0xc0,
    King = 0xd0,
}

/// NOTE: Only occupies the 4 least significant bits
#[derive(Copy, Clone, Eq, PartialEq)]
#[repr(u8)]
pub enum Suit {
    Diamonds = 1,
    Hearts = 2,
    Spades = 3,
    Clubs = 4,
}

impl Number {
    pub fn character(self) -> u8 {
        match self {
            Number::Ace => b'A',
            Number::Number1 => b'1',
            Number::Number2 => b'2',
            Number::Number3 => b'3',
            Number::Number4 => b'4',
            Number::Number5 => b'5',
            Number::Number6 => b'6',
            Number::Number7 => b'7',
            Number::Number8 => b'8',
            Number::Number9 => b'9',
            Number::Number10 => b't',
            Number::Jack => b'J',
            Number::Queen => b'Q',
            Number::King => b'K',
        }
    }
}

impl Suit {
    pub fn character(self) -> u8 {
        self as u8
    }
}
