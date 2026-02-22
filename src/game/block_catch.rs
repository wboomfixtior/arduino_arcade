use crate::{game::GameMode, LCD};

pub struct BlockCatch {}

impl Default for BlockCatch {
    fn default() -> Self {
        Self {}
    }
}

impl BlockCatch {
    pub fn start(&mut self, lcd: &mut LCD) {
        lcd.clear();
    }

    pub fn update(&mut self, lcd: &mut LCD, raw_input: [i8; 2]) -> Option<GameMode> {
        None
    }
}
