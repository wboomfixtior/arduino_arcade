use core::ops::{Index, IndexMut};

use crate::{
    game::{position::Position, GameMode},
    lcd::characters,
    LCD,
};

pub struct SpaceShooter {
    pub ship_position: Position,
    pub ship_health: u8,
    pub ship_health_flash_time: u8,

    pub shoot_row: Option<u8>,
    pub shoot_released: u8,
    pub shoot_cooldown: i8,
    pub update_cooldown: u8,

    pub objects: [[Option<Object>; 16]; 2],

    pub score: u32,
}

impl Default for SpaceShooter {
    fn default() -> Self {
        Self {
            ship_position: Position::new(1, 0),
            ship_health: 1,
            ship_health_flash_time: 0,

            shoot_row: None,
            shoot_released: u8::MAX,
            shoot_cooldown: 0,
            update_cooldown: 0,

            objects: [[None; 16]; 2],

            score: 0,
        }
    }
}

impl SpaceShooter {
    pub const PLAYER_CHARACTER: u8 = 0;

    pub const PLAYER_SHOOT_COOLDOWN: u8 = 3;

    pub const UPDATE_COOLDOWN: u8 = 20;

    pub fn draw_full_screen(&self, lcd: &mut LCD) {
        characters::load_character_set(lcd, 2);

        lcd.clear();

        for (row_index, row) in self.objects.iter().enumerate() {
            let mut cursor_valid = false;

            for (column_index, &object) in row.iter().enumerate() {
                let Some(object) = object else {
                    cursor_valid = false;
                    continue;
                };

                if !cursor_valid {
                    lcd.set_cursor(Position::new(column_index as u8, row_index as u8));
                    cursor_valid = true;
                }
                lcd.write(object as u8);
            }
        }

        lcd.set_cursor(self.ship_position);
        lcd.write(Self::PLAYER_CHARACTER)
    }

    pub fn set_object(
        &mut self,
        lcd: &mut LCD,
        position: Position,
        object: Option<Object>,
    ) -> bool {
        let target = &mut self[position];

        let mut success = false;
        match *target {
            Some(Object::Asteroid2X) => *target = Some(Object::Asteroid),
            Some(Object::Asteroid) => *target = None,
            Some(Object::Projectile) => match object {
                Some(Object::Asteroid2X) => *target = Some(Object::Asteroid),
                Some(Object::Asteroid) => *target = None,
                _ => (),
            },
            _ => {
                *target = object;
                success = true;
            }
        };

        lcd.set_cursor(position);
        lcd.write(target.map(|x| x as u8).unwrap_or(b' '));

        success
    }

    pub fn update(&mut self, lcd: &mut LCD, raw_input: [i8; 2]) -> Option<GameMode> {
        self.update_ship_position(lcd, raw_input);
        if raw_input[0] == 1 {
            if self.shoot_released > 2 {
                let row = self.ship_position.row();
                self.shoot_row = Some(row);

                if self.shoot_cooldown == 0 {
                    lcd.set_cursor(Position::new(2, row));
                    lcd.write(b'-');
                }
            }

            self.shoot_released = 0;
        } else {
            self.shoot_released += 1;
        }

        if self.update_cooldown > 0 {
            self.update_cooldown -= 1;
        } else {
            for i in 0..self.objects.len() {
                self.update_row(lcd, i);
            }

            self.update_ship_shooting(lcd, raw_input);
            self.shoot_row = None;

            self.update_cooldown = Self::UPDATE_COOLDOWN.saturating_sub(1);
        }

        None
    }

    pub fn update_ship_position(&mut self, lcd: &mut LCD, raw_input: [i8; 2]) {
        let new_position = match raw_input[1] {
            1 => self.ship_position.with_row(1),
            -1 => self.ship_position.with_row(0),
            _ => return,
        };

        lcd.set_cursor(self.ship_position);
        lcd.write(b' ');

        lcd.set_cursor(new_position);
        lcd.write(Self::PLAYER_CHARACTER);

        self.ship_position = new_position;
    }

    pub fn update_ship_shooting(&mut self, lcd: &mut LCD, raw_input: [i8; 2]) {
        if self.shoot_cooldown > 0 {
            self.shoot_cooldown -= 1;
            return;
        }

        if !(raw_input[0] == 1 || self.shoot_row.is_some() || self.shoot_cooldown < 0) {
            return;
        }

        let row = self.shoot_row.unwrap_or(self.ship_position.row());

        if self.shoot_cooldown < 0 {
            self.shoot_cooldown += 1;
        }

        if self.shoot_cooldown == 0 {
            self.shoot_cooldown = Self::PLAYER_SHOOT_COOLDOWN.saturating_sub(1) as i8;
        }

        self.set_object(lcd, Position::new(2, row), Some(Object::Projectile));
    }

    pub fn update_row(&mut self, lcd: &mut LCD, row: usize) {
        let mut i = 0;
        while i < self.objects[row].len() as u8 {
            let Some(object) = self.objects[row][i as usize].take() else {
                i += 1;
                continue;
            };
            lcd.set_cursor(Position::new(i, row as u8));
            lcd.write(b' ');

            let next_position = match object {
                Object::Projectile => {
                    while self.objects[row].get(i as usize).copied().flatten()
                        == Some(Object::Projectile)
                    {
                        i += 1;
                    }
                    let next_position = i + 1;
                    (next_position < self.objects[row].len() as u8).then_some(next_position)
                }
                _ => i.checked_sub(1),
            };

            if let Some(next_position) = next_position {
                let success =
                    self.set_object(lcd, Position::new(next_position, row as u8), Some(object));

                if success && object == Object::Projectile {
                    i += 1;
                }
            }
            i += 1;
        }
    }
}

impl Index<Position> for SpaceShooter {
    type Output = Option<Object>;

    fn index(&self, index: Position) -> &Self::Output {
        &self.objects[index.row() as usize][index.column() as usize]
    }
}

impl IndexMut<Position> for SpaceShooter {
    fn index_mut(&mut self, index: Position) -> &mut Self::Output {
        &mut self.objects[index.row() as usize][index.column() as usize]
    }
}

#[derive(Copy, Clone, Eq, PartialEq)]
#[repr(u8)]
pub enum Object {
    Asteroid2X = 0xff,
    Asteroid = 0x01,
    Projectile = b'-',
    Health = b'+',
    BeamPowerUpCollectible = 0x03,
    BeamPowerUpStored = 0x0b,
    TripleShotPowerUpCollectible = 0x04,
    TripleShotPowerUpStored = 0x0c,
}
