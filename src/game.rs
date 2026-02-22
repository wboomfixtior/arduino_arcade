use ufmt::uwrite;

use crate::LCD;

pub mod overworld;

pub struct Game {}

impl Default for Game {
    fn default() -> Self {
        Self {}
    }
}

impl Game {
    pub fn update(&mut self, lcd: &mut LCD, input: [i8; 2]) {
        lcd.set_cursor(0, 0);
        uwrite!(lcd.fmt(), "{:?}  ", input).unwrap();

        lcd.set_cursor(0, 1);
        lcd.print_bytes(b"Hello world!\0\x01\x02");
    }
}
