# luge
### Inspect a [Conduit](https://gitlab.com/famedly/conduit) [Sled](https://github.com/spacejam/sled) database in the terminal

## Use
So far we can print public rooms by RoomId and dump the state events of a specified room.
```bash
luge /path/to/db
```
This will prompt for info about what your query is
 - `pdus #alias:foo.com/!roomid:foo.com` will print all state events
 - `rooms` will print all room_ids the database knows as public rooms
 - `exit/e/''` exits the program