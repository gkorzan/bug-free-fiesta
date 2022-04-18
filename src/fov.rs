use tcod::map::{FovAlgorithm, Map as FovMap};

use crate::tile::{Map, MAP_HEIGHT, MAP_WIDTH};

pub const FOV_ALGO: FovAlgorithm = FovAlgorithm::Basic;
pub const FOV_LIGHT_WALLS: bool = true;
pub const TORCH_RADIUS: i32 = 10;

pub fn generate_fov_map(fov: &mut FovMap, map: &Map) {
    for y in 0..MAP_HEIGHT {
        for x in 0..MAP_WIDTH {
            let transparent = !map[x as usize][y as usize].get_is_block_sight();
            let walkable = map[x as usize][y as usize].get_is_passable();
            fov.set(x, y, transparent, walkable);
        }
    }
}
