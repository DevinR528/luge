use std::{
    convert::{TryFrom, TryInto},
    io::{self, BufRead},
};

use conduit::{Config, Database, PduEvent};
use ruma::{EventId, RoomAliasId, RoomId, RoomIdOrAliasId, UserId};

type EasyErr = Result<(), Box<dyn std::error::Error>>;

fn main() -> EasyErr {
    let args = std::env::args().collect::<Vec<_>>();
    match args.as_slice() {
        [_, path] => {
            let mut config = Config::development();
            config
                .extras
                .insert("database_path".to_owned(), path.to_string().into());

            let db = Database::load_or_create(&config).unwrap();
            loop {
                println!("What you looking for: ");
                let stdin = std::io::stdin();
                let line = {
                    let mut line = String::new();
                    stdin.lock().read_line(&mut line).unwrap();
                    line
                };
                match line.trim_end().split(' ').collect::<Vec<_>>().as_slice() {
                    ["pdus", room] => dump_pdus(&db, RoomIdOrAliasId::try_from(*room)?)?,
                    ["rooms"] => dump_rooms(&db)?,
                    ["exit"] | ["e"] | [""] => return Ok(()),
                    _ => panic!("not a recognized option"),
                }
            }
        }
        [] | [..] => panic!("Need path to DB"),
    }
}

fn dump_pdus(db: &Database, room: RoomIdOrAliasId) -> EasyErr {
    let res: Result<RoomId, RoomAliasId> = room.try_into();
    match res {
        Ok(rid) => {
            for pdu in db.rooms.room_state_full(&rid)?.values() {
                let pretty = serde_json::to_string_pretty(pdu)?;
                println!("{}", pretty);
            }
        }
        Err(id) => {
            let rid = db
                .rooms
                .id_from_alias(&id)?
                .expect("No room with that alias, use the form #room:server");
            for pdu in db.rooms.room_state_full(&rid)?.values() {
                let pretty = serde_json::to_string_pretty(pdu)?;
                println!("{}", pretty);
            }
        }
    }
    Ok(())
}

fn dump_rooms(db: &Database) -> EasyErr {
    println!("Rooms for server: {}", db.globals.server_name().as_str());
    for id in db.rooms.public_rooms() {
        println!("{}", id?.as_str())
    }
    Ok(())
}
