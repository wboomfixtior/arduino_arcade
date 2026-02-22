use crate::LCD;

#[rustfmt::skip]
pub const ARCADE: [[&[u8]; 2]; 2] = [
    [
        b"  \0 [1]     {3} ",
        b"        (2)     ",
    ],
    [
        b"     \x7f5\x7e     \xff7\xff",
        b" :4:     <6>    ",
    ]
];

pub fn print_screen(lcd: &mut LCD, screen: usize) {
    for (i, line) in ARCADE[screen].iter().enumerate() {
        lcd.set_cursor(0, i as u8);
        lcd.print_bytes(line);
    }
}
