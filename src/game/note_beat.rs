use ufmt::uwrite;

use crate::{
    game::{position::Position, GameMode},
    rng::rng,
    LCD,
};

// what I need to add hit detdection 
// for self and objects
// score 

pub struct NoteBeat {
    pub objects: [Object; 16],
    pub difficulty: u8,
    pub time: u8,
    pub time_gap: u8,
    
    pub player_position: Position,
    pub player_hit: bool,
    pub lives: u8,
    pub combo: u16,
    pub score: u32,
}

impl Default for NoteBeat {
    fn default() -> Self {
        Self {
            objects: [Object::None; 16],
            difficulty: 10,
            time: 0,
            time_gap: 60,

            player_position: Self::START_POSITION,

            player_hit: false,
            // normaly there would be three beond testing
            lives: 30,
            combo: 1,
            score: 0,
        }
    }
}
#[derive(Copy, Clone, Eq, PartialEq)]
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
        
        self.move_objects();
        
        if random < 4 {
             if random % 2 == 0 { 
                self.objects[0]= Object::Default; 
                self.objects[15]=Object::None;
            } else { 
                self.objects[0]= Object::None; 
                self.objects[15]=Object::Default; 
            }
        }
        
    }

    fn move_objects(&mut self){

        for i in 0..7 {
            self.objects[7-i] = self.objects[6-i];
        }
        for i in 8..15 {
           self.objects[i] = self.objects[i+1];
       }

        if (self.objects[7]==Object::Default)||(self.objects[8]==Object::Default){
            self.player_hit=true;
        }
    }

    fn hit_object(&mut self, raw_input: [i8; 2]){

            if (raw_input[0]>0)&&(self.objects[6]==Object::Default){
                self.objects[6]=Object::None;
                self.score=self.score+self.combo as u32;
                self.combo=self.combo+1;

            }else if(raw_input[0]<0)&&(self.objects[9]==Object::Default){
                self.objects[9]=Object::None;
                self.score=self.score+self.combo as u32;
                self.combo=self.combo+1;

            }else {
                self.combo=1;
            }
    }


    pub fn update(
        &mut self,
        lcd: &mut LCD,
        raw_input: [i8; 2],
        _soft_input: [i8; 2],
    ) -> Option<GameMode> {
        

        lcd.set_cursor(Self::LEFT_POSITION);

        if self.time % self.time_gap == 0 {

            self.add_to_queue();
            self.move_objects();
            lcd.clear();
            for i in self.objects{
                match i {
                    Object::None=>  lcd.write(b'_'),
                    Object::Default=>  lcd.write(b'*'),
               }
            }
            self.time = 1;
            
        } else {
            self.time = self.time + 1;
        }

        if self.player_hit {
            if self.lives>0{
                self.lives=self.lives-1;
                self.objects=[Object::None; 16];
                lcd.set_cursor(self.player_position);
                self.player_hit=false;
                for _ in 0..(self.lives/5){
                    lcd.write(b'x');
                }
                 
            }else{
                return Some(GameMode::Overworld);
            }
            
        }

        if raw_input[0] != 0 {
            lcd.set_cursor(self.player_position);
            lcd.write(b' ');

            self.hit_object( raw_input);
            self.player_position = if raw_input[0] > 0 {
                // Right
                Self::START_POSITION.nudge_column_overflowing(1).0
            } else {
                // Left
                Self::START_POSITION
            };

            lcd.set_cursor(self.player_position);
            lcd.write(Self::PLAYER_CHARACTER);

            //lcd.set_cursor(Self::LEFT_POSITION.nudge_row_overflowing(1).0);
            
            
        }

        None
    }
}