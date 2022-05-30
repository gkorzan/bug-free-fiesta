use serde::{Serialize, Deserialize};

use crate::entity::Entity;

pub const MAP_WIDTH: i32 = 80;
pub const MAP_HEIGHT: i32 = 43;

pub type Map = Vec<Vec<Tile>>;

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct Tile {
    passable: bool,
    block_sight: bool,
    explored: bool,
}

impl Tile {
    pub fn empty() -> Self {
        Tile {
            passable: true,
            block_sight: false,
            explored: false,
        }
    }
    pub fn wall() -> Self {
        Tile {
            passable: false,
            block_sight: true,
            explored: false,
        }
    }
    pub fn get_is_block_sight(&self) -> bool {
        self.block_sight
    }
    pub fn get_is_passable(&self) -> bool {
        self.passable
    }
    pub fn get_is_explored(&self) -> bool {
        self.explored
    }
    pub fn explore(&mut self) {
        self.explored = true;
    }
    pub fn is_blocked(x: i32, y: i32, map: &Map, entities: &[Entity]) -> bool {
        let is_tile_passable = map[x as usize][y as usize].get_is_passable();
        if !is_tile_passable {
            return true;
        }

        entities
            .iter()
            .any(|entity| entity.get_is_blocks() && entity.get_coordinates() == (x, y))
    }
}
