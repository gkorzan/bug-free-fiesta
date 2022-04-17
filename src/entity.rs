use tcod::{colors::Color, Console};

use crate::tile::{Map, MAP_HEIGHT, MAP_WIDTH};

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
        println!("{:?}", self.x + dx);

        if map[(self.x + dx) as usize][(self.y + dy) as usize].get_is_passable() {
            self.x += dx;
            self.y += dy;
        }
    }

    pub fn draw(&self, con: &mut dyn Console) {
        con.set_default_foreground(self.color);
        con.put_char(self.x, self.y, self.char, tcod::BackgroundFlag::None)
    }
}
