mod entity;
mod fov;
mod panel;
mod room;
mod tile;

use entity::{DeathCallback, Entity, PLAYER};
use fov::generate_fov_map;
use panel::render_bar;
use room::Room;
use tcod::colors::{Color, BLACK, DARKER_RED, LIGHT_RED, WHITE, YELLOW};
use tcod::console::{blit, BackgroundFlag, Console, FontLayout, FontType, Offscreen, Root};
use tcod::input::Key;
use tcod::input::KeyCode::*;
use tcod::map::Map as FovMap;
use tcod::TextAlignment;
use tile::{Map, Tile, MAP_HEIGHT, MAP_WIDTH};

struct Tcod {
    root: Root,
    con: Offscreen,
    panel: Offscreen,
    fov: FovMap,
}

struct Game {
    map: Map,
}

const FONT_SIZE: i32 = 10;
const SCREEN_WIDTH: i32 = 8 * FONT_SIZE;
const SCREEN_HEIGHT: i32 = 5 * FONT_SIZE;

pub const BAR_WIDTH: i32 = 20;
pub const PANEL_HEIGHT: i32 = 7;
pub const PANEL_Y: i32 = SCREEN_HEIGHT - PANEL_HEIGHT;

const COLOR_DARK_WALL: Color = Color { r: 0, g: 0, b: 100 };
const COLOR_DARK_GROUND: Color = Color {
    r: 50,
    g: 50,
    b: 150,
};
const COLOR_LIGHT_WALL: Color = Color {
    r: 130,
    g: 110,
    b: 50,
};
const COLOR_LIGHT_GROUND: Color = Color {
    r: 200,
    g: 180,
    b: 50,
};

const LIMIT_FPS: i32 = 24;

fn main() {
    tcod::system::set_fps(LIMIT_FPS);

    let root = Root::initializer()
        .font("terminal10x10_gs_tc.png", FontLayout::Tcod)
        .font_type(FontType::Greyscale)
        .size(SCREEN_WIDTH, SCREEN_HEIGHT)
        .title("Rust/libtcod tutorial")
        .init();

    let con = Offscreen::new(MAP_WIDTH, MAP_HEIGHT);
    let panel = Offscreen::new(SCREEN_WIDTH, PANEL_HEIGHT);
    let fov = FovMap::new(MAP_WIDTH, MAP_HEIGHT);

    let mut tcod = Tcod {
        root,
        con,
        panel,
        fov,
    };

    let mut player = entity::Entity::new(0, 0, '@', WHITE, "Player", true);
    player.make_alive();
    player.make_fighter(30, 30, 2, 5, DeathCallback::Player);
    let mut previous_player_position = (-1, -1);
    let npc = entity::Entity::new(
        SCREEN_WIDTH / 2 - 5,
        SCREEN_HEIGHT / 2,
        '@',
        YELLOW,
        "Frederic",
        true,
    );
    let mut entities = vec![player, npc];

    let mut map = make_map(&mut entities);
    generate_fov_map(&mut tcod.fov, &mut map);

    let mut game: Game = Game { map };

    // main game loop
    while !tcod.root.window_closed() {
        // prepare and draw scene
        tcod.con.set_default_foreground(WHITE);
        tcod.con.clear();

        render_all(&mut tcod, &mut game, &entities, previous_player_position);

        // draw everything
        tcod.root.flush();

        let key = tcod.root.wait_for_keypress(true);

        // game controls
        previous_player_position = entities[PLAYER].get_coordinates();
        let took_turn = player_controls(key, &game.map, &mut entities);
        let is_exit_presed = system_controls(key, &mut tcod.root);
        Entity::mobs_turn(&game.map, &tcod.fov, &mut entities, took_turn);
        if is_exit_presed {
            break;
        }
        // end of the main loop
    }
}

fn make_map(entities: &mut Vec<Entity>) -> Map {
    let mut map = vec![vec![Tile::wall(); MAP_HEIGHT as usize]; MAP_WIDTH as usize];

    Room::generate_rooms(&mut map, entities);

    map
}

