mod countermap;

use serde::{Deserialize, Serialize};
use countermap::*;
use std::{
    collections::{HashMap, HashSet},
    error::Error,
    fmt::{self, Display},
    ops::{Index, IndexMut},
};

pub const TILE_WIDTH: usize = 16;
pub const TILE_HEIGHT: usize = 16;

pub type Fail = Box<dyn Error>;
type MapId = usize;
type EntityId = usize;
type PlayerId = usize;
type Pos = (u16, u16);

#[derive(Serialize, Deserialize)]
pub enum FromServer {
    SendWorld(World),
    Noop,
}

#[derive(Serialize, Deserialize)]
pub enum Dir {
    Up, Down, Left, Right,
}

#[derive(Serialize, Deserialize)]
pub enum FromClient {
    Handshake,
    MoveDir(Dir),
}

#[derive(Clone, Serialize, Deserialize)]
pub struct World {
    pub maps: CounterMap<MapId, Map>,
    pub players: CounterMap<PlayerId, Player>,
}

impl World {
    pub fn new() -> Self {
        let mut maps = CounterMap::new();
        maps.push(Map::new(20, 12));
        Self {
            maps,
            players: CounterMap::new(),
        }
    }

    pub fn add_player<S: Into<String>>(&mut self, name: S, loc: MapLoc) -> Option<PlayerId> {
        let MapLoc { map_id, pos } = loc;
        self.maps
            .get_mut(&map_id)
            .and_then(|m| m.add_entity(pos, b'P'))
            .and_then(|entity_id| {
                let player_id = self.players.len();
                self.players
                    .insert(
                        player_id,
                        Player {
                            name: name.into(),
                            map_id,
                            entity_id,
                        },
                    )
                    .map(|_| player_id)
            })
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Entity {
    pos: Pos,
    sprite: u8,
    character: Char,
}

impl Entity {
    pub fn new(pos: Pos, sprite: u8) -> Self {
        Self {
            pos, sprite, character: Char::default(),
        }
    }
}

#[derive(Clone, Default, Serialize, Deserialize)]
pub struct Char {
    hp: u16,
    class: Class,
}  

impl Char {
    pub fn base_attack_bonus(&self) -> u8 {
        0
    }
}

#[derive(Clone, Default, Serialize, Deserialize)]
pub struct Class {
    name: String,
    hit_die: u8,
    skills: Vec<Skill>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Skill {
    name: String,
    properties: HashSet<Attribute>,
}

#[derive(Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Ability {
    name: String,
    key_attribute: Attribute,
    untrained: bool,
    armor_check_penalty: bool,
}

#[derive(Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Attribute {
    #[serde(rename = "Str")] Strength,
    #[serde(rename = "Dex")] Dexterity,
    #[serde(rename = "Con")] Constitution,
    #[serde(rename = "Int")] Intelligence,
    #[serde(rename = "Wis")] Wisdom,
    #[serde(rename = "Cha")] Charisma,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Player {
    pub name: String,
    pub map_id: MapId,
    pub entity_id: EntityId,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Map {
    map: Box<[Tile]>,
    width: u16,
    height: u16,
    entities: CounterMap<EntityId, Entity>,
}

impl Map {
    pub fn new(width: u16, height: u16) -> Self {
        let mut map = vec![Tile::Ground; width as usize * height as usize].into_boxed_slice();
        for x in 0..width {
            for y in 0..height {
                if x == 0 || x == (width - 1) || y == 0 || y == (height - 1) {
                    let index = y as usize * width as usize + x as usize;
                    map[index] = Tile::Wall;
                }
            }
        }
        let entities = CounterMap::new();
        Self {
            map,
            width,
            height,
            entities,
        }
    }

    pub fn add_entity(&mut self, pos: Pos, sprite: u8) -> Option<EntityId> {
        if self[pos] == Tile::Ground {
            self.entities.push(Entity::new(pos, sprite))
        } else {
            None
        }
    }
}

impl Index<Pos> for Map {
    type Output = Tile;
    fn index(&self, (x, y): Pos) -> &Self::Output {
        assert!(x < self.width && y < self.height);
        &self.map[y as usize * self.width as usize + x as usize]
    }
}

impl IndexMut<Pos> for Map {
    fn index_mut(&mut self, (x, y): Pos) -> &mut Self::Output {
        assert!(x < self.width && y < self.height);
        &mut self.map[y as usize * self.width as usize + x as usize]
    }
}

impl Display for Map {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for y in 0..self.height {
            for x in 0..self.width {
                let entity = self.entities.iter()
                    .find(|(_, v)| v.pos == (x, y))
                    .map(|(_, v)| v.sprite as char);
                let ch = if let Some(ch) = entity {
                    ch
                } else {
                    match self[(x as _, y as _)] {
                        Tile::Ground => '.',
                        Tile::Wall => '#',
                        Tile::Link(_) => 'L',
                        Tile::Door(false) => 'D',
                        Tile::Door(true) => 'I',
                    }
                };
                write!(f, "{}", ch)?;
            }
            writeln!(f)?;
        }
        Ok(())
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct MapLoc {
    pub map_id: MapId,
    pub pos: Pos,
}

#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Tile {
    Ground,
    Wall,
    // A "teleport" between maps
    Link(MapLoc),
    Door(bool),
}
