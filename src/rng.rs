use core::cell::Cell;

use avr_device::interrupt::{self, Mutex};

static RNG_SEED: Mutex<Cell<u32>> = Mutex::new(Cell::new(0));

pub fn set_seed(seed: u32) {
    interrupt::free(|cs| {
        RNG_SEED.borrow(cs).set(seed);
    });
}

pub fn rng() -> u32 {
    interrupt::free(|cs| {
        let state_cell = RNG_SEED.borrow(cs);
        let mut state = state_cell.get();

        // CREDIT: Wikipedia <https://en.wikipedia.org/wiki/Xorshift#Example_implementation>
        // Algorithm "xor" from p. 4 of Marsaglia, "Xorshift RNGs"
        state ^= state << 13;
        state ^= state >> 17;
        state ^= state << 5;

        state_cell.set(state);
        state
    })
}
