use core::{
    mem::{self, MaybeUninit},
    num::NonZeroU8,
    ops::Index,
};

use crate::{game::position::Position, rng, LCD};

/// INVARIANT: All cards with an index less than self.len should be initialized
#[derive(Clone)]
pub struct Deck<const N: usize> {
    len: u8,
    /// SAFETY: Card is Copy so it doesn't need drop
    cards: [MaybeUninit<Card>; N],
}

impl<const N: usize> Deck<N> {
    pub const fn new() -> Self {
        Self {
            len: 0,
            cards: [MaybeUninit::uninit(); N],
        }
    }

    pub fn len(&self) -> u8 {
        self.len
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    #[must_use]
    pub fn insert(&mut self, card: Card) -> Option<()> {
        const {
            assert!(N <= u8::MAX as usize);
        }

        if self.len >= N as u8 {
            return None;
        }

        self.cards[self.len as usize] = MaybeUninit::new(card);
        self.len += 1;

        Some(())
    }

    #[must_use]
    pub fn remove_random(&mut self) -> Option<Card> {
        if self.is_empty() {
            return None;
        }

        let i = (rng::rng() % self.len as u32) as usize;
        self.len -= 1;

        // SAFETY: Card was accessed from an index less than self.len
        let card = unsafe { self.cards[i].assume_init() };
        self.cards[i] = self.cards[self.len as usize];

        Some(card)
    }
}

impl<const N: usize> Default for Deck<N> {
    fn default() -> Self {
        Self::new()
    }
}

impl Deck<52> {
    pub fn full() -> Self {
        let mut deck = Self::new();

        for suit_bits in 1..4 + 1 {
            for number_bits in (0x00..0xd0 + 0x10).step_by(0x10) {
                deck.insert(unsafe { Card::from_bits(number_bits | suit_bits) })
                    .unwrap();
            }
        }

        deck
    }
}

impl<const N: usize> Index<u8> for Deck<N> {
    type Output = Card;

    fn index(&self, index: u8) -> &Self::Output {
        assert!(index < N as u8);
        unsafe { self.cards[index as usize].assume_init_ref() }
    }
}

/// INVARIANT: Must contain the a `Number`'s bits bitwised or a `Suit`'s bits
#[derive(Copy, Clone, Eq, PartialEq)]
pub struct Card(NonZeroU8);

impl Card {
    pub fn new(number: Number, suit: Suit) -> Self {
        // SAFETY: `suit as u8` is always nonzero
        Self(unsafe { NonZeroU8::new_unchecked(number as u8 | suit as u8) })
    }

    pub unsafe fn from_bits(bits: u8) -> Self {
        unsafe { Self(NonZeroU8::new_unchecked(bits)) }
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
