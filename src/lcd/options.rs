use embedded_hal::digital::PinState;

#[allow(unused)]
pub(super) mod commands {
    pub const CLEAR_DISPLAY: u8 = 0x01;
    pub const RETURN_HOME: u8 = 0x02;
    pub const ENTRY_MODE_SET: u8 = 0x04;
    pub const DISPLAY_CONTROL: u8 = 0x08;
    pub const CURSOR_SHIFT: u8 = 0x10;
    pub const FUNCTION_SET: u8 = 0x20;
    pub const SET_CGRAM_ADDRESS: u8 = 0x40;
    pub const SET_DDRAM_ADDRESS: u8 = 0x80;
}

#[derive(Copy, Clone)]
pub enum EntryDirection {
    Right = 0x00,
    Left = 0x02,
}

#[derive(Copy, Clone)]
pub enum EntryShift {
    Increment = 0x01,
    Decrement = 0x00,
}

#[derive(Copy, Clone)]
pub enum DisplayEnabled {
    On = 0x04,
    Off = 0x00,
}

#[derive(Copy, Clone)]
pub enum CursorEnabled {
    On = 0x02,
    Off = 0x00,
}

#[derive(Copy, Clone)]
pub enum BlinkEnabled {
    On = 0x01,
    Off = 0x00,
}

#[derive(Copy, Clone)]
pub enum ShiftTarget {
    Display = 0x08,
    Cursor = 0x00,
}

#[derive(Copy, Clone)]
pub enum ShiftDirection {
    Right = 0x04,
    Left = 0x00,
}

#[derive(Copy, Clone)]
pub enum BusSize {
    // NOTE: We only support a half bus
    // Full = 0x10,
    Half = 0x00,
}

#[derive(Copy, Clone)]
pub enum NumLines {
    Two = 0x08,
    One = 0x00,
}

impl NumLines {
    pub fn as_count(self) -> u8 {
        match self {
            NumLines::Two => 2,
            NumLines::One => 1,
        }
    }
}

#[derive(Copy, Clone)]
pub enum FontSize {
    Dots5x10 = 0x04,
    Dots5x8 = 0x00,
}

#[derive(Copy, Clone)]
pub enum Register {
    Instruction = 0x00,
    Data = 0x01,
}

impl From<Register> for PinState {
    fn from(value: Register) -> Self {
        match value {
            Register::Instruction => PinState::Low,
            Register::Data => PinState::High,
        }
    }
}
