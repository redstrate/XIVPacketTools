# XIVPacketTools

A collection of packet-related tools for a certain MMO.

## cfcap-expand

This extracts packets from captures (`.cfcap`) recorded from [Project Chronofoil](https://github.com/ProjectChronofoil) into a directory for easy browsing.

### Usage

Run using `cargo run`, giving it the path to your `.cfcap` file.

```shell
$ cargo run --bin cfcap-expand 91682a6e-73cc-49ce-a42d-6fe484c46aa5.cfcap
```

Once the process is complete, you will find it has created a structure like:

```
<capture-id>/
    lobby/ <lobby connection>
        ...
    chat/ <chat connection>
        ...
    zone/ <zone connection>
        <index>-<segment-type>-[ipc opcode] (to <client/server>)/
            data.bin
            ipc_header.bin (if ipc)
```

Notes:
* `<index>` is indexed *across every connection*, so it will usually start in Lobby and ends in Zone.
* `ipc_header.bin` is the IPC segment header data, if applicable.
* `data.bin` is the data of the segment with the headers stripped out.
* `ipc opcode` is in decimal format.

#### Translating IPC opcodes

To associate IPC opcodes with a well defined name, place an `opcodes.json` in the same directory where you run `cfcap-expand`. It should be in the following format:

```json
{
    "ServerZoneIpcType": [
        {
          "name": "WellDefinedOpcode",
          "opcode": 258
        }
    ],
    "ClientZoneIpcType": [
        {
          "name": "AnotherWellDefinedOpcode",
          "opcode": 250
        }
    ],
    "ServerLobbyIpcType": [],
    "ClientLobbyIpcType": [],
    "ServerChatIpcType": [],
    "ServerChatIpcType": [],
}
```

Segments already have well-defined names, so it's not possible to change those at the moment. Opcodes should be written in decimal. This format is intentionally similiar to the one used in [FFXIVOpcodes](https://github.com/karashiiro/FFXIVOpcodes/), so it's easier to move opcodes between.

## opcode-update

Updates a list of opcodes based on the an opcode diff from [opcodediff](https://github.com/xivdev/opcodediff). Note that the output isn't 100% correct, due to the nature of the process. This tool will try not to overwrite the same opcode twice.

Run using `cargo run`, giving it the path to the `.diff.json` file and the `opcodes.json`:

```shell
$ cargo run --bin opcode-update 7.21.diff.json opcodes.json
```

The file passed as the second argument will be modified in-place.

## License

![GPLv3](https://www.gnu.org/graphics/gplv3-127x51.png)

This project is licensed under the GNU General Public License 3. The `ChronofoilCaptureFile.proto` definition file is from [Project Chronofoil](https://github.com/ProjectChronofoil/Chronofoil.CaptureFile/blob/main/src/Proto/ChronofoilCaptureFile.proto).
