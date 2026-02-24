use embedded_hal::digital::InputPin;

use crate::{
    game::{
        black_jack::BlackJack, block_catch::BlockCatch, overworld::Overworld, sokoban::Sokoban,
        space_shooter::SpaceShooter,
    },
    LCD,
};

pub mod black_jack;
pub mod block_catch;
pub mod overworld;
pub mod position;
pub mod sokoban;
pub mod space_shooter;

pub struct Game<Right: InputPin, Up: InputPin, Left: InputPin, Down: InputPin> {
    pub repeat_time: [i8; 2],

    pub overworld: Overworld,
    pub game_mode: GameMode,

    pub high_scores: [u32; 6],

    pub dpad_right: Debouncer<Right>,
    pub dpad_up: Debouncer<Up>,
    pub dpad_left: Debouncer<Left>,
    pub dpad_down: Debouncer<Down>,
}

pub enum GameMode {
    Overworld,
    BlockCatch(BlockCatch),
    BlackJack(BlackJack),
    SpaceShooter(SpaceShooter),
    Sokoban(Sokoban),
}

impl GameMode {
    pub fn high_score_slot(&self) -> Option<u8> {
        Some(match self {
            GameMode::Overworld => return None,
            GameMode::BlockCatch(_) => 0,
            GameMode::BlackJack(_) => 1,
            GameMode::SpaceShooter(_) => 2,
            GameMode::Sokoban(_) => return None,
        })
    }
}

impl<Right: InputPin, Up: InputPin, Left: InputPin, Down: InputPin> Game<Right, Up, Left, Down> {
    pub const REPEAT_DELAY_FRAMES: i8 = 15;

    pub fn new(dpad_right: Right, dpad_up: Up, dpad_left: Left, dpad_down: Down) -> Self {
        Self {
            repeat_time: [0; 2],

            overworld: Overworld::default(),
            // DEBUG:
            // game_mode: GameMode::Overworld,
            game_mode: GameMode::SpaceShooter(SpaceShooter::default()),

            high_scores: [0; 6],

            dpad_right: Debouncer::new(dpad_right),
            dpad_up: Debouncer::new(dpad_up),
            dpad_left: Debouncer::new(dpad_left),
            dpad_down: Debouncer::new(dpad_down),
        }
    }

    pub fn draw_full_screen(&mut self, lcd: &mut LCD) {
        match &mut self.game_mode {
            GameMode::Overworld => self.overworld.draw_full_screen(lcd, &self.high_scores),
            GameMode::BlockCatch(block_catch) => block_catch.draw_full_screen(lcd),
            GameMode::BlackJack(black_jack) => black_jack.draw_full_screen(lcd),
            GameMode::SpaceShooter(space_shooter) => space_shooter.draw_full_screen(lcd),
            GameMode::Sokoban(sokoban) => sokoban.draw_full_screen(lcd),
        }
    }

    pub fn update(&mut self, lcd: &mut LCD) {
        let raw_input = self.update_raw_input();
        let soft_input = self.update_soft_input(raw_input);

        let new_mode = match &mut self.game_mode {
            GameMode::Overworld => {
                self.overworld
                    .update(lcd, raw_input, soft_input, &self.high_scores)
            }
            GameMode::BlockCatch(block_catch) => block_catch.update(lcd, raw_input),
            GameMode::BlackJack(black_jack) => black_jack.update(lcd, raw_input, soft_input),
            GameMode::SpaceShooter(space_shooter) => space_shooter.update(lcd, raw_input),
            GameMode::Sokoban(sokoban) => sokoban.update(lcd, raw_input, soft_input),
        };

        if let Some(mode) = new_mode {
            if let Some(slot) = self.game_mode.high_score_slot() {
                let score = self.score();
                let high_score = &mut self.high_scores[slot as usize];

                *high_score = score.max(*high_score);
            }

            self.game_mode = mode;

            self.draw_full_screen(lcd);
        }
    }

    pub fn update_raw_input(&mut self) -> [i8; 2] {
        [
            self.dpad_right.update() as i8 - self.dpad_left.update() as i8,
            self.dpad_down.update() as i8 - self.dpad_up.update() as i8,
        ]
    }

    pub fn get_debounced_input(&mut self) -> [bool; 2] {
        [
            self.dpad_right.is_held() || self.dpad_left.is_held(),
            self.dpad_down.is_held() || self.dpad_up.is_held(),
        ]
    }

    pub fn update_soft_input(&mut self, raw_input: [i8; 2]) -> [i8; 2] {
        let debounced = self.get_debounced_input();

        [0, 1].map(|i| {
            if raw_input[i] > 0 {
                if self.repeat_time[i] > 0 {
                    self.repeat_time[i] -= 1;
                } else {
                    self.repeat_time[i] = Self::REPEAT_DELAY_FRAMES;
                    return 1;
                }
            } else if raw_input[i] < 0 {
                if self.repeat_time[i] < 0 {
                    self.repeat_time[i] += 1;
                } else {
                    self.repeat_time[i] = -Self::REPEAT_DELAY_FRAMES;
                    return -1;
                }
            } else if !debounced[i] {
                self.repeat_time[i] = 0;
            }

            0
        })
    }

    pub fn score(&self) -> u32 {
        let Some(slot) = self.game_mode.high_score_slot() else {
            return 0;
        };

        match &self.game_mode {
            GameMode::Overworld => 0,
            GameMode::BlockCatch(block_catch) => block_catch.score,
            GameMode::BlackJack(black_jack) => {
                if black_jack.player_won() {
                    self.high_scores[slot as usize] + 1
                } else {
                    0
                }
            }
            GameMode::SpaceShooter(space_shooter) => space_shooter.score,
            GameMode::Sokoban(_) => 0,
        }
    }
}

pub struct Debouncer<T: InputPin> {
    pub pin: T,
    pub time_since_release: u8,
}

impl<T: InputPin> Debouncer<T> {
    pub const FALLING_EDGE_TIME: u8 = 5;

    pub fn new(pin: T) -> Self {
        Self {
            pin,
            time_since_release: u8::MAX,
        }
    }

    pub fn update(&mut self) -> bool {
        let raw_value = self.pin.is_low().unwrap();
        if raw_value {
            self.time_since_release = 0;
        } else {
            self.time_since_release = self.time_since_release.saturating_add(1);
        }

        raw_value
    }

    pub fn is_held(&mut self) -> bool {
        self.time_since_release < Self::FALLING_EDGE_TIME
    }
}
