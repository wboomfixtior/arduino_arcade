use crate::{
    game::{block_catch::BlockCatch, overworld::Overworld},
    LCD,
};

pub mod block_catch;
pub mod overworld;
pub mod position;

pub struct Game {
    pub repeat_time: [i8; 2],

    pub overworld: Overworld,
    pub game_mode: GameMode,
}

impl Default for Game {
    fn default() -> Self {
        Self {
            repeat_time: [0; 2],

            overworld: Overworld::default(),
            game_mode: GameMode::Overworld,
        }
    }
}

pub enum GameMode {
    Overworld,
    BlockCatch(BlockCatch),
}

impl Game {
    pub const REPEAT_DELAY_FRAMES: i8 = 15;

    pub fn start(&mut self, lcd: &mut LCD) {
        self.overworld.start(lcd);
    }

    pub fn update(&mut self, lcd: &mut LCD, raw_input: [i8; 2]) {
        let soft_input = self.update_soft_input(raw_input);

        let new_mode = match &mut self.game_mode {
            GameMode::Overworld => self.overworld.update(lcd, raw_input, soft_input),
            GameMode::BlockCatch(block_catch) => block_catch.update(lcd, raw_input),
        };

        if let Some(mode) = new_mode {
            self.game_mode = mode;

            match &mut self.game_mode {
                GameMode::Overworld => self.overworld.start(lcd),
                GameMode::BlockCatch(block_catch) => block_catch.start(lcd),
            }
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
}
