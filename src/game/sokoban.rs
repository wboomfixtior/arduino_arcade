use core::{
    mem,
    ops::{Index, IndexMut},
};

use crate::{
    game::{
        position::{GenericPosition, Position},
        GameMode,
    },
    lcd::characters,
    LCD,
};

const LEVELS: [(LevelPosition, [u32; 4]); 15] = parse_levels(include_bytes!("sokoban.txt"));

pub struct Sokoban {
    pub level_select: LevelSelect,
    pub level: Option<Level>,
}

impl Sokoban {
    pub fn draw_full_screen(&self, lcd: &mut LCD) {
        if let Some(level) = &self.level {
            level.draw_full_screen(lcd);
        } else {
            self.level_select.draw_full_screen(lcd);
        }
    }

    pub fn update(
        &mut self,
        lcd: &mut LCD,
        raw_input: [i8; 2],
        soft_input: [i8; 2],
    ) -> Option<GameMode> {
        if let Some(level) = &mut self.level {
            if let Some(was_success) = level.update(lcd, raw_input, soft_input) {
                self.level = None;
                if was_success {
                    self.level_select.draw_victory(lcd);
                } else {
                    self.level_select.draw_full_screen(lcd);
                }
            }

            None
        } else if let Some(selection) = self.level_select.update(lcd, raw_input, soft_input) {
            match selection {
                LevelSelection::Exit => Some(GameMode::Overworld),
                LevelSelection::PlayLevel(level) => {
                    let level = Level::load(level);
                    level.draw_full_screen(lcd);
                    self.level = Some(level);
                    None
                }
            }
        } else {
            None
        }
    }
}

impl Default for Sokoban {
    fn default() -> Self {
        Self {
            level_select: LevelSelect::default(),
            level: None,
        }
    }
}

pub struct LevelSelect {
    pub player_position: Position,

    pub selection_cooldown: u8,
}

pub enum LevelSelection {
    Exit,
    PlayLevel(u8),
}

impl Default for LevelSelect {
    fn default() -> Self {
        Self {
            player_position: Position::new(0, 1),

            selection_cooldown: 0,
        }
    }
}

impl LevelSelect {
    pub const PLAYER_CHARACTER: u8 = 0;

    pub const SELECTION_COOLDOWN: u8 = 3 * 60;

    pub const BACKGROUND: [u8; 16] = *b"  level select  ";

    pub fn draw_full_screen(&self, lcd: &mut LCD) {
        lcd.clear();
        lcd.set_cursor(Position::new(0, 0));
        lcd.print_bytes(b"\x7fABCDEFGHIJKLMNO");
        lcd.set_cursor(Position::new(0, 1));
        lcd.print_bytes(&Self::BACKGROUND);

        lcd.set_cursor(self.player_position);
        lcd.write(Self::PLAYER_CHARACTER);
    }

    pub fn draw_victory(&mut self, lcd: &mut LCD) {
        lcd.clear();
        lcd.set_cursor(Position::new(0, 0));
        lcd.print_bytes(b"Level complete!");

        self.selection_cooldown = Self::SELECTION_COOLDOWN;
    }

    pub fn update(
        &mut self,
        lcd: &mut LCD,
        _raw_input: [i8; 2],
        soft_input: [i8; 2],
    ) -> Option<LevelSelection> {
        if self.selection_cooldown > 0 {
            self.selection_cooldown -= 1;

            if self.selection_cooldown == 0 {
                self.draw_full_screen(lcd);
            }

            return None;
        }

        match soft_input[0] {
            input @ (1 | -1) => {
                let (new_position, blocked) = self.player_position.nudge_column_overflowing(input);

                if blocked {
                    if input == -1 {
                        return Some(LevelSelection::Exit);
                    }
                } else {
                    let replacement_position =
                        self.player_position.nudge_column_saturating(input * -1);
                    lcd.set_cursor(replacement_position);
                    lcd.write(Self::BACKGROUND[replacement_position.column() as usize]);

                    lcd.set_cursor(self.player_position);
                    lcd.write(b' ');

                    lcd.set_cursor(self.player_position.nudge_column_saturating(input * 2));
                    lcd.write(b' ');

                    lcd.set_cursor(new_position);
                    lcd.write(Self::PLAYER_CHARACTER);

                    self.player_position = new_position;
                }
            }
            _ => (),
        }

        if self.selection_cooldown == 0 && soft_input[1] == -1 {
            return Some(if self.player_position.column() > 0 {
                LevelSelection::PlayLevel(self.player_position.column() - 1)
            } else {
                LevelSelection::Exit
            });
        }

        None
    }
}

pub struct Level {
    pub level: [[Tile; 4]; 16],
    pub player_position: LevelPosition,

    pub num_boxes: u8,

    pub flash_timer: u8,

    pub emergency_exit_timer: u8,
}

