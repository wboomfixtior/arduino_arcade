use arduino_hal::port::{mode::Output, Pin, PinOps};
use embedded_hal::digital::{OutputPin, PinState};

pub struct LCDDisplay<
    RS: PinOps,
    RW: PinOps,
    E: PinOps,
    D0: PinOps,
    D1: PinOps,
    D2: PinOps,
    D3: PinOps,
> {
    pub register_select: Pin<Output, RS>,
    pub read_write: Pin<Output, RW>,
    pub enable: Pin<Output, E>,

    pub data_0: Pin<Output, D0>,
    pub data_1: Pin<Output, D1>,
    pub data_2: Pin<Output, D2>,
    pub data_3: Pin<Output, D3>,
}

#[derive(Copy, Clone, Debug)]
pub enum Register {
    Instruction,
    Data,
}

impl From<Register> for PinState {
    fn from(value: Register) -> Self {
        match value {
            Register::Instruction => PinState::Low,
            Register::Data => PinState::High,
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub enum ShiftDirection {
    Right,
    Left,
}

impl From<ShiftDirection> for bool {
    fn from(value: ShiftDirection) -> Self {
        match value {
            ShiftDirection::Right => true,
            ShiftDirection::Left => false,
        }
    }
}

impl<RS: PinOps, RW: PinOps, E: PinOps, D0: PinOps, D1: PinOps, D2: PinOps, D3: PinOps>
    LCDDisplay<RS, RW, E, D0, D1, D2, D3>
{
    pub fn begin(&mut self) {
        arduino_hal::delay_ms(50);

        self.register_select.set_low();
        self.read_write.set_low();
        self.enable.set_low();

        self.write_raw_nibble(Register::Instruction, 0b0011);
        arduino_hal::delay_us(4500);

        self.write_raw_nibble(Register::Instruction, 0b0011);
        arduino_hal::delay_us(4500);

        self.write_raw_nibble(Register::Instruction, 0b0011);
        arduino_hal::delay_us(150);

        self.write_raw_nibble(Register::Instruction, 0b0010);

        self.set_function(true, false);

        self.set_display(true, false, false);
        self.clear_screen();
        self.set_cursor_advance(ShiftDirection::Right, true);
    }

    pub fn set_characters(&mut self, characters: &[[u8; 8]; 8]) {
        for (i, character) in characters.iter().enumerate() {
            self.set_font_cursor((i as u8) << 3);
            for &line in character {
                self.write_data(line);
            }
        }
    }

    pub fn clear_screen(&mut self) {
        self.write_raw_byte(Register::Instruction, 0b00000001);
        arduino_hal::delay_ms(2);
    }

    pub fn return_cursor(&mut self) {
        self.write_raw_byte(Register::Instruction, 0b00000010);
        arduino_hal::delay_ms(2);
    }

    pub fn set_cursor_advance(&mut self, direction: ShiftDirection, shift_increment: bool) {
        self.write_raw_byte(
            Register::Instruction,
            0b00000100 | (bool::from(direction) as u8) << 1 | shift_increment as u8,
        );
    }

    pub fn set_display(&mut self, display_on: bool, cursor_on: bool, cursor_blink: bool) {
        self.write_raw_byte(
            Register::Instruction,
            0b00001000 | (display_on as u8) << 2 | (cursor_on as u8) << 1 | cursor_blink as u8,
        );
    }

    pub fn shift_screen(&mut self, display_only: bool, direction: ShiftDirection) {
        self.write_raw_byte(
            Register::Instruction,
            0b00010000 | (display_only as u8) << 3 | (bool::from(direction) as u8) << 2,
        );
    }

    /// Always sets data width to 4
    pub fn set_function(&mut self, both_rows: bool, use_extra_font_space: bool) {
        self.write_raw_byte(
            Register::Instruction,
            0b00100000 | (both_rows as u8) << 3 | (use_extra_font_space as u8) << 2,
        );
    }

    pub fn set_font_cursor(&mut self, cursor: u8) {
        self.write_raw_byte(Register::Instruction, 0b01000000 | (0b00111111 & cursor));
    }

    pub fn set_text_cursor(&mut self, cursor: u8) {
        self.write_raw_byte(Register::Instruction, 0b10000000 | (0b01111111 & cursor));
    }

    pub fn write_data(&mut self, data: u8) {
        self.write_raw_byte(Register::Data, data);
    }

    fn write_raw_byte(&mut self, register: Register, byte: u8) {
        self.write_raw_nibble(register, byte >> 4);
        self.write_raw_nibble(register, byte);
    }

    fn write_raw_nibble(&mut self, register: Register, nibble: u8) {
        self.register_select.set_state(register.into()).unwrap();
        self.read_write.set_low();

        let mut data = nibble;
        self.data_0.set_state(((data & 1) != 0).into()).unwrap();
        data >>= 1;
        self.data_1.set_state(((data & 1) != 0).into()).unwrap();
        data >>= 1;
        self.data_2.set_state(((data & 1) != 0).into()).unwrap();
        data >>= 1;
        self.data_3.set_state(((data & 1) != 0).into()).unwrap();

        self.enable.set_low();
        arduino_hal::delay_us(1);

        self.enable.set_high();
        arduino_hal::delay_us(1);

        self.enable.set_low();
        arduino_hal::delay_us(1);
    }
}
