# luge
## Sledding tubes
### Inspect a [Conduit](https://gitlab.com/famedly/conduit) [Sled](https://github.com/spacejam/sled) database in the terminal

## Use
So far we can print public rooms by RoomId and dump the state events of a specified room.
```bash
luge </path/to/db> [file=</path/to/file>]
```
`file` is the file to direct output to, if not specified stdout is assumed. If the file is not found one can be created by running the program.

This will prompt for info about what your query is
 - `pdus #alias:foo.com/!room_id:foo.com [filter string]` will print all state events of the specified room
 - `rooms` will print all room_ids the database knows as public rooms
 - `size` will print a table of `Tree` name, number of elements in the `Tree` and a count of "bytes" (the number of items in each `IVec`, both keys and values)
 - `exit/e/` exits the program

 ## Notes
 When printing events they are sorted by `origin_server_timestamp` for consistency.