impl Level {
    pub const FLASH_PERIOD: u8 = 70;
    pub const FLASH_LENGTH: u8 = 40;

    pub const EMERGENCY_EXIT_TIME: u8 = 60 * 4;

    fn load(level: u8) -> Self {
        let (player_position, level, num_boxes) = decode_level(level);

        Self {
            level,
            player_position,

            num_boxes,

            flash_timer: 0,

            emergency_exit_timer: 0,
        }
    }

    pub fn draw_full_screen(&self, lcd: &mut LCD) {
        characters::load_character_set(lcd, 1);

        self.update_all_tiles(lcd, |_, _| true);
    }

    pub fn update_all_tiles(&self, lcd: &mut LCD, mut condition: impl FnMut(Tile, Tile) -> bool) {
        for row in 0..2 {
            let mut cursor_valid = false;

            for column in 0..16 {
                let position = Position::new(column, row);
                let [top, bottom] = self.pair_of_tile(position);
                if condition(top, bottom) {
                    if !cursor_valid {
                        cursor_valid = true;
                        lcd.set_cursor(position);
                    }
                    lcd.write(self.byte_of_pair(top, bottom));
                } else {
                    cursor_valid = false;
                }
            }
        }
    }

    pub fn update(
        &mut self,
        lcd: &mut LCD,
        raw_input: [i8; 2],
        soft_input: [i8; 2],
    ) -> Option<bool> {
        self.flash_timer = self.flash_timer.saturating_sub(1);
        if self.flash_timer == 0 || self.flash_timer == Self::FLASH_LENGTH {
            self.update_all_tiles(lcd, |top, bottom| {
                top == Tile::Destination || bottom == Tile::Destination
            });
        }

        if self.flash_timer == 0 {
            self.flash_timer = Self::FLASH_PERIOD;
            self.update_all_tiles(lcd, |top, bottom| {
                top == Tile::Destination || bottom == Tile::Destination
            });
        }

        let moved = self.move_player(lcd, soft_input);

        if moved || raw_input != [0, 1] {
            self.emergency_exit_timer = 0;
        } else {
            self.emergency_exit_timer += 1;
        }

        if self.num_boxes == 0
            || !moved && soft_input[0] == -1 && self.player_position.column() == 0
            || self.emergency_exit_timer >= Self::EMERGENCY_EXIT_TIME
        {
            characters::load_character_set(lcd, 0);
            Some(self.num_boxes == 0)
        } else {
            None
        }
    }

    pub fn move_player(&mut self, lcd: &mut LCD, input: [i8; 2]) -> bool {
        self.move_player_on_axis(lcd, LevelPosition::nudge_column_overflowing, input[0])
            | self.move_player_on_axis(lcd, LevelPosition::nudge_row_overflowing, input[1])
    }

    pub fn move_player_on_axis(
        &mut self,
        lcd: &mut LCD,
        mut nudge: impl FnMut(LevelPosition, i8) -> (LevelPosition, bool),
        input: i8,
    ) -> bool {
        match input {
            1 | -1 => 'outer: {
                let (next_position, blocked) = nudge(self.player_position, input);
                let next_tile = self[next_position];
                if !matches!(next_tile, Tile::Empty | Tile::Box) || blocked {
                    break 'outer false;
                }

                if next_tile == Tile::Box {
                    let (lookahead, lookahead_blocked) = nudge(self.player_position, input * 2);

                    if lookahead_blocked {
                        break 'outer false;
                    }

                    match self[lookahead] {
                        Tile::Empty => self[lookahead] = Tile::Box,
                        Tile::Destination => {
                            self.num_boxes = self.num_boxes.saturating_sub(1);
                            self[lookahead] = Tile::BoxOnDestination;
                        }
                        _ => break 'outer false,
                    }

                    self.draw_tile(lcd, lookahead);
                }

                let old_position = self.player_position;
                self.player_position = next_position;

                self[next_position] = Tile::Player;
                self[old_position] = Tile::Empty;

                self.draw_tile(lcd, next_position);
                self.draw_tile(lcd, old_position);

                true
            }
            _ => false,
        }
    }

    pub fn draw_tile(&self, lcd: &mut LCD, tile_position: LevelPosition) {
        let position = Position::new(tile_position.column(), tile_position.row() / 2);
        lcd.set_cursor(position);
        lcd.write(self.byte_of_tile(position));
    }

    pub fn pair_of_tile(&self, position: Position) -> [Tile; 2] {
        let top_tile = LevelPosition::new(position.column(), position.row() * 2);
        let top = self[top_tile];
        let bottom = self[top_tile.with_row(top_tile.row() + 1)];

        [top, bottom]
    }

    pub fn byte_of_pair(&self, top: Tile, bottom: Tile) -> u8 {
        Tile::byte_of_pair(top, bottom, self.flash_timer <= Self::FLASH_LENGTH)
    }

    pub fn byte_of_tile(&self, position: Position) -> u8 {
        let [top, bottom] = self.pair_of_tile(position);
        self.byte_of_pair(top, bottom)
    }
}

