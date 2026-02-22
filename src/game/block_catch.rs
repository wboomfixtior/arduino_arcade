use core::ops::{Index, IndexMut, Range};

use crate::{
    game::{position::Position, GameMode},
    rng, LCD,
};

pub struct BlockCatch {
    pub player_position: Position,
    pub blocks: [Option<Block>; 8],
}

impl Default for BlockCatch {
    fn default() -> Self {
        Self {
            player_position: Position::new(0, 0),
            blocks: [None; 8],
        }
    }
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

        lcd.set_cursor(self.player_position);
        lcd.write(Self::PLAYER_CHARACTER);

        let mut first = true;

        for (i, block) in self.blocks.iter().enumerate() {
            let Some(block) = block else { continue };

            block.draw_at(lcd, i as u8);

            if first && i != 0 {
                block.draw_guide_at(lcd, 0, Some(self.player_position));
            }

            first = false;
        }
    }

    pub fn update(&mut self, lcd: &mut LCD, raw_input: [i8; 2]) -> Option<GameMode> {
        self.move_player_by(lcd, raw_input);

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
        let mut new_position = self.player_position;

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

        if new_position == self.player_position {
            return false;
        }

        lcd.set_cursor(self.player_position);
        let marker = if let Some(marker_block) = self.first_block_index() {
            self.blocks[marker_block as usize].unwrap()[self.player_position].marker_tile()
        } else {
            b' '
        };
        lcd.write(marker);

        lcd.set_cursor(new_position);
        lcd.write(Self::PLAYER_CHARACTER);

        self.player_position = new_position;

        true
    }
}

impl Block {
    pub fn draw_at(&self, lcd: &mut LCD, index: u8) {
        self.draw_with_map_at(lcd, |tile, _| Some(tile as u8), index)
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
