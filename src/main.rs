mod entity;
mod tile;

use entity::Entity;
use tcod::colors::{Color, WHITE, YELLOW};
use tcod::console::{blit, BackgroundFlag, Console, FontLayout, FontType, Offscreen, Root};
use tcod::input::Key;
use tcod::input::KeyCode::*;
use tile::{Map, Tile};

struct Tcod {
    root: Root,
    con: Offscreen,
}

struct Game {
    map: Map,
}

const FONT_SIZE: i32 = 10;
const SCREEN_WIDTH: i32 = 8 * FONT_SIZE;
const SCREEN_HEIGHT: i32 = 5 * FONT_SIZE;

const MAP_WIDTH: i32 = 80;
const MAP_HEIGHT: i32 = 45;

const COLOR_DARK_WALL: Color = Color { r: 0, g: 0, b: 100 };
const COLOR_DARK_GROUND: Color = Color {
    r: 50,
    g: 50,
    b: 150,
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

    let con = Offscreen::new(SCREEN_WIDTH, SCREEN_HEIGHT);

    let mut tcod = Tcod { root, con };

    let game: Game = Game { map: make_map() };

    let player = entity::Entity::new(SCREEN_WIDTH / 2, SCREEN_HEIGHT / 2, '@', WHITE);
    let npc = entity::Entity::new(SCREEN_WIDTH / 2 - 5, SCREEN_HEIGHT / 2, '@', YELLOW);

    let mut entities = [player, npc];
    // main game loop
    while !tcod.root.window_closed() {
        // prepare and draw scene
        tcod.con.set_default_foreground(WHITE);
        tcod.con.clear();

        render_all(&mut tcod, &game, &entities);

        // draw everything
        tcod.root.flush();

        let key = tcod.root.wait_for_keypress(true);
        let player = &mut entities[0];

        // game controls
        player_controls(key, player, &game.map);
        let is_exit_presed = system_controls(key, &mut tcod.root);

        if is_exit_presed {
            break;
        }
        // end of the main loop
    }
}

fn make_map() -> Map {
    let mut map = vec![vec![Tile::empty(); MAP_HEIGHT as usize]; MAP_WIDTH as usize];
    map[30][22] = Tile::wall();
    map[50][22] = Tile::wall();

    map
}

fn render_all(tcod: &mut Tcod, game: &Game, entities: &[Entity]) {
    // draw all entities from list
    for entity in entities {
        entity.draw(&mut tcod.con);
    }
    // draw the map
    for y in 0..MAP_HEIGHT {
        for x in 0..MAP_WIDTH {
            let wall = game.map[x as usize][y as usize].get_is_block_sight();
            if wall {
                tcod.con
                    .set_char_background(x, y, COLOR_DARK_WALL, BackgroundFlag::Set);
            } else {
                tcod.con
                    .set_char_background(x, y, COLOR_DARK_GROUND, BackgroundFlag::Set);
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
}

fn player_controls(key: Key, player: &mut entity::Entity, map: &Map) {
    // charecter movement,
    match key {
        Key { code: Up, .. } => player.move_by(0, -1, map),
        Key { code: Down, .. } => player.move_by(0, 1, map),
        Key { code: Left, .. } => player.move_by(-1, 0, map),
        Key { code: Right, .. } => player.move_by(1, 0, map),

        _ => {}
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
