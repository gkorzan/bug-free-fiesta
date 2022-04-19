use std::cmp;

use rand::Rng;

use crate::{
    entity::{Entity, PLAYER},
    tile::{Map, Tile, MAP_HEIGHT, MAP_WIDTH},
};

const ROOM_MAX_SIZE: i32 = 10;
const ROOM_MIN_SIZE: i32 = 6;
const MAX_ROOMS: i32 = 30; //30

pub struct Room {
    x1: i32,
    y1: i32,
    x2: i32,
    y2: i32,
}

impl Room {
    pub fn new(x: i32, y: i32, w: i32, h: i32) -> Self {
        Room {
            x1: x,
            y1: y,
            x2: x + w,
            y2: y + h,
        }
    }
    pub fn put_on_map(&self, map: &mut Map) -> &Self {
        for x in (self.x1 + 1)..self.x2 {
            for y in (self.y1 + 1)..self.y2 {
                map[x as usize][y as usize] = Tile::empty();
            }
        }
        self
    }
    pub fn get_center(&self) -> (i32, i32) {
        let center_x = (self.x1 + self.x2) / 2;
        let center_y = (self.y1 + self.y2) / 2;
        (center_x, center_y)
    }
    pub fn includes(&self, other: &Room) -> bool {
        (self.x1 <= other.x2)
            && (self.x2 >= other.x1)
            && (self.y1 <= other.y2)
            && (self.y2 >= other.y1)
    }
    pub fn get_room_coordinates(&self) -> (i32, i32, i32, i32) {
        (self.x1, self.x2, self.y1, self.y2)
    }

    pub fn create_h_tunel(c1: i32, c2: i32, l: i32, map: &mut Map) {
        for c in cmp::min(c1, c2)..(cmp::max(c1, c2) + 1) {
            map[c as usize][l as usize] = Tile::empty();
        }
    }
    pub fn create_v_tunel(c1: i32, c2: i32, l: i32, map: &mut Map) {
        for c in cmp::min(c1, c2)..(cmp::max(c1, c2) + 1) {
            map[l as usize][c as usize] = Tile::empty();
        }
    }
    pub fn generate_rooms(map: &mut Map, entities: &mut Vec<Entity>) {
        let mut rooms = Vec::<Room>::new();
        let mut player_x: i32 = 25;
        let mut player_y: i32 = 23;

        for _ in 0..MAX_ROOMS {
            // random width and height
            let w = rand::thread_rng().gen_range(ROOM_MIN_SIZE..=ROOM_MAX_SIZE);
            let h = rand::thread_rng().gen_range(ROOM_MIN_SIZE..=ROOM_MAX_SIZE);
            // random possition walidating screen boarders
            let x = rand::thread_rng().gen_range(0..MAP_WIDTH - w);
            let y = rand::thread_rng().gen_range(0..MAP_HEIGHT - h);
            let mut new_room = Room::new(x, y, w, h);

            let failed = rooms.iter().any(|other_room| new_room.includes(other_room));

            if !failed {
                // if no intersections, it is a valid room
                new_room.put_on_map(map);

                let (new_x, new_y) = new_room.get_center();

                if rooms.is_empty() {
                    // first room, and player coordiantes
                    player_x = new_x;
                    player_y = new_y;
                } else {
                    let (prev_x, prev_y) = rooms[rooms.len() - 1].get_center();

                    // simulate coin flip
                    let is_horizontal = rand::random::<bool>();
                    if is_horizontal {
                        Room::create_h_tunel(prev_x, new_x, prev_y, map);
                        Room::create_v_tunel(prev_y, new_y, new_x, map);
                    } else {
                        Room::create_v_tunel(prev_y, new_y, prev_x, map);
                        Room::create_h_tunel(prev_x, new_x, new_y, map);
                    }
                }
                Entity::populate_room(&mut new_room, entities);
                rooms.push(new_room);
            }
        }
        entities[PLAYER].set_position(player_x, player_y);
    }
}
