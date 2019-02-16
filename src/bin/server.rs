use castlemaker::*;
use std::sync::{Arc, RwLock};
use tokio::{
    io,
    net::{TcpListener, TcpStream},
    prelude::*,
};

type SharedWorld = Arc<RwLock<World>>;

fn handle_client(mut stream: TcpStream, world: SharedWorld) -> Result<(), Fail> {
    let player_id = {
        world
            .write()
            .unwrap()
            .add_player(
                "Lolz",
                MapLoc {
                    map_id: 0,
                    pos: (2, 3),
                },
            )
            .ok_or(Box::new(io::Error::from(io::ErrorKind::Other)))?
    };
    println!("player {} joined", player_id);
    send_packet::<FromServer>(
        &mut stream,
        FromServer::SendWorld(world.read().unwrap().clone()),
    )?;
    loop {
        match receive_packet::<FromClient>(&mut stream)? {
            Some(FromClient::MoveDir(dir)) => {
                println!("got to Move");
                let Player {
                    map_id, entity_id, ..
                } = world.read().unwrap().players[&player_id];
                world
                    .write()
                    .unwrap()
                    .maps
                    .get_mut(&map_id)
                    .ok_or("lols")?
                    .move_entity(entity_id, dir);
            }
            None => (),
            _ => panic!("unknown packet"),
        }
        send_packet::<FromServer>(
            &mut stream,
            FromServer::SendWorld(world.read().unwrap().clone()),
        )?;
    }
}

fn main() {
    let listener =
        TcpListener::bind(&("127.0.0.1:4335".parse().unwrap())).expect("tcplistener bind error");

    let world = Arc::new(RwLock::new(World::new()));
    println!("Server started");
    listener.incoming().for_each(|stream| {
        let new_world = Arc::clone(&world);
        tokio::spawn(move || {
            handle_client(stream, new_world).unwrap();
        });
    });
    println!("Server closed");
}
