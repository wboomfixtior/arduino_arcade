use crate::LCD;

pub mod overworld;

pub struct Game {}

impl Default for Game {
    fn default() -> Self {
        Self {}
    }
}

impl Game {
    pub fn update(&mut self, lcd: &mut LCD) {}
}
