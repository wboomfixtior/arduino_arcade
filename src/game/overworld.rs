use crate::{
    game::{
        black_jack::BlackJack, block_catch::BlockCatch, position::Position, sokoban::Sokoban,
        space_shooter::SpaceShooter, GameMode,
    },
    utils, LCD,
};

#[rustfmt::skip]
pub const ARCADE: [[&[u8]; 2]; 2] = [
    [
        b"  \x05 [1]     =3= ",
        b"        \x032\x03    \x7e",
    ],
    [
        b" :4:     <6>    ",
        b"\x7f    {5}     \xff7\xff",
    ]
];

const _: () = {
    let mut i = 0;
    while i < ARCADE.len() {
        let mut j = 0;
        while j < ARCADE[0].len() {
            assert!(
                ARCADE[i][j].len() == 16,
                "The width of each line should match the screen's width",
            );

            j += 1;
        }

        i += 1;
    }
};

pub struct Overworld {
    pub screen: u8,

    pub player_position: Position,
    pub score_length: Option<u8>,
}

impl Default for Overworld {
    fn default() -> Self {
        Self {
            screen: 0,

            player_position: Position::new(0, 0),
            score_length: None,
        }
    }
}

impl Overworld {
    pub const PLAYER_CHARACTER: u8 = 0;
    pub const SCORE_SYMBOLS: [u8; 7] = [0x06, b'W', 0x06, b'?', b'?', b'?', b'?'];

    pub fn draw_full_screen(&mut self, lcd: &mut LCD, scores: &[u32; 6]) {
        self.print_screen(lcd);

        lcd.set_cursor(self.player_position);
        lcd.write(Self::PLAYER_CHARACTER);

        self.update_high_score_display(lcd, scores);
    }

    pub fn update(
        &mut self,
        lcd: &mut LCD,
        _raw_input: [i8; 2],
        soft_input: [i8; 2],
        scores: &[u32; 6],
    ) -> Option<GameMode> {
        let old_position = self.player_position;

        match self.move_player_by(lcd, soft_input, scores) {
            Some(b'1') => Some(GameMode::BlockCatch(BlockCatch::default())),
            Some(b'2') => Some(GameMode::BlackJack(BlackJack::default())),
            Some(b'3') => Some(GameMode::SpaceShooter(SpaceShooter::default())),
            Some(b'4') => Some(GameMode::Sokoban(Sokoban::default())),
            _ => 'outer: {
                if self.player_position == old_position {
                    break 'outer None;
                }

                if let Some(length) = self.score_length {
                    self.score_length = None;
                    lcd.set_cursor(old_position.nudge_column_saturating(-(length as i8)));
                    let range = (old_position.column().saturating_sub(length)) as usize
                        ..self.player_position.column() as usize;
                    lcd.print_bytes(
                        &ARCADE[self.screen as usize][old_position.row() as usize][range],
                    );
                }

                self.update_high_score_display(lcd, scores);

                None
            }
        }
    }

    pub fn update_high_score_display(&mut self, lcd: &mut LCD, scores: &[u32; 6]) {
        let opposite_position = self
            .player_position
            .with_row(1 - self.player_position.row());

        match self.get_tile_at(opposite_position) {
            tile @ b'1'..=b'7' if tile != b'4' => {
                let mut slot = tile - b'1';
                if tile > b'4' {
                    slot -= 1;
                }
                self.draw_score(lcd, scores, slot);
            }
            _ => (),
        }
    }

    pub fn draw_score(&mut self, lcd: &mut LCD, scores: &[u32; 6], slot: u8) {
        let score = scores[slot as usize];
        if score == 0 {
            return;
        }

        let length = utils::num_digits(score) + 1;

        let start = self
            .player_position
            .nudge_column_saturating(-(length as i8));

        let actual_length = self.player_position.column() - start.column();

        lcd.set_cursor(start);
        lcd.write(Self::SCORE_SYMBOLS[slot as usize]);
        if actual_length == length {
            lcd.print(score);
        } else {
            for _ in 1..actual_length {
                lcd.write(b'9');
            }
        }

        self.score_length = Some(length);
    }

    pub fn move_player_by(
        &mut self,
        lcd: &mut LCD,
        input: [i8; 2],
        scores: &[u32; 6],
    ) -> Option<u8> {
        let mut new_position = self.player_position;

        match input[0] {
            input @ (1 | -1) => {
                let position = new_position.nudge_column_saturating(input);

                if input == 1 {
                    if self.screen < ARCADE.len() as u8 - 1
                        && position.column() == Position::COLUMN_MASK
                    {
                        // NOTE: We assume the target tile to be valid
                        self.screen += 1;
                        self.player_position = new_position.with_column(0);
                        self.draw_full_screen(lcd, scores);
                        return None;
                    } else {
                        new_position = position;
                    }
                } else {
                    if self.screen > 0 && position.column() == 0 {
                        // NOTE: We assume the target tile to be valid
                        self.screen -= 1;
                        self.player_position = new_position.with_column(Position::COLUMN_MASK);
                        self.draw_full_screen(lcd, scores);
                        return None;
                    } else {
                        new_position = position;
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
        if !b" \x7e\x7f".contains(&target) {
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
