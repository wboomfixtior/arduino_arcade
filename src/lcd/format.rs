use core::convert::Infallible;

use arduino_hal::{port::PinOps, prelude::_unwrap_infallible_UnwrapInfallible};
use ufmt::{uDisplay, uWrite, uwrite};

use crate::lcd::LCD;

impl<RS: PinOps, RW: PinOps, E: PinOps, D4: PinOps, D5: PinOps, D6: PinOps, D7: PinOps>
    LCD<RS, RW, E, D4, D5, D6, D7>
{
    pub fn print_bytes(&mut self, bytes: &[u8]) {
        for &byte in bytes {
            self.write(byte);
        }
    }

    pub fn print(&mut self, value: impl uDisplay) {
        uwrite!(self.fmt(), "{}", value).unwrap_infallible();
    }

    pub fn fmt(&'_ mut self) -> FnWriter<impl FnMut(u8) + '_> {
        FnWriter::new(|byte| self.write(byte))
    }
}

#[derive(Debug)]
pub struct FnWriter<F: FnMut(u8)> {
    function: F,
}

impl<F: FnMut(u8)> FnWriter<F> {
    pub fn new(function: F) -> Self {
        FnWriter { function }
    }
}

impl<F: FnMut(u8)> uWrite for FnWriter<F> {
    type Error = Infallible;

    fn write_str(&mut self, string: &str) -> Result<(), Self::Error> {
        for byte in string.bytes() {
            (self.function)(byte);
        }
        Ok(())
    }
}
