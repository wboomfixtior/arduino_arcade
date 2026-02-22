#![no_std]
#![no_main]
#![feature(abi_avr_interrupt)]

pub mod characters;
pub mod game;
pub mod lcd;
pub mod time;

use arduino_hal::hal::port::{PB2, PB3, PB4, PD2, PD3, PD4, PD5};
use avr_device::interrupt;
use panic_halt as _;
use ufmt::uWrite;

use crate::{
    characters::CHARACTERS,
    game::Game,
    lcd::{
        options::{FontSize, NumLines},
        LCDInfo,
    },
};

/// That's too many to type out all the time
pub type LCD = lcd::LCD<PB4, PB2, PB3, PD5, PD4, PD3, PD2>;

pub const FPS: u32 = 60;
pub const FRAME_TIME_MILLISECONDS: u32 = 1000 / FPS;
pub const TOTAL_FRAME_TIME_DEFICIT: u32 = 1000 - FRAME_TIME_MILLISECONDS * FPS;
pub const DEFICIT_NUMERATOR: u32 = 2;
pub const DEFICIT_DENOMINATOR: u32 = 3;
const _: () = assert!(DEFICIT_NUMERATOR * FPS == TOTAL_FRAME_TIME_DEFICIT * DEFICIT_DENOMINATOR);

#[arduino_hal::entry]
fn main() -> ! {
    let dp = arduino_hal::Peripherals::take().unwrap();
    let pins = arduino_hal::pins!(dp);
    let mut serial = arduino_hal::default_serial!(dp, pins, 57600);

    time::millis_init(dp.TC0);
    unsafe { interrupt::enable() };

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

    lcd.set_cursor(0, 1);
    lcd.print(TOTAL_FRAME_TIME_DEFICIT);

    let mut deficit = 0u8;

    let mut game = Game::default();

    loop {
        let start = time::millis();

        game.update(&mut lcd);

        let frame_time = if deficit >= DEFICIT_NUMERATOR as u8 {
            FRAME_TIME_MILLISECONDS
        } else {
            FRAME_TIME_MILLISECONDS + 1
        };

        deficit += 1;
        deficit %= DEFICIT_DENOMINATOR as u8;

        let elapsed = time::millis() - start;

        if elapsed < frame_time {
            arduino_hal::delay_ms(frame_time - elapsed);
        } else {
            serial.write_str("Frame too long").unwrap();
        }
    }
}