impl Index<LevelPosition> for Level {
    type Output = Tile;

    fn index(&self, index: LevelPosition) -> &Self::Output {
        &self.level[index.column() as usize][index.row() as usize]
    }
}

impl IndexMut<LevelPosition> for Level {
    fn index_mut(&mut self, index: LevelPosition) -> &mut Self::Output {
        &mut self.level[index.column() as usize][index.row() as usize]
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Default)]
pub enum Tile {
    #[default]
    Empty,
    Wall,
    Box,
    Destination,
    Player,
    BoxOnDestination,
}

#[derive(Copy, Clone, Eq, PartialEq)]
pub enum TileDisplay {
    Empty,
    Dither,
    Full,
}

impl Tile {
    pub fn byte_of_pair(top: Self, bottom: Self, flash: bool) -> u8 {
        let [top, bottom] = [top, bottom].map(|tile| tile.display_kind(flash));
        (top as u8 * 3 + bottom as u8)
            .checked_sub(1)
            .unwrap_or(b' ')
    }

    pub fn display_kind(self, flash: bool) -> TileDisplay {
        match self {
            Tile::Empty => TileDisplay::Empty,
            Tile::Box => TileDisplay::Dither,
            Tile::Wall | Tile::Player | Tile::BoxOnDestination => TileDisplay::Full,
            Tile::Destination => {
                if flash {
                    TileDisplay::Full
                } else {
                    TileDisplay::Dither
                }
            }
        }
    }
}

type LevelPosition = GenericPosition<2>;

pub fn decode_level(index: u8) -> (LevelPosition, [[Tile; 4]; 16], u8) {
    let &(start, ref level) = &LEVELS[index as usize];

    let mut tiles = [[Tile::Empty; 4]; 16];

    let mut num_boxes = 0;

    for (row_index, &(mut row)) in level.iter().enumerate() {
        for column_index in (0..16).rev() {
            let tile_bits = row as u8 & 3;
            row >>= 2;

            // SAFETY: tile_bits contains only variations of the two least significant bits, which
            // are all valid Tile discriminants
            let tile = unsafe { mem::transmute::<u8, Tile>(tile_bits) };

            tiles[column_index][row_index] = tile;

            if tile == Tile::Box {
                num_boxes += 1;
            }
        }
    }

    tiles[start.column() as usize][start.row() as usize] = Tile::Player;

    (start, tiles, num_boxes)
}

pub const fn parse_levels<const N: usize>(file: &[u8]) -> [(LevelPosition, [u32; 4]); N] {
    let mut level = [(LevelPosition::new(0, 0), [0u32; 4]); N];
    let mut i = 0;

    let mut character = 0;

    while i < file.len() {
        // Skip label
        loop {
            let byte = file[i];
            if byte == b'\n' {
                if file[i - 1] == b'\r' {
                    assert!(file[i - 2] == b':', "Line must end with a `:`");
                } else {
                    assert!(file[i - 1] == b':', "Line must end with a `:`");
                }

                i += 1;
                break;
            }
            i += 1;
        }

        let mut start_position = None;

        // Read character
        let mut line_number = 0;
        while line_number < 4 {
            assert!(file[i] == b'|', "Line must start with a `|`");
            i += 1;

            // Read line
            let mut line = 0u32;
            let mut length = 0u32;

            loop {
                let byte = file[i];

                match byte {
                    b' ' => {
                        line <<= 2;
                    }
                    b'#' => {
                        line <<= 2;
                        line |= 1;
                    }
                    b'x' => {
                        line <<= 2;
                        line |= 2;
                    }
                    b'z' => {
                        line <<= 2;
                        line |= 3;
                    }
                    b'p' => {
                        assert!(
                            start_position.is_none(),
                            "Player position can only be defined once per level",
                        );
                        start_position = Some(LevelPosition::new(length as u8, line_number as u8));
                        line <<= 2;
                    }
                    b'|' => break,
                    b'\n' => panic!("Line must end with a `|`"),
                    _ => panic!("Line must contain only ` `, `#`, `x`, or `z` between the `|`s"),
                }

                i += 1;
                length += 1;
            }
            assert!(
                length == 16,
                "Line must contain 16 characters between the `|`s"
            );

            assert!(file[i] == b'|', "Line must end with a `|`");
            i += 1;
            if i < file.len() && file[i] == b'\r' {
                i += 1;
            }

            assert!(
                i >= file.len() || file[i] == b'\n',
                "Line must end with a `|`"
            );
            i += 1;

            level[character].1[line_number] = line;

            line_number += 1;
        }

        level[character].0 = start_position.expect("Player start position must be defined");

        character += 1;
    }

    level
}
