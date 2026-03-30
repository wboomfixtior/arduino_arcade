//just the beats game?
//something randomly adding to a 8 size arrray 
//that is then split to either left or right bolth size 8
//every time increment moves notes up
//hit the right note and an increment hapens reseting time 
//dificulty is number in the corner slowly going down, the time left to make an action 
use core::ops::{Index, IndexMut, Range};
use arduino_hal::prelude::_unwrap_infallible_UnwrapInfallible;
use ufmt::uwrite;

use crate::{
    game::{position::Position, GameMode},
    rng, utils, LCD,
};

pub struct note_beat{
    // pub spawn_timer: u8,
    // pub move_timer: u8,
    // pub score: u8,
    // generated_array:[directions; 8], 
//array of positons or somthing for 
}

impl default for note_beat{
    fn default() -> Self{
        Self{
            // spawn_timer: 0,
            // move_timer: 0,
            // score: 0,
            

        }

    }

}
pub enum directions{
    left, right, none

    
}