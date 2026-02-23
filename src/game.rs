use crate::{
    game::{
        black_jack::BlackJack, block_catch::BlockCatch, overworld::Overworld, sokoban::Sokoban,
    },
    LCD,
};

pub mod black_jack;
pub mod block_catch;
pub mod overworld;
pub mod position;
pub mod sokoban;

pub struct Game {
    pub repeat_time: [i8; 2],

    pub overworld: Overworld,
    pub game_mode: GameMode,

    pub high_scores: [u32; 6],
}

impl Default for Game {
    fn default() -> Self {
        Self {
            repeat_time: [0; 2],

            overworld: Overworld::default(),
            game_mode: GameMode::Overworld,

            high_scores: [0; 6],
        }
    }
}

pub enum GameMode {
    Overworld,
    BlockCatch(BlockCatch),
    BlackJack(BlackJack),
    Sokoban(Sokoban),
}

impl GameMode {
    pub fn high_score_slot(&self) -> Option<u8> {
        Some(match self {
            GameMode::Overworld => return None,
            GameMode::BlockCatch(_) => 0,
            GameMode::BlackJack(_) => 1,
            GameMode::Sokoban(_) => return None,
        })
    }
}

impl Game {
    pub const REPEAT_DELAY_FRAMES: i8 = 15;

    pub fn draw_full_screen(&mut self, lcd: &mut LCD) {
        match &mut self.game_mode {
            GameMode::Overworld => self.overworld.draw_full_screen(lcd, &self.high_scores),
            GameMode::BlockCatch(block_catch) => block_catch.draw_full_screen(lcd),
            GameMode::BlackJack(black_jack) => black_jack.draw_full_screen(lcd),
            GameMode::Sokoban(sokoban) => sokoban.draw_full_screen(lcd),
        }
    }

    pub fn update(&mut self, lcd: &mut LCD, raw_input: [i8; 2]) {
        let soft_input = self.update_soft_input(raw_input);

        let new_mode = match &mut self.game_mode {
            GameMode::Overworld => {
                self.overworld
                    .update(lcd, raw_input, soft_input, &self.high_scores)
            }
            GameMode::BlockCatch(block_catch) => block_catch.update(lcd, raw_input),
            GameMode::BlackJack(black_jack) => black_jack.update(lcd, raw_input, soft_input),
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

    pub fn update_soft_input(&mut self, input: [i8; 2]) -> [i8; 2] {
        [0, 1].map(|i| {
            if input[i] > 0 {
                if self.repeat_time[i] > 0 {
                    self.repeat_time[i] -= 1;
                } else {
                    self.repeat_time[i] = Self::REPEAT_DELAY_FRAMES;
                    return 1;
                }
            } else if input[i] < 0 {
                if self.repeat_time[i] < 0 {
                    self.repeat_time[i] += 1;
                } else {
                    self.repeat_time[i] = -Self::REPEAT_DELAY_FRAMES;
                    return -1;
                }
            } else {
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
            GameMode::Sokoban(_) => 0,
        }
    }
}
