use rand::Rng;
use tcod::{
    colors::{self, Color},
    Console,
};

use crate::{
    room::Room,
    tile::{Map, Tile, MAP_HEIGHT, MAP_WIDTH},
};

const MAX_ROOM_MONSTERS: i32 = 3;
pub const PLAYER: usize = 0;

// TODO : refactor player movement code types

// struct Coordinates {
//     pub x: i32,
//     pub y: i32,
// }
// impl Coordinates {
//     pub fn new(x: i32, y: i32) -> Coordinates {
//         Coordinates { x, y }
//     }
// }
// enum Direction {
//     Up,
//     Down,
//     Right,
//     Left,
// }
// impl Direction {
//     pub fn get_coords(dir: Direction) -> Coordinates {
//         match dir {
//             Direction::Up => Coordinates { x: 0, y: -1 },
//             Direction::Down => Coordinates { x: 0, y: 1 },
//             Direction::Left => Coordinates { x: -1, y: 0 },
//             Direction::Right => Coordinates { x: 1, y: 0 },
//         }
//     }
// }

#[derive(Debug)]
pub struct Entity {
    x: i32,
    y: i32,
    char: char,
    color: Color,
    name: String,
    blocks: bool,
    alive: bool,
}

impl Entity {
    pub fn new(x: i32, y: i32, char: char, color: Color, name: &str, blocks: bool) -> Self {
        Entity {
            x,
            y,
            char,
            color,
            name: name.to_string(),
            blocks,
            alive: false,
        }
    }

    pub fn move_by(id: usize, dx: i32, dy: i32, map: &Map, entities: &mut [Entity]) {
        let (x, y) = entities[id].get_coordinates();
        if x + dx >= MAP_WIDTH || (y + dy) >= MAP_HEIGHT || x + dx < 0 || y + dy < 0 {
            return;
        }
        // println!("{:?}", (Tile::is_blocked(x + dx, y + dy, map, entities)));

        if !Tile::is_blocked(x + dx, y + dy, map, entities) {
            entities[id].set_position(x + dx, y + dy);
        }
    }

    pub fn player_move_or_attack(_id: usize, dx: i32, dy: i32, map: &Map, entities: &mut [Entity]) {
        let (mut x, mut y) = entities[PLAYER].get_coordinates();
        x = x + dx;
        y = y + dy;

        let target_id = entities
            .iter()
            .position(|entity| entity.get_coordinates() == (x, y));

        match target_id {
            Some(target_id) => {
                println!(
                    "The {0} laughs at your puny efforts to attacl him!",
                    entities[target_id].get_name()
                )
            }
            None => Entity::move_by(PLAYER, dx, dy, &map, entities),
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
    pub fn get_is_blocks(&self) -> bool {
        self.blocks
    }
    pub fn make_alive(&mut self) {
        self.alive = true;
    }
    pub fn kill(&mut self) {
        self.alive = false;
    }
    pub fn is_alive(&self) -> bool {
        self.alive
    }
    pub fn get_name(&self) -> &String {
        &self.name
    }

    pub fn populate_room(room: &mut Room, map: &Map, entities: &mut Vec<Entity>) {
        let num_monsters = rand::thread_rng().gen_range(0..=MAX_ROOM_MONSTERS);
        let (x1, x2, y1, y2) = room.get_room_coordinates();
        for _ in 0..num_monsters {
            let x = rand::thread_rng().gen_range(x1 + 1..x2);
            let y = rand::thread_rng().gen_range(y1 + 1..y2);

            let do_generate_ork = rand::random::<f32>() < 0.8;
            if !Tile::is_blocked(x, y, map, entities) {
                let mut monster = if do_generate_ork {
                    Entity::new(x, y, 'o', colors::DESATURATED_GREEN, "Ork", true)
                // generate ORK
                } else {
                    Entity::new(x, y, 'T', colors::DARKER_GREEN, "Troll", true) // gen TROLL
                };
                monster.make_alive();
                entities.push(monster);
            }
        }
    }

    pub fn mobs_turn(entities: &[Entity], player_took_turn: bool) {
        if entities[PLAYER].is_alive() && player_took_turn {
            for entity in entities {
                if (entity as *const _) != (&entities[PLAYER] as *const _) {
                    // println!("The {} growls!", entity.get_name());
                }
            }
        }
    }
}
