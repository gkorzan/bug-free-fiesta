mod entity;
mod fov;
mod message;
mod panel;
mod room;
mod tile;

use entity::{DeathCallback, Entity, Item, UseResult, PLAYER};
use fov::generate_fov_map;
use message::{Messages, MSG_HEIGHT, MSG_WIDTH, MSG_X};
use panel::render_bar;
use room::Room;
use tcod::colors::{Color, BLACK, DARKER_RED, LIGHT_GREY, LIGHT_RED, RED, WHITE, YELLOW};
use tcod::console::{blit, BackgroundFlag, Console, FontLayout, FontType, Offscreen, Root};
use tcod::input::{self, KeyCode::*};
use tcod::input::{Event, Key, Mouse};
use tcod::map::Map as FovMap;
use tcod::TextAlignment;
use tile::{Map, Tile, MAP_HEIGHT, MAP_WIDTH};

pub struct Tcod {
    root: Root,
    con: Offscreen,
    panel: Offscreen,
    fov: FovMap,
    key: Key,
    mouse: Mouse,
}

pub struct Game {
    map: Map,
    messages: Messages,
    inventory: Vec<Entity>,
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
const INVENTORY_WIDTH: i32 = 50;

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
    let key = Default::default();
    let mouse = Default::default();

    let mut tcod = Tcod {
        root,
        con,
        panel,
        fov,
        key,
        mouse,
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
    let messages = Messages::new();
    let inventory: Vec<Entity> = vec![];

    let mut game: Game = Game {
        map,
        messages,
        inventory,
    };
    game.messages.add(
        "Welcome stranger! Prepre to perish in the Tombs of the Ancient Kings.",
        RED,
    );

    // main game loop
    while !tcod.root.window_closed() {
        // prepare and draw scene
        tcod.con.set_default_foreground(WHITE);
        tcod.con.clear();

        match input::check_for_event(input::MOUSE | input::KEY_PRESS) {
            Some((_, Event::Mouse(m))) => tcod.mouse = m,
            Some((_, Event::Key(k))) => tcod.key = k,
            _ => tcod.key = Default::default(),
        }

        render_all(&mut tcod, &mut game, &entities, previous_player_position);

        // draw everything
        tcod.root.flush();

        // let key = tcod.root.wait_for_keypress(true);

        // game controls
        previous_player_position = entities[PLAYER].get_coordinates();
        let took_turn = player_controls(tcod.key, &mut game, &mut entities, &mut tcod);
        let is_exit_presed = system_controls(tcod.key, &mut tcod.root);
        Entity::mobs_turn(&mut game, &tcod.fov, &mut entities, took_turn);
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

    tcod.panel.set_default_foreground(LIGHT_GREY);
    tcod.panel.print_ex(
        1,
        0,
        BackgroundFlag::None,
        TextAlignment::Left,
        get_names_under_mouse(tcod.mouse, entities, &tcod.fov),
    );

    let mut y = MSG_HEIGHT as i32;
    for &(ref msg, color) in game.messages.iter().rev() {
        let msg_height = tcod.panel.get_height_rect(MSG_X, y, MSG_WIDTH, 0, msg);
        y -= msg_height;
        if y < 0 {
            break;
        }
        tcod.panel.set_default_foreground(color);
        tcod.panel.print_rect(MSG_X, y, MSG_WIDTH, 0, msg);
    }

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

fn player_controls(key: Key, game: &mut Game, entities: &mut Vec<Entity>, tcod: &mut Tcod) -> bool {
    // charecter movement,
    match (key, key.text(), entities[PLAYER].is_alive()) {
        (Key { code: Up, .. }, _, _) => {
            Entity::player_move_or_attack(PLAYER, 0, -1, game, entities);
            true
        }
        (Key { code: Down, .. }, _, _) => {
            Entity::player_move_or_attack(PLAYER, 0, 1, game, entities);
            true
        }
        (Key { code: Left, .. }, _, _) => {
            Entity::player_move_or_attack(PLAYER, -1, 0, game, entities);
            true
        }
        (Key { code: Right, .. }, _, _) => {
            Entity::player_move_or_attack(PLAYER, 1, 0, game, entities);
            true
        }
        (Key { code: Text, .. }, "g", true) => {
            let item_id = entities.iter().position(|entity| {
                entity.get_coordinates() == entities[PLAYER].get_coordinates()
                    && entity.get_item().is_some()
            });
            if let Some(item_id) = item_id {
                Entity::pick_item_up(item_id, game, entities);
            }
            false
        }
        (Key { code: Text, .. }, "i", true) => {
            let inventory_index = inventory_menu(
                &game.inventory,
                "Press the key next to an item or any ohter to close menu\n",
                &mut tcod.root,
            );
            if let Some(inventory_index) = inventory_index {
                use_item(inventory_index, tcod, game, entities);
            }
            false
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

fn get_names_under_mouse(mouse: Mouse, entities: &[Entity], fov_map: &FovMap) -> String {
    let (x, y) = (mouse.cx as i32, mouse.cy as i32);

    let names = entities
        .iter()
        .filter(|ent| {
            ent.get_coordinates() == (x, y)
                && fov_map.is_in_fov(ent.get_coordinates().0, ent.get_coordinates().1)
        })
        .map(|ent| ent.get_name())
        .collect::<Vec<_>>();

    names.join(", ")
}

fn menu<T: AsRef<str>>(header: &str, options: &[T], width: i32, root: &mut Root) -> Option<usize> {
    assert!(
        options.len() <= 26,
        "Can't have a menu with more than 26 options"
    );
    let header_height = root.get_height_rect(0, 0, width, SCREEN_HEIGHT, header);
    let height = options.len() as i32 + header_height;

    let mut window = Offscreen::new(width, height);

    window.set_default_foreground(WHITE);
    window.print_rect_ex(
        0,
        0,
        width,
        height,
        BackgroundFlag::None,
        TextAlignment::Left,
        header,
    );
    for (index, option_text) in options.iter().enumerate() {
        let menu_letter = (b'a' + index as u8) as char;
        let text = format!("({}) {}", menu_letter, option_text.as_ref());
        window.print_ex(
            0,
            header_height + index as i32,
            BackgroundFlag::None,
            TextAlignment::Left,
            text,
        );
    }
    let x = SCREEN_WIDTH / 2 - width / 2;
    let y = SCREEN_HEIGHT / 2 - height / 2;
    blit(&window, (0, 0), (width, height), root, (x, y), 1.0, 0.7);

    root.flush();
    let key = root.wait_for_keypress(true);

    if key.printable.is_alphabetic() {
        let index = key.printable.to_ascii_lowercase() as usize - 'a' as usize;
        if index < options.len() {
            Some(index)
        } else {
            None
        }
    } else {
        None
    }
}

fn inventory_menu(inventory: &[Entity], header: &str, root: &mut Root) -> Option<usize> {
    let options = if inventory.len() == 0 {
        vec!["Inventory is empty.".into()]
    } else {
        inventory
            .iter()
            .map(|item| item.get_name().clone())
            .collect()
    };
    let inventory_index = menu(header, &options, INVENTORY_WIDTH, root);

    if inventory.len() > 0 {
        inventory_index
    } else {
        None
    }
}

fn use_item(inventory_id: usize, tcod: &Tcod, game: &mut Game, entities: &mut [Entity]) {
    if let Some(item) = game.inventory[inventory_id].get_item() {
        let on_use = match item {
            Item::Heal => Entity::cast_heal,
            Item::Lightning => Entity::cast_lightning,
        };
        match on_use(inventory_id, tcod, game, entities) {
            UseResult::UsedUp => {
                game.inventory.remove(inventory_id);
            }
            UseResult::Cancelled => {
                game.messages.add("Cancelled", WHITE);
            }
        }
    } else {
        game.messages.add(
            format!(
                "The {} cannot be used.",
                game.inventory[inventory_id].get_name()
            ),
            WHITE,
        )
    }
}
