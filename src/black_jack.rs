use core::{mem, num::NonZeroU8};

use arduino_hal::prelude::_unwrap_infallible_UnwrapInfallible;
use ufmt::uwrite;

use crate::{
    game::{position::Position, GameMode},
    utils::List,
    LCD,
};

pub struct BlackJack {
    pub num_player_cards: u8,
    pub player_points: u8,

    pub num_dealer_cards: u8,
    pub dealer_points: u8,

    pub player_position: u8,
    pub countdown: Option<Countdown>,

    pub deck: Deck<52>,
}

#[derive(Copy, Clone)]
pub enum Countdown {
    Standing(u8),
    Waiting(u8),
    Exiting(u8),
}

impl Countdown {
    pub fn standing() -> Self {
        Self::Standing(30)
    }

    pub fn waiting() -> Self {
        Self::Waiting(90)
    }

    pub fn exiting() -> Self {
        Self::Exiting(150)
    }
}

impl Default for BlackJack {
    fn default() -> Self {
        Self {
            num_player_cards: 0,
            player_points: 0,

            num_dealer_cards: 0,
            dealer_points: 0,

            player_position: 0,
            countdown: None,

            deck: Deck::full(),
        }
    }
}

const _: () = assert!(BlackJack::MAXIMUM_CARDS <= 52);

impl BlackJack {
    pub const PLAYER_CHARACTER: u8 = 0;

    pub const TABLE_WIDTH: u8 = 12;
    pub const MAXIMUM_CARDS: u8 = Self::TABLE_WIDTH - 1;

    pub const TABLE_START_COLUMN: u8 = 4;

    pub fn draw_full_screen(&mut self, lcd: &mut LCD) {
        lcd.clear();

        self.add_player_card(lcd);
        self.add_player_card(lcd);
        self.add_dealer_card(lcd);

        lcd.set_cursor(Position::new(self.player_position, 0));
        lcd.write(Self::PLAYER_CHARACTER);
        lcd.set_cursor(Position::new(0, 1));
        lcd.print_bytes(b"HS");

        self.draw_scores(lcd);
    }

    pub fn update(
        &mut self,
        lcd: &mut LCD,
        _raw_input: [i8; 2],
        soft_input: [i8; 2],
    ) -> Option<GameMode> {
        if let Some(countdown) = &mut self.countdown {
            let done = match countdown {
                Countdown::Waiting(time) | Countdown::Exiting(time) | Countdown::Standing(time) => {
                    *time = time.saturating_sub(1);
                    *time == 0
                }
            };

            if done {
                match countdown {
                    Countdown::Standing(_) => self.stand(lcd),
                    Countdown::Waiting(_) => {
                        self.countdown = Some(Countdown::exiting());

                        lcd.print_multiline(
                            Position::new(4, 0),
                            if self.player_points > 21 {
                                "Player\nbusted!"
                            } else if self.dealer_points > 21 {
                                "Dealer\nbusted!"
                            } else if self.player_won() {
                                "Player\nwon!"
                            } else {
                                "Dealer\nwon!"
                            },
                        );
                    }
                    Countdown::Exiting(_) => {
                        return Some(GameMode::Overworld);
                    }
                }
            }
        } else {
            match soft_input[0] {
                1 => self.set_player_position(lcd, 1),
                -1 => self.set_player_position(lcd, 0),
                _ => (),
            }

            if soft_input[1] == 1 {
                match self.player_position {
                    0 => {
                        self.hit(lcd);
                    }
                    1 => {
                        self.stand(lcd);
                    }
                    _ => unreachable!(),
                }
            }
        }

        None
    }

    pub fn set_player_position(&mut self, lcd: &mut LCD, position: u8) {
        if self.player_position == position {
            return;
        }

        lcd.set_cursor(Position::new(self.player_position, 0));
        lcd.write(b' ');
        if self.player_position + 1 != position {
            lcd.set_cursor(Position::new(position, 0));
        }
        lcd.write(Self::PLAYER_CHARACTER);

        self.player_position = position;
    }

