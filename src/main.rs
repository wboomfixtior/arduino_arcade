#![no_std]
#![no_main]
#![feature(abi_avr_interrupt)]

pub mod game;
pub mod lcd;
pub mod rng;
pub mod time;
pub mod utils;

use arduino_hal::{hal::port::*, prelude::_unwrap_infallible_UnwrapInfallible};
use avr_device::interrupt;
use panic_halt as _;
use ufmt::uwriteln;

use crate::{
    game::Game,
    lcd::{
        characters,
        options::{FontSize, NumLines},
        LCDInfo,
    },
};

/// That's too many to type out all the time
pub type LCD = lcd::LCD<PB4, PB2, PB3, PD5, PD4, PD3, PD2>;

// NOTE: For wboom's hardware

// pub type LCD = lcd::LCD<PD4, PD5, PD6, PB1, PB2, PB3, PB4>;

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

    let mut adc = arduino_hal::Adc::new(dp.ADC, Default::default());
    rng::set_seed(pins.a5.into_analog_input(&mut adc).analog_read(&mut adc) as u32);

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

    // NOTE: For wboom's hardware

    // let mut lcd = LCD {
    //     register_select: pins.d4.into_output(),
    //     read_write: pins.d5.into_output(),
    //     enable: pins.d6.into_output(),
    //
    //     data_4: pins.d9.into_output(),
    //     data_5: pins.d10.into_output(),
    //     data_6: pins.d11.into_output(),
    //     data_7: pins.d12.into_output(),
    //
    //     info: LCDInfo::new(16, NumLines::Two, FontSize::Dots5x8),
    // };

    lcd.begin();

    characters::load_character_set(&mut lcd, 0);

    let mut deficit = 0u8;

    let joystick_x = pins.a3.into_analog_input(&mut adc);
    let joystick_y = pins.a2.into_analog_input(&mut adc);

    let mut game = Game::new(
        pins.d7.into_pull_up_input(),
        pins.d9.into_pull_up_input(),
        pins.d6.into_pull_up_input(),
        pins.d8.into_pull_up_input(),
    );

    game.draw_full_screen(&mut lcd);

    loop {
        let start = time::millis();

        // Keep the rng from being too predictable
        rng::rng();

        game.joystick_x.update(joystick_x.analog_read(&mut adc));
        game.joystick_y.update(joystick_y.analog_read(&mut adc));

        game.update(&mut lcd);

        let frame_time = if deficit >= DEFICIT_NUMERATOR as u8 {
            FRAME_TIME_MILLISECONDS
        } else {
            FRAME_TIME_MILLISECONDS + 1
        };

        deficit += 1;
        deficit %= DEFICIT_DENOMINATOR as u8;

        let elapsed = time::millis().wrapping_sub(start);

        if elapsed < frame_time {
            arduino_hal::delay_ms(frame_time - elapsed);
        } else if elapsed > frame_time {
            uwriteln!(serial, "Frame too slow: {}ms/{}ms", elapsed, frame_time).unwrap_infallible();
        }
    }
}
