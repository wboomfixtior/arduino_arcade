use core::ops::{Index, IndexMut, Range};

use arduino_hal::prelude::_unwrap_infallible_UnwrapInfallible;
use ufmt::uwrite;

use crate::{
    game::{position::Position, GameMode},
    rng, utils, LCD,
};

pub struct BlockCatch {
    pub player_position: Option<Position>,
    pub blocks: [Option<Block>; 8],

    pub block_motion_timer: u8,
    pub block_spawn_timer: u8,

    pub max_block_spawn_time: u8,
    pub difficulty_timer: u8,
    pub score: u32,
}

impl Default for BlockCatch {
    fn default() -> Self {
        Self {
            player_position: Some(Position::new(0, 0)),
            blocks: [None; 8],

            block_motion_timer: 0,
            block_spawn_timer: 0,

            max_block_spawn_time: Self::INITIAL_MAX_BLOCK_SPAWN_TIME,
            difficulty_timer: Self::MAX_DIFFICULTY_TIME,
            score: 0,
        }
    }
}

impl BlockCatch {
    pub const MAX_BLOCK_MOTION_TIME: u8 = 30;
    pub const MAX_DIFFICULTY_TIME: u8 = 60;
    pub const INITIAL_MAX_BLOCK_SPAWN_TIME: u8 = 5;

    pub const GAME_OVER_TIME: u8 = 60 * 4;
}

#[derive(Copy, Clone)]
pub struct Block {
    pub tiles: [[Tile; 2]; 2],
}

impl Index<Position> for Block {
    type Output = Tile;

    fn index(&self, index: Position) -> &Self::Output {
        &self.tiles[index.row() as usize][(index.column() & 1) as usize]
    }
}

impl IndexMut<Position> for Block {
    fn index_mut(&mut self, index: Position) -> &mut Self::Output {
        &mut self.tiles[index.row() as usize][(index.column() & 1) as usize]
    }
}

#[derive(Copy, Clone, Eq, PartialEq)]
#[repr(u8)]
pub enum Tile {
    Empty = b'_',
    Collectible = 0x02,
    Wall = 0xff,
}

impl BlockCatch {
    pub const PLAYER_CHARACTER: u8 = 0;

    pub fn draw_full_screen(&mut self, lcd: &mut LCD) {
        lcd.clear();

        if let Some(position) = self.player_position {
            lcd.set_cursor(position);
            lcd.write(Self::PLAYER_CHARACTER);
        }

        let mut first = true;

        for (i, block) in self.blocks.iter().enumerate() {
            let Some(block) = block else { continue };

            if i == 0 {
                block.draw_at(lcd, i as u8, self.player_position);
            } else {
                block.draw_at(lcd, i as u8, None);

                if first && i != 0 {
                    block.draw_guide_at(lcd, 0, self.player_position);
                }
            }

            first = false;
        }

        if self.player_position.is_none() {
            lcd.set_cursor(Position::new(2, 0));
            uwrite!(lcd.fmt(), "Score: {}", self.score).unwrap_infallible();
            for _ in 7 + utils::num_digits(self.score)..16 {
                lcd.write(b' ');
            }
        }
    }

    pub fn update(&mut self, lcd: &mut LCD, raw_input: [i8; 2]) -> Option<GameMode> {
        self.move_player_by(lcd, raw_input);

        self.block_motion_timer = self.block_motion_timer.saturating_sub(1);
        if self.block_motion_timer == 0 {
            let Some(player_position) = self.player_position else {
                return Some(GameMode::Overworld);
            };

            self.block_motion_timer = Self::MAX_BLOCK_MOTION_TIME;

            for i in 0..self.blocks.len() {
                self.blocks[i] = self.blocks.get(i + 1).copied().flatten();
            }

            self.block_spawn_timer = self.block_spawn_timer.saturating_sub(1);
            if self.block_spawn_timer == 0 {
                self.block_spawn_timer = self.max_block_spawn_time;

                self.blocks[self.blocks.len() - 1] = Some(Block::random(1..4, 2));
            }

            self.difficulty_timer = self.difficulty_timer.saturating_sub(1);
            if self.difficulty_timer == 0 && self.block_spawn_timer > 2 {
                self.difficulty_timer = Self::MAX_DIFFICULTY_TIME;
                self.max_block_spawn_time -= 1;
            }

            if let Some(block) = &mut self.blocks[0] {
                match block[player_position] {
                    Tile::Empty => (),
                    Tile::Collectible => {
                        block[player_position] = Tile::Empty;
                        self.score += 1;
                    }
                    Tile::Wall => {
                        self.player_position = None;
                        self.block_motion_timer = Self::GAME_OVER_TIME;
                    }
                }
            }

            self.draw_full_screen(lcd);
        }

        None
    }

