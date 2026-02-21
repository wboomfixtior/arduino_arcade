#![no_std]
#![no_main]

pub mod characters;
pub mod overworld;

use ag_lcd::{Blink, Cursor, Display, LcdDisplay, Lines};
use panic_halt as _;
// use ufmt::uWrite;

#[arduino_hal::entry]
fn main() -> ! {
    let dp = arduino_hal::Peripherals::take().unwrap();
    let pins = arduino_hal::pins!(dp);
    let delay = arduino_hal::Delay::new();
    // let mut serial = arduino_hal::default_serial!(dp, pins, 57600);

    let rs = pins.d12.into_output().downgrade();
    let rw = pins.d10.into_output().downgrade();
    let en = pins.d11.into_output().downgrade();

    let d4 = pins.d5.into_output().downgrade();
    let d5 = pins.d4.into_output().downgrade();
    let d6 = pins.d3.into_output().downgrade();
    let d7 = pins.d2.into_output().downgrade();

    let mut lcd = LcdDisplay::new(rs, en, delay)
        .with_half_bus(d4, d5, d6, d7)
        .with_rw(rw)
        .with_display(Display::On)
        .with_blink(Blink::Off)
        .with_cursor(Cursor::Off)
        .with_lines(Lines::TwoLines)
        .build();

    for (i, &character) in characters::CHARACTERS.iter().enumerate() {
        lcd.set_character(i as u8, character);
    }

    loop {
        lcd.clear();

        lcd.set_position(0, 0);
        lcd.print("Hello world!");
        lcd.write(0);

        let mut counter = 0;

        lcd.set_position(4, 1);
        for _ in 0..8 {
            lcd.write(counter);
            counter += 1;
        }
        arduino_hal::delay_ms(1000);
    }
}
