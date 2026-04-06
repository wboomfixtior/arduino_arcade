// just the beats game?
// something randomly adding to a 8 size arrray
// that is then split to either left or right bolth size 8
// every time increment moves notes up
// hit the right note and an increment hapens reseting time
// dificulty is number in the corner slowly going down, the time left to make an action

use crate::{
    game::{position::Position, GameMode},
    rng::rng,
    LCD,
};

pub struct NoteBeat {
    pub objects: [Object; 16],
    pub difficulty: u8,
    pub time: u8,
    pub time_gap: u8,

    pub player_position: Position,

    pub score: u32,
}

impl Default for NoteBeat {
    fn default() -> Self {
        Self {
            objects: [Object::None; 16],
            difficulty: 8,
            time: 0,
            time_gap: 60,

            player_position: Self::START_POSITION,

            score: 0,
        }
    }
}
#[derive(Copy, Clone)]
#[repr(u8)]
pub enum Object {
    None = b' ',
    Default = b'*',
}

impl NoteBeat {
    pub const LEFT_POSITION: Position = Position::new(0, 0);
    pub const START_POSITION: Position = Position::new(7, 1);

    pub const PLAYER_CHARACTER: u8 = 0;

    pub fn draw_full_screen(&mut self, lcd: &mut LCD) {
        lcd.clear();

        lcd.set_cursor(Position::new(0, 0));
        for enemy in self.objects {
            lcd.write(enemy as u8);
        }

        lcd.set_cursor(self.player_position);
        lcd.write(Self::PLAYER_CHARACTER);
    }

    pub fn add_to_queue(&mut self) {
        let random = rng() % self.difficulty as u32;

        // TODO: if the player hits an enemy delete it and reduce time until the next enemy

        for i in 0..7 {
            self.objects[i] = self.objects[i + 1];
        }
        for i in 9..16 {
            self.objects[i] = self.objects[i - 1];
        }

        if random < 4 {
            let index = if random % 2 == 0 { 0 } else { 15 };
            self.objects[index] = Object::Default;
        }
    }

    pub fn update(
        &mut self,
        lcd: &mut LCD,
        raw_input: [i8; 2],
        soft_input: [i8; 2],
    ) -> Option<GameMode> {
        if raw_input[1] < 0 {
            return Some(GameMode::Overworld);
        }

        lcd.set_cursor(Self::LEFT_POSITION);

        if self.time % self.time_gap == 0 {
            self.add_to_queue();
            self.time = 0;
            lcd.clear();
        } else {
            self.time = self.time + 1;
        }

        if raw_input[0] != 0 {
            lcd.set_cursor(self.player_position);
            lcd.write(b' ');

            self.player_position = if raw_input[0] > 0 {
                // Right
                Self::START_POSITION.nudge_column_overflowing(1).0
            } else {
                // Left
                Self::START_POSITION
            };

            lcd.set_cursor(self.player_position);
            lcd.write(Self::PLAYER_CHARACTER);
        }

        None
    }
}
