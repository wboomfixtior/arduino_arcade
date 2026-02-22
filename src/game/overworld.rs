use crate::{
    game::{block_catch::BlockCatch, position::Position, GameMode},
    LCD,
};

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

pub struct Overworld {
    pub screen: u8,

    pub player_position: Position,
}

impl Default for Overworld {
    fn default() -> Self {
        Self {
            screen: 0,
            player_position: Position::new(0, 0),
        }
    }
}

impl Overworld {
    pub const PLAYER_CHARACTER: u8 = 0;

    pub fn draw_full_screen(&mut self, lcd: &mut LCD) {
        self.print_screen(lcd);

        lcd.set_cursor(self.player_position);
        lcd.write(Self::PLAYER_CHARACTER);
    }

    pub fn update(
        &mut self,
        lcd: &mut LCD,
        _raw_input: [i8; 2],
        soft_input: [i8; 2],
    ) -> Option<GameMode> {
        match self.move_player_by(lcd, soft_input) {
            Some(b'1') => Some(GameMode::BlockCatch(BlockCatch::default())),
            _ => None,
        }
    }

    pub fn move_player_by(&mut self, lcd: &mut LCD, input: [i8; 2]) -> Option<u8> {
        let mut new_position = self.player_position;

        match input[0] {
            input @ (1 | -1) => {
                let (pos, edge) = new_position.nudge_column_overflowing(input);
                if !edge {
                    new_position = pos;
                } else if input == 1 {
                    if self.screen < ARCADE.len() as u8 - 1 {
                        // NOTE: We assume the target tile to be valid
                        self.screen += 1;
                        self.player_position = new_position.with_column(0);
                        self.draw_full_screen(lcd);
                        return None;
                    }
                } else {
                    if self.screen > 0 {
                        // NOTE: We assume the target tile to be valid
                        self.screen -= 1;
                        self.player_position = new_position.with_column(Position::MAX_COLUMN);
                        self.draw_full_screen(lcd);
                        return None;
                    }
                }
            }
            _ => (),
        }

        match input[1] {
            1 => new_position = new_position.with_row(1),
            -1 => new_position = new_position.with_row(0),
            _ => (),
        }

        if new_position == self.player_position {
            return None;
        }

        let target = self.get_tile_at(new_position);
        if target != b' ' {
            return Some(target);
        }

        lcd.set_cursor(self.player_position);
        lcd.write(self.get_tile_at(self.player_position));

        lcd.set_cursor(new_position);
        lcd.write(Self::PLAYER_CHARACTER);

        self.player_position = new_position;

        None
    }

    pub fn get_tile_at(&self, position: Position) -> u8 {
        ARCADE[self.screen as usize][position.row() as usize][position.column() as usize]
    }

    pub fn print_screen(&self, lcd: &mut LCD) {
        for (i, line) in ARCADE[self.screen as usize].iter().enumerate() {
            lcd.set_cursor(Position::new(0, i as u8));
            lcd.print_bytes(line);
        }
    }
}
