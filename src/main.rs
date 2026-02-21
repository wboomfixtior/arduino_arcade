#![no_std]
#![no_main]

pub mod characters;
pub mod overworld;

use panic_halt as _;
use ufmt::uWrite;

#[arduino_hal::entry]
fn main() -> ! {
    let dp = arduino_hal::Peripherals::take().unwrap();
    let pins = arduino_hal::pins!(dp);
    let mut serial = arduino_hal::default_serial!(dp, pins, 57600);

    let mut led = pins.d13.into_output();

    loop {
        serial.write_str("Hello world!\n").unwrap();
        led.toggle();
        arduino_hal::delay_ms(1000);
    }
}