    pub fn draw_scores(&self, lcd: &mut LCD) {
        lcd.set_cursor(Position::new(2, 0));
        uwrite!(lcd.fmt(), "{}", self.player_points).unwrap_infallible();

        lcd.set_cursor(Position::new(2, 1));
        uwrite!(lcd.fmt(), "{}", self.dealer_points).unwrap_infallible();
    }

    pub fn hit(&mut self, lcd: &mut LCD) {
        self.add_player_card(lcd);
        self.draw_scores(lcd);

        if self.player_points > 21 {
            self.countdown = Some(Countdown::waiting());
        }
    }

    pub fn stand(&mut self, lcd: &mut LCD) {
        self.add_dealer_card(lcd);
        self.draw_scores(lcd);

        self.countdown = Some(if self.dealer_points < 17 {
            Countdown::standing()
        } else {
            Countdown::waiting()
        });
    }

    pub fn table_full(&mut self) -> bool {
        self.num_player_cards + self.num_dealer_cards >= Self::MAXIMUM_CARDS
    }

    pub fn add_player_card(&mut self, lcd: &mut LCD) {
        let card = self.deck.remove_random().unwrap();

        if !self.table_full() {
            let index = self.num_player_cards;
            card.draw_at(lcd, index + Self::TABLE_START_COLUMN);
        }

        self.num_player_cards += 1;

        self.player_points += card.number().points(self.player_points);
    }

    pub fn add_dealer_card(&mut self, lcd: &mut LCD) {
        let card = self.deck.remove_random().unwrap();

        self.num_dealer_cards += 1;

        if !self.table_full() {
            let index = Self::TABLE_WIDTH - self.num_dealer_cards;
            card.draw_at(lcd, index + Self::TABLE_START_COLUMN);
        }

        self.dealer_points += card.number().points(self.dealer_points);
    }

    pub fn player_won(&self) -> bool {
        self.player_points <= 21
            && (self.player_points > self.dealer_points
                || self.dealer_points > 21
                || (self.player_points == 21 && self.num_player_cards == 2)
                    && !(self.dealer_points == 21 && self.dealer_points == 2))
    }
}

pub type Deck<const N: usize> = List<Card, N>;

impl Deck<52> {
    pub fn full() -> Self {
        let mut deck = Self::new();

        for suit_bits in 1..4 + 1 {
            for number_bits in (0x00..0xc0 + 0x10).step_by(0x10) {
                deck.insert(unsafe { Card::from_bits_unchecked(number_bits | suit_bits) })
                    .unwrap();
            }
        }

        assert_eq!(deck.len(), 52);

        deck
    }
}

/// INVARIANT: Must contain the a `Number`'s bits bitwise or'ed with a `Suit`'s bits
#[derive(Copy, Clone, Eq, PartialEq)]
pub struct Card(NonZeroU8);

impl Card {
    pub fn new(number: Number, suit: Suit) -> Self {
        // SAFETY: `suit as u8` is always nonzero
        Self(unsafe { NonZeroU8::new_unchecked(number as u8 | suit as u8) })
    }

    pub unsafe fn from_bits_unchecked(bits: u8) -> Self {
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
    Number2 = 0x10,
    Number3 = 0x20,
    Number4 = 0x30,
    Number5 = 0x40,
    Number6 = 0x50,
    Number7 = 0x60,
    Number8 = 0x70,
    Number9 = 0x80,
    Number10 = 0x90,
    Jack = 0xa0,
    Queen = 0xb0,
    King = 0xc0,
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

    pub fn points(self, current_score: u8) -> u8 {
        match self {
            Number::Ace => {
                if current_score > 10 {
                    1
                } else {
                    11
                }
            }
            Number::Number2 => 2,
            Number::Number3 => 3,
            Number::Number4 => 4,
            Number::Number5 => 5,
            Number::Number6 => 6,
            Number::Number7 => 7,
            Number::Number8 => 8,
            Number::Number9 => 9,
            Number::Number10 | Number::Jack | Number::Queen | Number::King => 10,
        }
    }
}

impl Suit {
    pub fn character(self) -> u8 {
        self as u8
    }
}
