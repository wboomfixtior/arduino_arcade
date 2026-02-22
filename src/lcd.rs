//! CREDIT: LiquidCrystal library for arduino <https://github.com/arduino-libraries/LiquidCrystal>

pub mod characters;
pub mod format;
pub mod options;

use arduino_hal::port::{mode::Output, Pin, PinOps};
use embedded_hal::digital::OutputPin;

use crate::lcd::options::{
    commands, BlinkEnabled, BusSize, CursorEnabled, DisplayEnabled, EntryDirection, EntryShift,
    FontSize, NumLines, Register,
};

pub struct LCD<RS: PinOps, RW: PinOps, E: PinOps, D4: PinOps, D5: PinOps, D6: PinOps, D7: PinOps> {
    pub register_select: Pin<Output, RS>,
    pub read_write: Pin<Output, RW>,
    pub enable: Pin<Output, E>,

    pub data_4: Pin<Output, D4>,
    pub data_5: Pin<Output, D5>,
    pub data_6: Pin<Output, D6>,
    pub data_7: Pin<Output, D7>,

    pub info: LCDInfo,
}

pub struct LCDInfo {
    num_lines: NumLines,
    row_offsets: [u8; 4],

    bus_size: BusSize,
    font_size: FontSize,

    display_enabled: DisplayEnabled,
    cursor_enabled: CursorEnabled,
    blink_enabled: BlinkEnabled,
}

impl LCDInfo {
    pub fn new(line_width: u8, num_lines: NumLines, font_size: FontSize) -> Self {
        Self {
            num_lines,
            row_offsets: [0x00, 0x40, 0x00 + line_width, 0x40 + line_width],

            bus_size: BusSize::Half,
            font_size,

            display_enabled: DisplayEnabled::On,
            cursor_enabled: CursorEnabled::Off,
            blink_enabled: BlinkEnabled::Off,
        }
    }
}

impl<RS: PinOps, RW: PinOps, E: PinOps, D4: PinOps, D5: PinOps, D6: PinOps, D7: PinOps>
    LCD<RS, RW, E, D4, D5, D6, D7>
{
    pub fn begin(&mut self) {
        arduino_hal::delay_ms(50);

        self.register_select.set_low();
        self.read_write.set_low();
        self.enable.set_low();

        self.write_4_bits(0x03);
        arduino_hal::delay_us(4500);

        self.write_4_bits(0x03);
        arduino_hal::delay_us(4500);

        self.write_4_bits(0x03);
        arduino_hal::delay_us(150);

        self.write_4_bits(0x02);

        self.command(
            commands::FUNCTION_SET
                | self.info.bus_size as u8
                | self.info.num_lines as u8
                | self.info.font_size as u8,
        );

        self.update_display_control();

        self.clear();

        self.command(
            commands::ENTRY_MODE_SET | EntryDirection::Left as u8 | EntryShift::Decrement as u8,
        );
    }

    pub fn clear(&mut self) {
        self.command(commands::CLEAR_DISPLAY);
        arduino_hal::delay_ms(2);
    }

    pub fn home(&mut self) {
        self.command(commands::RETURN_HOME);
        arduino_hal::delay_ms(2);
    }

    pub fn set_cursor(&mut self, column: u8, row: u8) {
        let row = row
            .min(self.info.row_offsets.len() as u8 - 1)
            .min(self.info.num_lines.as_count() - 1);

        self.command(commands::SET_DDRAM_ADDRESS | (column + self.info.row_offsets[row as usize]));
    }

    pub fn set_display_control(
        &mut self,
        display_enabled: DisplayEnabled,
        cursor_enabled: CursorEnabled,
        blink_enabled: BlinkEnabled,
    ) {
        self.info.display_enabled = display_enabled;
        self.info.cursor_enabled = cursor_enabled;
        self.info.blink_enabled = blink_enabled;

        self.update_display_control();
    }

    pub fn update_display_control(&mut self) {
        self.command(
            commands::DISPLAY_CONTROL
                | self.info.display_enabled as u8
                | self.info.cursor_enabled as u8
                | self.info.blink_enabled as u8,
        );
    }

    pub fn create_character(&mut self, slot: u8, character: &[u8; 8]) {
        let slot = slot % 8;
        self.command(commands::SET_CGRAM_ADDRESS | slot << 3);
        for &row in character {
            self.write(row);
        }
    }

    pub fn command(&mut self, byte: u8) {
        self.send(Register::Instruction, byte);
    }

    pub fn write(&mut self, byte: u8) {
        self.send(Register::Data, byte);
    }

    fn send(&mut self, register: Register, byte: u8) {
        let _ = self.register_select.set_state(register.into());
        self.read_write.set_low();

        self.write_4_bits(byte >> 4);
        self.write_4_bits(byte);
    }

    fn pulse_enable_pin(&mut self) {
        self.enable.set_low();
        arduino_hal::delay_us(1);
        self.enable.set_high();
        arduino_hal::delay_us(1);
        self.enable.set_low();
        arduino_hal::delay_us(100);
    }

    fn write_4_bits(&mut self, nibble: u8) {
        let _ = self.data_4.set_state((nibble >> 0 & 1 != 0).into());
        let _ = self.data_5.set_state((nibble >> 1 & 1 != 0).into());
        let _ = self.data_6.set_state((nibble >> 2 & 1 != 0).into());
        let _ = self.data_7.set_state((nibble >> 3 & 1 != 0).into());

        self.pulse_enable_pin();
    }
}
