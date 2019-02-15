use castlemaker::*;
use rmp_serde::{Serializer, Deserializer};
use serde::{Serialize, Deserialize};
use smush::{decode, encode, Encoding, Quality};
use std::{
    io::{self, Read, Write},
    net::*,
    sync::{Arc, RwLock},
    thread,
};

type SharedWorld = Arc<RwLock<World>>;

fn send_packet(stream: &mut TcpStream, packet: FromServer) -> Result<(), Fail> {
    let mut serialized = vec![];
    packet.serialize(&mut Serializer::new(&mut serialized))?;
    let compressed = encode(&serialized, Encoding::Lz4, Quality::Default)?;
    stream.write_all(&compressed)?;
    Ok(())
}

fn receive_packet(stream: &mut TcpStream) -> Result<FromClient, Fail> {
    let mut buffer = vec![];
    io::copy(stream, &mut buffer)?;
    let decompressed = decode(&buffer, Encoding::Lz4)?;
    let mut de = Deserializer::new(io::Cursor::new(decompressed));
    Ok(Deserialize::deserialize(&mut de)?)
}

fn handle_client(mut stream: TcpStream, world: SharedWorld) -> Result<(), Fail> {
    world.write().unwrap().add_player(
        "Lolz",
        MapLoc {
            map_id: 0,
            pos: (2, 3),
        },
    );
    println!("client handled");
    send_packet(&mut stream, FromServer::SendWorld(world.read().unwrap().clone()))?;
    Ok(())
}

fn main() {
    let world = Arc::new(RwLock::new(World::new()));
    let listener = TcpListener::bind("127.0.0.1:4335")
        .expect("tcplistener bind error");
    println!("Server started");
    for stream in listener.incoming() {
        let new_world = Arc::clone(&world);
        thread::spawn(move || {
            handle_client(stream.expect("client error"), new_world).unwrap();
        });
    }
    println!("Server closed");
}
