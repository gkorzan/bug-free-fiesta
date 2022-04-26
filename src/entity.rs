use rand::Rng;
use tcod::{
    colors::{self, Color, DARK_RED, ORANGE, RED, VIOLET, WHITE},
    Console, Map as FovMap,
};

use crate::{
    message::{self, Messages},
    room::Room,
    tile::{Map, Tile, MAP_HEIGHT, MAP_WIDTH},
    Game,
};

const MAX_ROOM_MONSTERS: i32 = 3;
const MAX_ROOM_ITEMS: i32 = 2;
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
    fighter: Option<Fighter>,
    ai: Option<AI>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum DeathCallback {
    Player,
    Monster,
}
impl DeathCallback {
    fn callback(self, entity: &mut Entity, messages: &mut Messages) {
        use DeathCallback::{Monster, Player};
        let callback: fn(&mut Entity, &mut Messages) = match self {
            Player => Entity::player_death,
            Monster => Entity::monster_death,
        };
        callback(entity, messages);
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Fighter {
    max_hp: i32,
    hp: i32,
    defense: i32,
    power: i32,
    on_death: DeathCallback,
}
impl Fighter {
    pub fn get_hp(&self) -> (i32, i32) {
        (self.hp, self.max_hp)
    }
}

#[derive(Clone, Debug, PartialEq)]
enum AI {
    Basic,
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
            fighter: None,
            ai: None,
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

    pub fn move_towards(
        id: usize,
        target_x: i32,
        target_y: i32,
        map: &Map,
        entities: &mut [Entity],
    ) {
        let dx = target_x - entities[id].x;
        let dy = target_y - entities[id].y;

        let distance = ((dx.pow(2) + dy.pow(2)) as f32).sqrt();
        let dx = (dx as f32 / distance).round() as i32;
        let dy = (dy as f32 / distance).round() as i32;
        Entity::move_by(id, dx, dy, map, entities);
    }

    pub fn distance_to(&self, other: &Entity) -> f32 {
        let dx = other.x - self.x;
        let dy = other.y - self.y;
        ((dx.pow(2) + dy.pow(2)) as f32).sqrt()
    }

    fn mut_two<T>(first_index: usize, second_index: usize, items: &mut [T]) -> (&mut T, &mut T) {
        assert!(first_index != second_index);
        let split_at_index = std::cmp::max(first_index, second_index);
        let (first_slice, second_slice) = items.split_at_mut(split_at_index);
        if first_index < second_index {
            (&mut first_slice[first_index], &mut second_slice[0])
        } else {
            (&mut second_slice[0], &mut first_slice[second_index])
        }
    }

    pub fn ai_take_turn(monster_id: usize, fov: &FovMap, game: &mut Game, entities: &mut [Entity]) {
        let (m_x, m_y) = entities[monster_id].get_coordinates();
        if fov.is_in_fov(m_x, m_y) {
            if entities[monster_id].distance_to(&entities[PLAYER]) >= 2.0 {
                let (p_x, p_y) = entities[PLAYER].get_coordinates();
                Entity::move_towards(monster_id, p_x, p_y, &game.map, entities);
            } else if entities[PLAYER].fighter.map_or(false, |f| f.hp > 0) {
                let (monster, player) = Entity::mut_two(monster_id, PLAYER, entities);
                monster.attack(player, &mut game.messages);
            }
        }
    }

    pub fn player_move_or_attack(
        _id: usize,
        dx: i32,
        dy: i32,
        game: &mut Game,
        entities: &mut [Entity],
    ) {
        if !entities[PLAYER].is_alive() {
            return;
        }
        let (mut x, mut y) = entities[PLAYER].get_coordinates();
        x = x + dx;
        y = y + dy;

        let target_id = entities
            .iter()
            .position(|entity| entity.fighter.is_some() && entity.get_coordinates() == (x, y));

        match target_id {
            Some(target_id) => {
                let (player, target) = Entity::mut_two(PLAYER, target_id, entities);
                player.attack(target, &mut game.messages);
            }
            None => Entity::move_by(PLAYER, dx, dy, &game.map, entities),
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
    pub fn get_name(&self) -> String {
        return self.name.clone();
    }
    pub fn make_alive(&mut self) {
        self.alive = true;
    }
    pub fn kill(&mut self) {
        self.alive = false;
        self.char = '%';
        self.color = DARK_RED;
        self.name = format!("remains of {}", self.name);
    }
    fn player_death(player: &mut Entity, messages: &mut Messages) {
        messages.add("You died!", RED);

        player.kill();
    }
    fn monster_death(monster: &mut Entity, messages: &mut Messages) {
        messages.add(format!("{} is dead!", monster.name), ORANGE);

        monster.kill();
        monster.blocks = false;
        monster.fighter = None;
        monster.ai = None;
    }
    pub fn is_alive(&self) -> bool {
        self.alive
    }
    pub fn take_damage(&mut self, damage: i32, messages: &mut Messages) {
        if let Some(fighter) = self.fighter.as_mut() {
            if damage > 0 {
                fighter.hp -= damage;
            }
        }

        if let Some(fighter) = self.fighter {
            if fighter.hp <= 0 {
                self.alive = false;
                fighter.on_death.callback(self, messages)
            }
        }
    }
    pub fn attack(&mut self, target: &mut Entity, messages: &mut Messages) {
        let damage = self.fighter.map_or(0, |f| f.power) - target.fighter.map_or(0, |f| f.defense);
        if damage > 0 {
            messages.add(
                format!(
                    "{0} attacks {1} for {2} hit points.",
                    self.name, target.name, damage
                ),
                WHITE,
            );
            target.take_damage(damage, messages);
        } else {
            messages.add(
                format!(
                    "{0} attacks {1} but it has no effect!",
                    self.name, target.name
                ),
                WHITE,
            );
        }
    }

    pub fn make_fighter(
        &mut self,
        max_hp: i32,
        hp: i32,
        defense: i32,
        power: i32,
        on_death: DeathCallback,
    ) {
        self.fighter = Some(Fighter {
            max_hp,
            hp,
            defense,
            power,
            on_death,
        })
    }
    pub fn get_fighter(&self) -> Option<Fighter> {
        self.fighter
    }

    pub fn set_ai(&mut self) {
        self.ai = Some(AI::Basic);
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
                    let mut ork = Entity::new(x, y, 'o', colors::DESATURATED_GREEN, "Ork", true);
                    ork.make_fighter(10, 10, 0, 3, DeathCallback::Monster);
                    ork.set_ai();
                    ork
                // generate ORK
                } else {
                    let mut troll = Entity::new(x, y, 'T', colors::DARKER_GREEN, "Troll", true); // gen TROLL
                    troll.make_fighter(16, 16, 1, 4, DeathCallback::Monster);
                    troll.set_ai();
                    troll
                };
                monster.make_alive();
                entities.push(monster);
            }
        }

        // add items to the room
        let num_items = rand::thread_rng().gen_range(0..=MAX_ROOM_ITEMS);

        for _ in 0..num_items {
            let x = rand::thread_rng().gen_range(x1 + 1..x2);
            let y = rand::thread_rng().gen_range(y1 + 1..y2);

            if !Tile::is_blocked(x, y, map, entities) {
                let mut item = Entity::new(x, y, '!', VIOLET, "healing potion", false);
                entities.push(item);
            }
        }
    }

    pub fn mobs_turn(
        game: &mut Game,
        fov: &FovMap,
        entities: &mut [Entity],
        player_took_turn: bool,
    ) {
        if entities[PLAYER].is_alive() && player_took_turn {
            for id in 0..entities.len() {
                if entities[id].ai.is_some() {
                    Entity::ai_take_turn(id, fov, game, entities);
                }
            }
        }
    }
}