    pub fn first_block_index(&self) -> Option<u8> {
        let num_empty = self
            .blocks
            .iter()
            .copied()
            .take_while(Option::is_none)
            .count() as u8;

        (num_empty < self.blocks.len() as u8).then_some(num_empty)
    }

    pub fn move_player_by(&mut self, lcd: &mut LCD, input: [i8; 2]) -> bool {
        let Some(position) = self.player_position else {
            return false;
        };

        let mut new_position = position;

        match input[0] {
            1 => new_position = new_position.with_column(1),
            -1 => new_position = new_position.with_column(0),
            _ => (),
        }

        match input[1] {
            1 => new_position = new_position.with_row(1),
            -1 => new_position = new_position.with_row(0),
            _ => (),
        }

        if new_position == position {
            return false;
        }

        if let Some(block) = &mut self.blocks[0] {
            match &mut block[new_position] {
                Tile::Empty => (),
                tile @ Tile::Collectible => {
                    *tile = Tile::Empty;
                }
                Tile::Wall => return false,
            }
        }

        lcd.set_cursor(position);
        let marker = if let Some(marker_block) = self.first_block_index() {
            self.blocks[marker_block as usize].unwrap()[position].marker_tile()
        } else {
            b' '
        };
        lcd.write(marker);

        lcd.set_cursor(new_position);
        lcd.write(Self::PLAYER_CHARACTER);

        self.player_position = Some(new_position);

        true
    }
}

impl Block {
    pub fn draw_at(&self, lcd: &mut LCD, index: u8, skip_position: Option<Position>) {
        self.draw_with_map_at(
            lcd,
            |tile, position| (Some(position) != skip_position).then_some(tile as u8),
            index,
        )
    }

    pub fn draw_guide_at(&self, lcd: &mut LCD, index: u8, skip_position: Option<Position>) {
        self.draw_with_map_at(
            lcd,
            |tile, position| {
                if Some(position) == skip_position {
                    return None;
                };
                let marker = tile.marker_tile();
                (marker != b' ').then_some(marker)
            },
            index,
        )
    }

    pub fn clear_at(&self, lcd: &mut LCD, index: u8, skip_position: Option<Position>) {
        self.draw_with_map_at(
            lcd,
            |_, position| (Some(position) != skip_position).then_some(b' '),
            index,
        )
    }

    pub fn draw_with_map_at(
        &self,
        lcd: &mut LCD,
        mut filter: impl FnMut(Tile, Position) -> Option<u8>,
        index: u8,
    ) {
        let start_column = index * self.tiles[0].len() as u8;

        for (i, row) in self.tiles.iter().enumerate() {
            let mut previous = false;

            for (j, &tile) in row.iter().enumerate() {
                let position = Position::new(start_column + j as u8, i as u8);
                let tile = filter(tile, position);
                if let Some(tile) = tile {
                    if !previous {
                        lcd.set_cursor(position);
                    }
                    lcd.write(tile);
                }
                previous = tile.is_some();
            }
        }
    }

    pub fn random(max_walls: Range<u8>, reciprocal_collectible_chance: u8) -> Self {
        let mut positions = [
            Position::new(0, 0),
            Position::new(1, 0),
            Position::new(0, 1),
            Position::new(1, 1),
        ];
        let mut length = 4;

        let mut block = Block {
            tiles: [[Tile::Empty; 2]; 2],
        };

        let mut insert = |tile| {
            let i = rng::rng() % length as u32;
            length -= 1;
            block[positions[i as usize]] = tile;
            positions[i as usize] = positions[length];
        };

        let num_walls =
            max_walls.start + (rng::rng() % (max_walls.end - max_walls.start) as u32) as u8;
        let collectible = reciprocal_collectible_chance > 0
            && rng::rng() < u32::MAX / reciprocal_collectible_chance as u32;

        for _ in 0..num_walls {
            insert(Tile::Wall);
        }
        if collectible {
            insert(Tile::Collectible);
        }

        block
    }
}

impl Tile {
    pub fn marker_tile(self) -> u8 {
        match self {
            Tile::Empty => b'_',
            Tile::Collectible => b'.',
            Tile::Wall => b' ',
        }
    }

    pub fn random() -> Self {
        match rng::rng() % 3 {
            0 => Tile::Empty,
            1 => Tile::Collectible,
            2 => Tile::Wall,
            _ => unreachable!(),
        }
    }
}
