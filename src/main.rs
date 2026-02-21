#![no_std]
#![no_main]

pub mod characters;
pub mod display;
pub mod overworld;

use panic_halt as _;
// use ufmt::uWrite;

use crate::display::LCDDisplay;

#[arduino_hal::entry]
fn main() -> ! {
    let dp = arduino_hal::Peripherals::take().unwrap();
    let pins = arduino_hal::pins!(dp);
    // let mut serial = arduino_hal::default_serial!(dp, pins, 57600);

    let mut display = LCDDisplay {
        register_select: pins.d12.into_output(),
        read_write: pins.d10.into_output(),
        enable: pins.d11.into_output(),
        data_0: pins.d5.into_output(),
        data_1: pins.d4.into_output(),
        data_2: pins.d3.into_output(),
        data_3: pins.d2.into_output(),
    };

    // display.set_characters(&characters::CHARACTERS);
    display.begin();

    // display.set_text_cursor(0);
    // display.write_data(b'H');
    // display.write_data(b'i');
    // display.write_data(b'!');

    loop {
        arduino_hal::delay_ms(1000);
    }
}
