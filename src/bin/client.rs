use castlemaker::*;
use smush::{decode, encode, Encoding, Quality};
use rmp_serde::{Serializer, Deserializer};
use serde::{Serialize, Deserialize};
use std::{
    io::{self, Read, Write},
    net::*,
};

fn send_packet(stream: &mut TcpStream, packet: FromClient) -> Result<(), Fail> {
    let mut serialized = vec![];
    packet.serialize(&mut Serializer::new(&mut serialized))?;
    let compressed = encode(&serialized, Encoding::Lz4, Quality::Default)?;
    stream.write_all(&compressed)?;
    Ok(())
}

fn receive_packet(stream: &mut TcpStream) -> Result<FromServer, Fail> {
    let mut buffer = vec![];
    io::copy(stream, &mut buffer)?;
    let decompressed = decode(&buffer, Encoding::Lz4)?;
    let mut de = Deserializer::new(io::Cursor::new(decompressed));
    Ok(Deserialize::deserialize(&mut de)?)
}

fn get_world(stream: &mut TcpStream) -> Result<World, Fail> {
    if let FromServer::SendWorld(world) = receive_packet(stream)? {
        Ok(world)
    } else {
        Err(io::Error::from(io::ErrorKind::InvalidInput).into())
    }
}

fn main() {
    let mut stream = TcpStream::connect("127.0.0.1:4335").unwrap();
    let mut world = get_world(&mut stream).unwrap();
    for (id, player) in world.players.iter() {
        println!("id {}: {}", id, player.name);
    }
    println!("{}", world.maps[&0]);
}
