use castlemaker::*;
use tokio::{io, net::TcpStream, prelude::*};

fn get_world(stream: &mut TcpStream) -> Result<World, Fail> {
    match receive_packet::<FromServer>(stream)? {
        Some(FromServer::SendWorld(world)) => Ok(world),
        None => Err::<_, io::Error>(io::ErrorKind::TimedOut.into())?,
        _ => Err::<_, io::Error>(io::ErrorKind::InvalidInput.into())?,
    }
}

fn main() {
    let mut stream = TcpStream::connect("127.0.0.1:4335").unwrap();
    let mut world = get_world(&mut stream).unwrap();
    for (id, player) in world.players.iter() {
        println!("id {}: {}", id, player.name);
    }
    println!("{}", world.maps[&0]);
    for _ in 0..2 {
        println!("move");
        send_packet::<FromClient>(&mut stream, FromClient::MoveDir(Dir::Right)).unwrap();
        world = get_world(&mut stream).unwrap();
        println!("{}", world.maps[&0]);
    }
}