fn render_all(
    tcod: &mut Tcod,
    game: &mut Game,
    entities: &[Entity],
    previous_player_position: (i32, i32),
) {
    let current_player_coordinates = entities[PLAYER].get_coordinates();
    let do_calculate_fov = previous_player_position != current_player_coordinates;
    if do_calculate_fov {
        tcod.fov.compute_fov(
            current_player_coordinates.0,
            current_player_coordinates.1,
            fov::TORCH_RADIUS,
            fov::FOV_LIGHT_WALLS,
            fov::FOV_ALGO,
        );
    }
    // draw all entities from list
    let mut to_draw: Vec<_> = entities
        .iter()
        .filter(|e1| {
            let e_pos = e1.get_coordinates();
            tcod.fov.is_in_fov(e_pos.0, e_pos.1)
        })
        .collect();
    to_draw.sort_by(|o1, o2| o1.get_is_blocks().cmp(&o2.get_is_blocks()));
    for entity in to_draw {
        entity.draw(&mut tcod.con);
    }
    // draw the map
    for y in 0..MAP_HEIGHT {
        for x in 0..MAP_WIDTH {
            let current_tile = &mut game.map[x as usize][y as usize];
            let visible = tcod.fov.is_in_fov(x, y);
            let wall = current_tile.get_is_block_sight();

            let color = match (visible, wall) {
                (false, false) => COLOR_DARK_GROUND,
                (false, true) => COLOR_DARK_WALL,
                (true, false) => COLOR_LIGHT_GROUND,
                (true, true) => COLOR_LIGHT_WALL,
            };

            if visible {
                current_tile.explore();
            }
            if current_tile.get_is_explored() {
                tcod.con
                    .set_char_background(x, y, color, BackgroundFlag::Set);
            }
        }
    }
    // place all the Tile
    blit(
        &tcod.con,
        (0, 0),
        (SCREEN_WIDTH, SCREEN_HEIGHT),
        &mut tcod.root,
        (0, 0),
        1.0,
        1.0,
    );

    // prepare to render the GUI panel
    tcod.panel.set_default_background(BLACK);
    tcod.panel.clear();

    let (hp, max_hp) = entities[PLAYER]
        .get_fighter()
        .map_or((0, 0), |f| f.get_hp());
    render_bar(
        &mut tcod.panel,
        1,
        1,
        BAR_WIDTH,
        "HP",
        hp,
        max_hp,
        LIGHT_RED,
        DARKER_RED,
    );

    blit(
        &tcod.panel,
        (0, 0),
        (SCREEN_WIDTH, SCREEN_HEIGHT),
        &mut tcod.root,
        (0, PANEL_Y),
        1.0,
        1.0,
    );
}

fn player_controls(key: Key, map: &Map, entities: &mut [Entity]) -> bool {
    // charecter movement,
    match (key, entities[PLAYER].is_alive()) {
        (Key { code: Up, .. }, _) => {
            Entity::player_move_or_attack(PLAYER, 0, -1, map, entities);
            true
        }
        (Key { code: Down, .. }, _) => {
            Entity::player_move_or_attack(PLAYER, 0, 1, map, entities);
            true
        }
        (Key { code: Left, .. }, _) => {
            Entity::player_move_or_attack(PLAYER, -1, 0, map, entities);
            true
        }
        (Key { code: Right, .. }, _) => {
            Entity::player_move_or_attack(PLAYER, 1, 0, map, entities);
            true
        }

        _ => false,
    }
}

fn system_controls(key: Key, root: &mut Root) -> bool {
    match key {
        // exit game
        Key { code: Escape, .. } => true,

        // set fullscreen
        Key {
            code: Enter,
            alt: true,
            ..
        } => {
            let is_fullscreen = root.is_fullscreen();
            root.set_fullscreen(!is_fullscreen);
            false
        }
        _ => false,
    }
}
