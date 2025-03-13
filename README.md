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
* `ipc opcode` is in hexadecimal format.

## License

![GPLv3](https://www.gnu.org/graphics/gplv3-127x51.png)

This project is licensed under the GNU General Public License 3. The `ChronofoilCaptureFile.proto` definition file is from [Project Chronofoil](https://github.com/ProjectChronofoil/Chronofoil.CaptureFile/blob/main/src/Proto/ChronofoilCaptureFile.proto).
