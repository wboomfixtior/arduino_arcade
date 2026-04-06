///speed , angle of speed 
/// level of thrust, angle of thrust 
/// amount of fuel f99%->f00% 
/// last corner shows the brawd image of angle and distance to the moon 
use ufmt::uwrite;

use crate::{
    game::{position::Position, GameMode},
    rng, utils, LCD,
};

pub struct MoonLanding{
    pub speedX: u8,
    pub speedY: u8,
   // pub velosity: u8,
    pub speedAngle: u8,
    pub thrustPower: u8,
    pub thrustAngle: u8,
    pub fuel: u8,
    pub gravity: u8,
    pub shipMass: u8,

    pub speedDisplay: Position,
    pub thrustDisplay: Position,
    pub fuelDisplay: Position,
    pub placeHolder: Position,

}


impl Default for MoonLanding{
    fn default() -> Self{

        Self{
            speedX: 300,
            speedY: 0,
          //  velosity: sqrt(speedX*speedX+speedY*speedY),
            speedAngle: speedY/speedX,//add tan^-1 later
            thrustPower: 0,
            thrustAngle: 0,//parilel to surface 
            fuel: 100, //used to power thrust
            gravity: 1,
            shipMass: 400+fuel,

            speedDisplay: Position::new(1, 0),
            thrustDisplay: Position::new(0, 0),
            fuelDisplay: Position::new(0, 8),
            placeHolder: Position::new(1, 8),
            
        }

    }
}

impl MoonLanding {
    pub fn draw_full_screen(&mut self, lcd: &mut LCD) {
        lcd.set_cursor(thrustDisplay);
        lcd.clear();
        uwrite!(lcd.fmt(), "Hello, spaaace").unwrap_infallible();
        ///  lcd.set_cursor(speedDisplay);
        /// uwrite!(lcd.fmt(), "m/s {} angle{}", sqrt(speedX*speedX+speedY*speedY), speedAngle ).unwrap_infallible();
        /// lcd.set_cursor(thrustDisplay);
        /// uwrite!(lcd.fmt(), "thrust{} angle{}", thrustPower, thrustAngle ).unwrap_infallible();
        ///  lcd.set_cursor(fuelDisplay);
        /// uwrite!(lcd.fmt(), "fuel{}", fuel ).unwrap_infallible();


    }

    pub fn update(&mut self, lcd: &mut LCD, raw_input: [i8; 2], soft_input: [i8; 2]) -> Option<GameMode> {
        

        None
    }

    

}
