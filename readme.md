# luge
## Sledding tubes
### Inspect a [Conduit](https://gitlab.com/famedly/conduit) [Sled](https://github.com/spacejam/sled) database in the terminal

## Use
So far we can print public rooms by RoomId and dump the state events of a specified room.
```bash
luge /path/to/db [-file=/path/to/file]
```
This will prompt for info about what your query is
 - `pdus #alias:foo.com/!roomid:foo.com [filter string]` will print all state events of the specified room
 - `rooms` will print all room_ids the database knows as public rooms
 - `exit/e/''` exits the program

 ## Notes
 When printing events they are sorted by `origin_server_timestamp` for consistency.