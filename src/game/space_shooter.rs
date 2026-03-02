use core::ops::{Index, IndexMut};

use arduino_hal::prelude::_unwrap_infallible_UnwrapInfallible;
use ufmt::uwrite;

use crate::{
    game::{position::Position, GameMode},
    lcd::characters,
    rng, utils, LCD,
};

pub struct SpaceShooter {
    pub ship_position: Position,
    pub ship_health: u8,
    pub ship_health_flash_time: u8,

    pub shoot_row: Option<u8>,
    pub shoot_released: u8,
    pub shoot_cooldown: i8,
    pub triple_shot_cooldown: u8,

    pub spawn_cooldown: u8,
    pub update_cooldown: u8,

    pub objects: [[Option<Object>; 16]; 2],

    pub exit_countdown: u8,

    pub score: u32,
}

impl Default for SpaceShooter {
    fn default() -> Self {
        Self {
            ship_position: Position::new(1, 0),
            ship_health: Self::MAX_HEALTH,
            ship_health_flash_time: 0,

            shoot_row: None,
            shoot_released: u8::MAX,
            shoot_cooldown: 0,
            triple_shot_cooldown: 0,

            spawn_cooldown: 0,
            update_cooldown: 0,

            objects: [[None; 16]; 2],

            exit_countdown: Self::EXIT_COUNTDOWN,

            score: 0,
        }
    }
}

impl SpaceShooter {
    pub const PLAYER_CHARACTER: u8 = 0;

    pub const PLAYER_SHOOT_COOLDOWN: u8 = 3;
    pub const MAX_HEALTH: u8 = 3;

    pub const UPDATE_COOLDOWN: u8 = 20;
    pub const SHIP_HEALTH_FLASH_TIME: u8 = 60;

    pub const MIN_TIME: u8 = 2;
    pub const MAX_TIME: u8 = 5;

    pub const DOUBLE_SPAWN_CHANCE: u8 = (80u16 * 255 / 100) as u8;

    pub const EXIT_COUNTDOWN: u8 = 60 * 4;

    pub const TRIPLE_SHOT_COOLDOWN: u8 = 24;

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

