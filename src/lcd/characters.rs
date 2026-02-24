use crate::LCD;

pub const CHARACTERS: [[u8; 8]; 3 * 8] = parse_characters(include_bytes!("characters.txt"));

/// WARNING: This is very slow (~25ms)
pub fn load_character_set(lcd: &mut LCD, character_set: u8) {
    for i in 8 * character_set..8 * character_set + 8 {
        let character = &CHARACTERS[i as usize];
        lcd.create_character(i % 8, character);
    }
}

pub const fn parse_characters<const N: usize>(file: &[u8]) -> [[u8; 8]; N] {
    let mut characters = [[0u8; 8]; N];
    let mut i = 0;

    let mut character = 0;

    while i < file.len() {
        // Skip label
        loop {
            let byte = file[i];
            if byte == b'\n' {
                assert!(file[i - 1] == b':', "Line must end with a `:`");

                i += 1;
                break;
            }
            i += 1;
        }

        // Read character
        let mut line_number = 0;
        while line_number < 8 {
            assert!(file[i] == b'|', "Line must start with a `|`");
            i += 1;

            // Read line
            let mut line = 0u8;
            let mut length = 0;

            loop {
                let byte = file[i];

                match byte {
                    b' ' => {
                        line <<= 1;
                    }
                    b'#' => {
                        line <<= 1;
                        line |= 1;
                    }
                    b'|' => break,
                    b'\n' => panic!("Line must end with a `|`"),
                    _ => panic!("Line must contain only ` ` or `#` between the `|`s"),
                }

                i += 1;
                length += 1;
            }
            assert!(
                length == 5,
                "Line must contain 5 characters between the `|`s"
            );

            assert!(file[i] == b'|', "Line must end with a `|`");
            i += 1;
            assert!(
                i >= file.len() || file[i] == b'\n',
                "Line must end with a `|`"
            );
            i += 1;

            characters[character][line_number] = line;

            line_number += 1;
        }

        character += 1;

        if character % 8 == 0 {
            assert!(
                i >= file.len() || file[i] == b'\n',
                "An empty line must follow every 8th character"
            );
            i += 1;
        }
    }

    characters
}
