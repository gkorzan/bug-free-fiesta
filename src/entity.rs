use rand::Rng;
use tcod::{
    colors::{self, Color},
    Console,
};

use crate::{
    room::Room,
    tile::{Map, MAP_HEIGHT, MAP_WIDTH},
};

const MAX_ROOM_MONSTERS: i32 = 3;
pub const PLAYER: usize = 0;

#[derive(Debug)]
pub struct Entity {
    x: i32,
    y: i32,
    char: char,
    color: Color,
}

impl Entity {
    pub fn new(x: i32, y: i32, char: char, color: Color) -> Self {
        Entity { x, y, char, color }
    }

    pub fn move_by(&mut self, dx: i32, dy: i32, map: &Map) {
        if self.x + dx >= MAP_WIDTH
            || (self.y + dy) >= MAP_HEIGHT
            || self.x + dx < 0
            || self.y + dy < 0
        {
            return;
        }

        if map[(self.x + dx) as usize][(self.y + dy) as usize].get_is_passable() {
            self.x += dx;
            self.y += dy;
        }
    }

    pub fn draw(&self, con: &mut dyn Console) {
        con.set_default_foreground(self.color);
        con.put_char(self.x, self.y, self.char, tcod::BackgroundFlag::None)
    }

    pub fn get_coordinates(&self) -> (i32, i32) {
        (self.x, self.y)
    }
    pub fn set_position(&mut self, x: i32, y: i32) {
        self.x = x;
        self.y = y;
    }

    pub fn populate_room(room: &mut Room, entity: &mut Vec<Entity>) {
        let num_monsters = rand::thread_rng().gen_range(0..=MAX_ROOM_MONSTERS);
        let (x1, x2, y1, y2) = room.get_room_coordinates();
        for _ in 0..num_monsters {
            let x = rand::thread_rng().gen_range(x1 + 1..x2);
            let y = rand::thread_rng().gen_range(y1 + 1..y2);

            let do_generate_ork = rand::random::<f32>() < 0.8;

            let monster = if do_generate_ork {
                Entity::new(x, y, 'o', colors::DESATURATED_GREEN) // generate ORK
            } else {
                Entity::new(x, y, 'T', colors::DARKER_GREEN) // gen TROLL
            };

            entity.push(monster);
        }
    }
}
