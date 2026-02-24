arduino_arcade
==============

This project's skeleton was generated with this `cargo-generate` template: <https://github.com/Rahix/avr-hal-template.git>. 

Rust project for the _Arduino Uno_.

## Build Instructions
1. Install prerequisites as described in the [`avr-hal` README] (`avr-gcc`, `avr-libc`, `avrdude`, [`ravedude`]).

2. Run `cargo build` to build the firmware.

3. Run `cargo run` to flash the firmware to a connected board.  If `ravedude`
   fails to detect your board, check its documentation at
   <https://crates.io/crates/ravedude>.

4. `ravedude` will open a console session after flashing where you can interact
   with the UART console of your board.

[`avr-hal` README]: https://github.com/Rahix/avr-hal#readme
[`ravedude`]: https://crates.io/crates/ravedude

## License
Licensed under the GNU GPL 3.0
   ([LICENSE](LICENSE) or <https://www.gnu.org/licenses/gpl-3.0.en.html>)
