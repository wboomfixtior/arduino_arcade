#![no_std]
#![no_main]

pub mod characters;
pub mod lcd;
pub mod overworld;

use panic_halt as _;

use crate::{
    characters::CHARACTERS,
    lcd::{
        options::{FontSize, NumLines},
        LCDInfo, LCD,
    },
};

#[arduino_hal::entry]
fn main() -> ! {
    let dp = arduino_hal::Peripherals::take().unwrap();
    let pins = arduino_hal::pins!(dp);
    // let mut serial = arduino_hal::default_serial!(dp, pins, 57600);

    let mut lcd = LCD {
        register_select: pins.d12.into_output(),
        read_write: pins.d10.into_output(),
        enable: pins.d11.into_output(),

        data_4: pins.d5.into_output(),
        data_5: pins.d4.into_output(),
        data_6: pins.d3.into_output(),
        data_7: pins.d2.into_output(),

        info: LCDInfo::new(16, NumLines::Two, FontSize::Dots5x8),
    };

    lcd.begin();

    for (i, character) in CHARACTERS.iter().enumerate() {
        lcd.create_character(i as u8, character);
    }

    lcd.set_cursor(0, 0);
    lcd.print("Hello world!");

    lcd.set_cursor(4, 1);
    for i in 0..8 {
        lcd.write(i);
    }

    loop {
        arduino_hal::delay_ms(1000);
    }
}
