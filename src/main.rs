use std::{
    convert::{TryFrom, TryInto},
    env, fs,
    io::{self, BufRead, Write},
};

use conduit::{Config, Database};
use ruma::{user_id, RoomAliasId, RoomId, RoomIdOrAliasId};

type EasyErr = Result<(), Box<dyn std::error::Error>>;

const HELP_MSG: &str = "Usage:
    luge </path/to/db> [file=</path/to/file>]

If a file is specified it will be appended to or created if not found

Queries:
    - all <room alias or room id> [string to filter events]
    - pdus <room alias or room id> [string to filter events]
    - rooms
";

#[tokio::main]
async fn main() -> EasyErr {
    let mut args = env::args().collect::<Vec<_>>();

    if args.iter().any(|s| s.contains("help")) {
        println!("{}", HELP_MSG);
        return Ok(());
    }

    for arg in args.iter_mut() {
        *arg = arg.trim().to_string();
    }

    let mut writer: Box<dyn Write> = if args.iter().any(|s| s.starts_with("file=")) {
        let pos = args.iter().position(|it| it.starts_with("file=")).unwrap();
        let arg = args.remove(pos);
        let path = arg.strip_prefix("file=").unwrap();
        Box::new(
            fs::OpenOptions::new()
                .write(true)
                .create(true)
                .append(true)
                .open(path)?,
        )
    } else {
        Box::new(io::stdout())
    };

    args.retain(|s| !s.is_empty());

    match args.as_slice() {
        [_, path] => {
            let mut config = Config::development();
            config.extras.insert(
                "database_path".to_owned(),
                path.trim_end().to_string().into(),
            );

            let db = Database::load_or_create(&config).unwrap();
            loop {
                print!("What you looking for: ");
                io::stdout().flush()?;

                // This works since every time we loop we need a flush
                writer.flush()?;

                let stdin = std::io::stdin();
                let line = {
                    let mut line = String::new();
                    stdin.lock().read_line(&mut line).unwrap();
                    line
                };

                match line.trim_end().split(' ').collect::<Vec<_>>().as_slice() {
                    ["all", room] => {
                        dump_all_events(&mut writer, &db, RoomIdOrAliasId::try_from(*room)?, None)?
                    }
                    ["all", room, filter] => dump_all_events(
                        &mut writer,
                        &db,
                        RoomIdOrAliasId::try_from(*room)?,
                        Some(filter),
                    )?,
                    ["pdus", room] => {
                        dump_state_pdus(&mut writer, &db, RoomIdOrAliasId::try_from(*room)?, None)?
                    }
                    ["pdus", room, filter] => dump_state_pdus(
                        &mut writer,
                        &db,
                        RoomIdOrAliasId::try_from(*room)?,
                        Some(filter),
                    )?,
                    ["rooms"] => dump_rooms(&mut writer, &db)?,
                    ["size"] => dump_size(&mut writer, &db)?,
                    ["help"] | [""] => {
                        println!("{}", HELP_MSG);
                        io::stdout().flush()?;
                    }
                    ["exit"] | ["e"] => return Ok(()),
                    _ => panic!("not a recognized option"),
                }
            }
        }
        [] | [..] => panic!("Need path to DB"),
    }
}

fn print_state_pdus(
    write: &mut dyn Write,
    db: &Database,
    room: &RoomId,
    filter: Option<&str>,
) -> EasyErr {
    use itertools::Itertools;

    for pdu in db
        .rooms
        .room_state_full(room)?
        .values()
        .sorted_by_key(|pdu| pdu.origin_server_ts)
    {
        if let Some(filter) = filter {
            let pretty = serde_json::to_string_pretty(pdu)?;
            if pretty.contains(filter) {
                writeln!(write, "{}", pretty)?;
            }
        } else {
            let pretty = serde_json::to_string_pretty(pdu)?;
            writeln!(write, "{}", pretty)?;
        }
    }
    Ok(())
}

fn dump_state_pdus(
    write: &mut dyn Write,
    db: &Database,
    room: RoomIdOrAliasId,
    filter: Option<&str>,
) -> EasyErr {
    let res: Result<RoomId, RoomAliasId> = room.try_into();
    match res {
        Ok(rid) => print_state_pdus(write, db, &rid, filter)?,
        Err(id) => {
            let rid = db
                .rooms
                .id_from_alias(&id)?
                .expect("No room with that alias, use the form #room:server");
            print_state_pdus(write, db, &rid, filter)?;
        }
    }
    Ok(())
}

fn print_all_pdus(
    write: &mut dyn Write,
    db: &Database,
    room: &RoomId,
    filter: Option<&str>,
) -> EasyErr {
    use itertools::Itertools;

    for pdu in db
        .rooms
        .pdus_after(&user_id!("@fakeuser:fake.com"), room, 0)
        .flat_map(|pair| Some(pair.ok()?.1))
        .sorted_by_key(|pdu| pdu.origin_server_ts)
    {
        if let Some(filter) = filter {
            let pretty = serde_json::to_string_pretty(&pdu)?;
            if pretty.contains(filter) {
                writeln!(write, "{}", pretty)?;
            }
        } else {
            let pretty = serde_json::to_string_pretty(&pdu)?;
            writeln!(write, "{}", pretty)?;
        }
    }
    Ok(())
}

fn dump_all_events(
    write: &mut dyn Write,
    db: &Database,
    room: RoomIdOrAliasId,
    filter: Option<&str>,
) -> EasyErr {
    let res: Result<RoomId, RoomAliasId> = room.try_into();
    match res {
        Ok(rid) => print_all_pdus(write, db, &rid, filter)?,
        Err(id) => {
            let rid = db
                .rooms
                .id_from_alias(&id)?
                .expect("No room with that alias, use the form #room:server");
            print_all_pdus(write, db, &rid, filter)?;
        }
    }
    Ok(())
}

fn dump_rooms(write: &mut dyn Write, db: &Database) -> EasyErr {
    writeln!(
        write,
        "Rooms for server: {}",
        db.globals.server_name().as_str()
    )?;
    for id in db.rooms.public_rooms() {
        writeln!(write, "{}", id?.as_str())?;
    }
    Ok(())
}

fn dump_size(write: &mut dyn Write, db: &Database) -> EasyErr {
    writeln!(
        write,
        "Size for server: {}",
        db.globals.server_name().as_str()
    )?;
    writeln!(
        write,
        "SIZE{}ELEMENTS{}BYTES",
        " ".repeat(30 - 4),
        " ".repeat(30 - 8),
    )?;
    for id in db._db.tree_names() {
        let tree = db._db.open_tree(&id)?;

        let name = String::from_utf8_lossy(&id);
        let name_pad = 30_usize.saturating_sub(name.len());
        let num_el = tree.len();
        let el_pad = 30_usize.saturating_sub(num_el.to_string().len());

        let bytes = tree
            .iter()
            .flat_map(|pair| pair.ok())
            .fold(0, |mut acc, (key, val)| {
                acc += key.len() + val.len();
                acc
            });

        writeln!(
            write,
            "{}{}{}{}{}",
            name,
            " ".repeat(name_pad),
            num_el,
            " ".repeat(el_pad),
            bytes
        )?;
    }
    Ok(())
}
