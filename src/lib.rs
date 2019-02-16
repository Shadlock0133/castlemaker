mod countermap;
//mod char;

use countermap::*;
use rmp_serde::{Serializer, Deserializer};
use serde::{Serialize, Deserialize};
use smush::{decode, encode, Encoding, Quality};
use tokio::{
    io,
    net::TcpStream,
    prelude::*,
};
use std::{
    error::Error,
    io::Cursor,
    fmt::{self, Debug, Display},
    ops::{Index, IndexMut},
};

pub const TILE_WIDTH: usize = 16;
pub const TILE_HEIGHT: usize = 16;

pub type Fail = Box<dyn Error>;
type MapId = usize;
type EntityId = usize;
type PlayerId = usize;
type Pos = (u16, u16);

pub fn send_packet<T: Serialize>(stream: &mut TcpStream, packet: T) -> Result<(), Fail> {
    eprint!("sending... ");
    let mut serialized = vec![];
    packet.serialize(&mut Serializer::new(&mut serialized))?;
    let compressed = encode(&serialized, Encoding::Lz4, Quality::Default)?;
    stream.write_all(&compressed)?;
    stream.flush()?;
    eprintln!("done");
    Ok(())
}

pub fn receive_packet<'de, T: Deserialize<'de>>(stream: &mut TcpStream) -> Result<Option<T>, Fail> {
    eprint!("receiving... ");
    let mut buffer = vec![];
    io::copy(stream, &mut Cursor::new(buffer))
        .and_then(|| {
            let decompressed = decode(&buffer, Encoding::Lz4).unwrap();
            let mut de = Deserializer::new(Cursor::new(decompressed));
            let res = Deserialize::deserialize(&mut de).unwrap();
            eprintln!("done");
            Ok(Some(res))
        }).wait()
}

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
    MoveDir(Dir),
    Noop,
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
            .map(|entity_id| {
                self.players
                    .push(
                        Player {
                            name: name.into(),
                            map_id,
                            entity_id,
                        },
                    )
            })
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Entity {
    pos: Pos,
    sprite: u8,
    //name: String,
    //character: Char,
}

impl Entity {
    pub fn new(pos: Pos, sprite: u8) -> Self {
        Self {
            pos, sprite, //character: Char::default(),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
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
        let occupied = self.entities.values().find(|v| v.pos == pos).is_some();
        if self[pos] == Tile::Ground && !occupied {
            Some(self.entities.push(Entity::new(pos, sprite)))
        } else {
            None
        }
    }
    
    pub fn move_entity(&mut self, entity_id: EntityId, dir: Dir) {
        let (old_x, old_y) = self.entities[&entity_id].pos;
        self.entities.get_mut(&entity_id).unwrap().pos = match dir {
            Dir::Up => (old_x, old_y - 1),
            Dir::Down => (old_x, old_y + 1),
            Dir::Left => (old_x - 1, old_y),
            Dir::Right => (old_x + 1, old_y),
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
                let entity = self.entities.values()
                    .find(|v| v.pos == (x, y))
                    .map(|v| v.sprite as char);

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

impl Debug for Map {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MapLoc {
    pub map_id: MapId,
    pub pos: Pos,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Tile {
    Ground,
    Wall,
    // A "teleport" between maps
    Link(MapLoc),
    Door(bool),
}