        self.draw_player(lcd);
    }

    pub fn draw_player(&self, lcd: &mut LCD) {
        lcd.set_cursor(self.ship_position);
        lcd.write(if self.ship_health_flash_time == 0 {
            Self::PLAYER_CHARACTER
        } else {
            b'0' + self.ship_health.min(9)
        });
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
            Some(
                Object::Projectile | Object::TripleProjectile | Object::Beam | Object::BeamDecay,
            ) => match object {
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
        if self.ship_health == 0 {
            if self.exit_countdown == Self::EXIT_COUNTDOWN {
                lcd.set_cursor(Position::new(2, 0));
                lcd.print_bytes(b"Score: \x04");
                uwrite!(lcd.fmt(), "{}", self.score).unwrap_infallible();
                for _ in 8 + utils::num_digits(self.score)..16 {
                    lcd.write(b' ');
                }
            }

            self.exit_countdown -= 1;

            if self.exit_countdown == 0 {
                characters::load_character_set(lcd, 0);
                return Some(GameMode::Overworld);
            } else {
                return None;
            }
        }

        if raw_input[0] == -1 {
            self.use_power_up(lcd);
        }

        self.update_ship_position(lcd, raw_input);
        if raw_input[0] == 1 {
            if self.shoot_released > 2 && self.shoot_cooldown >= 0 {
                let row = self.ship_position.row();
                self.shoot_row = Some(row);

                let position = Position::new(2, row);

                if self.shoot_cooldown == 0 && self[position].is_none() {
                    lcd.set_cursor(position);
                    lcd.write(if self.triple_shot_cooldown == 0 {
                        Object::Projectile
                    } else {
                        Object::TripleProjectile
                    } as u8);
                }
            }

            self.shoot_released = 0;
        } else {
            self.shoot_released += 1;
        }

        if self.update_cooldown > 0 {
            self.update_cooldown -= 1;
        } else {
            self.update_ship_shooting(lcd, raw_input);

            for i in 0..self.objects.len() {
                self.update_row(lcd, i);
            }

            self.spawn_objects(lcd);

            self.update_ship_collision(lcd);

            if self.triple_shot_cooldown > 0 {
                self.triple_shot_cooldown -= 1;
            }

            self.update_cooldown = Self::UPDATE_COOLDOWN.saturating_sub(1);
        }

        if self.ship_health_flash_time > 0 {
            self.ship_health_flash_time -= 1;

            if self.ship_health_flash_time == 0 {
                self.draw_player(lcd);
            }
        }

        None
    }

    pub fn use_power_up(&mut self, lcd: &mut LCD) {
        let power_up_position = self.ship_position.with_column(0);
        let Some(power_up) = self[power_up_position] else {
            return;
        };
        match power_up {
            Object::BeamPowerUpStored => {
                lcd.set_cursor(self.ship_position.with_column(2));
                let row = self.ship_position.row();
                for i in 2..self.objects[0].len() as u8 {
                    lcd.write(b'-');
                    self.objects[row as usize][i as usize] = Some(Object::Beam);
                }
            }
            Object::TripleShotPowerUpStored => {
                self.triple_shot_cooldown = Self::TRIPLE_SHOT_COOLDOWN;
            }
            _ => return,
        }
        self.set_object(lcd, power_up_position, None);
    }

    pub fn update_ship_position(&mut self, lcd: &mut LCD, raw_input: [i8; 2]) {
        let new_position = match raw_input[1] {
            1 => self.ship_position.with_row(1),
            -1 => self.ship_position.with_row(0),
            _ => return,
        };

        lcd.set_cursor(self.ship_position);
        lcd.write(b' ');

        self.ship_position = new_position;

        self.draw_player(lcd);

        self.update_ship_collision(lcd);
    }

    pub fn update_ship_shooting(&mut self, lcd: &mut LCD, raw_input: [i8; 2]) {
        if self.shoot_cooldown > 0 {
            self.shoot_cooldown -= 1;
            return;
        }

        let shoot_control = raw_input[0] == 1 || self.shoot_row.is_some();

        if !(shoot_control || self.shoot_cooldown < 0) {
            return;
        }

        if shoot_control && self.triple_shot_cooldown > 0 && self.shoot_cooldown == 0 {
            self.shoot_cooldown = -3;
        }

        let row = self.shoot_row.unwrap_or(self.ship_position.row());
        self.shoot_row = None;

        if self.shoot_cooldown < 0 {
            self.shoot_cooldown += 1;
        }

        if self.shoot_cooldown == 0 {
            self.shoot_cooldown = Self::PLAYER_SHOOT_COOLDOWN.saturating_sub(1) as i8;
        }

        let projectile = if self.triple_shot_cooldown > 0 || self.shoot_cooldown < 0 {
            Object::TripleProjectile
        } else {
            Object::Projectile
        };

        self.set_object(lcd, Position::new(2, row), Some(projectile));
    }

    pub fn update_row(&mut self, lcd: &mut LCD, row: usize) {
        let mut i = 0;
        while i < self.objects[row].len() as u8 {
            let Some(mut object) = self.objects[row][i as usize].take() else {
                i += 1;
                continue;
            };
            lcd.set_cursor(Position::new(i, row as u8));
            lcd.write(b' ');

            let next_position = match object {
                Object::BeamPowerUpStored | Object::TripleShotPowerUpStored => Some(i),
                Object::Beam => {
                    object = Object::BeamDecay;
                    Some(i)
                }
                Object::BeamDecay => None,
                Object::Projectile | Object::TripleProjectile => {
                    i += 1;
                    while self.objects[row]
                        .get(i as usize)
                        .copied()
                        .flatten()
                        .is_some_and(Object::is_projectile)
                    {
                        i += 1;
                    }
                    let next_position = i;
                    let max_position = self.objects[row].len() as u8;
                    i -= 1;
                    (next_position < max_position).then_some(next_position)
                }
                _ => i.checked_sub(1),
            };

            if let Some(next_position) = next_position {
                let success =
                    self.set_object(lcd, Position::new(next_position, row as u8), Some(object));

                if success && object.is_projectile() {
                    i += 1;
                }
            }
            i += 1;
        }
    }

    pub fn spawn_objects(&mut self, lcd: &mut LCD) {
        if self.spawn_cooldown > 0 {
            self.spawn_cooldown -= 1;
        } else {
            self.spawn_cooldown = Self::MIN_TIME.saturating_sub(1)
                + (rng::rng() % (1 + Self::MAX_TIME - Self::MIN_TIME) as u32) as u8;

            let row = rng::rng() as u8 & 1;
            let object = Object::random();
            self.set_object(
                lcd,
                Position::new(self.objects[0].len() as u8 - 1, row),
                Some(object),
            );

            if (rng::rng() as u8) < Self::DOUBLE_SPAWN_CHANCE {
                let mut object_2 = Object::random();
                if object == object_2 {
                    object_2 = Object::random();
                }
                self.set_object(
                    lcd,
                    Position::new(self.objects[0].len() as u8 - 1, 1 - row),
                    Some(object_2),
                );
            }
        }
    }

    pub fn update_ship_collision(&mut self, lcd: &mut LCD) {
        let position = self.ship_position;
        let Some(object) = self[position].take() else {
            return;
        };

        match object {
            Object::Asteroid2X | Object::Asteroid => {
                if self.ship_health_flash_time == 0 {
                    self.ship_health = self.ship_health.saturating_sub(1);
                    self.ship_health_flash_time = Self::SHIP_HEALTH_FLASH_TIME;
                }
            }
            Object::Health => {
                if self.ship_health < Self::MAX_HEALTH {
                    self.ship_health += 1;
                    self.ship_health_flash_time = Self::SHIP_HEALTH_FLASH_TIME;
                } else {
                    self.score += 1;
                }
            }
            Object::BeamPowerUpCollectible | Object::TripleShotPowerUpCollectible => {
                let position = self.ship_position.with_column(0);
                self[position] = None;
                self.set_object(lcd, position, object.into_stored_power_up());
            }
            Object::Point => {
                self.score += 1;
            }
            _ => (),
        }

        self.draw_player(lcd);
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
    TripleProjectile = b'>',
    Beam = 0xb0, // Looks the same as b'-'
    BeamDecay = b'=',
    Health = b'+',
    Point = 0x04,
    BeamPowerUpCollectible = 0x02,
    BeamPowerUpStored = 0x0a, // Looks the same as 0x03
    TripleShotPowerUpCollectible = 0x03,
    TripleShotPowerUpStored = 0x0b, // Looks the same as 0x03
}

impl Object {
    pub fn random() -> Self {
        match rng::rng() % 100 {
            100.. => unreachable!(),
            ..25 => Object::Asteroid,
            ..75 => Object::Asteroid2X,
            ..93 => Object::Point,
            ..95 => Object::Health,
            ..98 => Object::BeamPowerUpCollectible,
            ..100 => Object::TripleShotPowerUpCollectible,
        }
    }

    pub fn is_projectile(self) -> bool {
        matches!(self, Object::Projectile | Object::TripleProjectile)
    }

    pub fn into_stored_power_up(self) -> Option<Object> {
        Some(match self {
            Object::BeamPowerUpCollectible | Object::BeamPowerUpStored => Object::BeamPowerUpStored,
            Object::TripleShotPowerUpCollectible | Object::TripleShotPowerUpStored => {
                Object::TripleShotPowerUpStored
            }
            _ => return None,
        })
    }

    pub fn is_stored_power_up(self) -> bool {
        matches!(
            self,
            Object::BeamPowerUpStored | Object::TripleShotPowerUpStored
        )
    }
}
