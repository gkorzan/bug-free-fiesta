pub const MAP_WIDTH: i32 = 80;
pub const MAP_HEIGHT: i32 = 45;

pub type Map = Vec<Vec<Tile>>;

#[derive(Clone, Copy, Debug)]
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
}
