use arduino_hal::port::PinOps;

use crate::lcd::LCD;

impl<RS: PinOps, RW: PinOps, E: PinOps, D4: PinOps, D5: PinOps, D6: PinOps, D7: PinOps>
    LCD<RS, RW, E, D4, D5, D6, D7>
{
    pub fn print(&mut self, value: impl Display) {
        let _ = write!(self.fmt(), "{value}");
    }

    pub fn fmt(&'_ mut self) -> FnWriter<impl FnMut(u8) + '_> {
        FnWriter::new(|byte| self.write(byte))
    }
}

use core::fmt::{self, Display, Write};

#[derive(Debug)]
pub struct FnWriter<F: FnMut(u8)> {
    function: F,
}

impl<F: FnMut(u8)> FnWriter<F> {
    pub fn new(function: F) -> Self {
        FnWriter { function }
    }
}

impl<F: FnMut(u8)> Write for FnWriter<F> {
    fn write_str(&mut self, string: &str) -> fmt::Result {
        for byte in string.bytes() {
            (self.function)(byte);
        }
        Ok(())
    }
}
