use arduino_hal::port::PinOps;

use crate::lcd::LCD;

impl<RS: PinOps, RW: PinOps, E: PinOps, D4: PinOps, D5: PinOps, D6: PinOps, D7: PinOps>
    LCD<RS, RW, E, D4, D5, D6, D7>
{
    pub fn print(&mut self, text: &str) {
        for byte in text.bytes() {
            self.write(byte);
        }
    }
}
