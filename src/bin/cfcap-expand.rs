use binrw::{BinRead, BinWrite, binread, binrw};
use prost::Message;
use serde::Deserialize;
use std::env::current_exe;
use std::path::PathBuf;
use std::{
    fs,
    io::{Cursor, Read},
};

#[binread]
#[br(repr = u16)]
#[derive(Debug)]
pub enum ConnectionType {
    None = 0x0,
    Zone = 0x1,
    Chat = 0x2,
    Lobby = 0x3,
}
#[binread]
#[br(repr = u8)]
#[derive(Debug, PartialEq)]
pub enum CompressionType {
    Uncompressed = 0,
    Oodle = 2,
}

#[binread]
#[derive(Debug, PartialEq)]
pub enum PacketType {
    // Client->Server Packets
    #[br(magic = 0x1u16)]
    InitializeSession,
    #[br(magic = 0x9u16)]
    InitializeEncryption,
    #[br(magic = 0x3u16)]
    Ipc,
    #[br(magic = 0x7u16)]
    KeepAlive,
    #[br(magic = 0xAu16)]
    InitializationEncryptionResponse,
    #[br(magic = 0x8u16)]
    KeepAliveResponse,
    #[br(magic = 0x2u16)]
    ZoneInitialize,
}

impl PacketType {
    fn to_string(&self) -> &'static str {
        match self {
            PacketType::InitializeSession => "InitSession",
            PacketType::InitializeEncryption => "InitEncryption",
            PacketType::Ipc => "Ipc",
            PacketType::KeepAlive => "KeepAlive",
            PacketType::InitializationEncryptionResponse => "InitEncryption",
            PacketType::KeepAliveResponse => "KeepAlive",
            PacketType::ZoneInitialize => "InitZone",
        }
    }
}

#[binread]
#[derive(Debug)]
pub struct PacketHeader {
    pub size: u32,
    pub src_entity: u32,
    pub dst_entity: u32,
    packet_type: PacketType,
    pub padding: u16,
}

#[binread]
#[derive(Debug)]
pub struct PacketSegment {
    pub size: u32,
    pub source_actor: u32,
    pub target_actor: u32,
}

#[binrw]
#[derive(Debug)]
pub struct PacketIPCHeader {
    unk: u16,
    opcode: u16,
    unk1: u16,
    server_id: u16,
    timestamp: u32,
    unk2: u32,
}

#[binread]
#[derive(Debug)]
pub struct Packet {
    header: PacketHeader,
    #[br(if(header.packet_type == PacketType::Ipc))]
    ipc_header: Option<PacketIPCHeader>,
    #[br(count = if header.packet_type == PacketType::Ipc { header.size - 32 } else { header.size - 16 })]
    // FIXME: this is terrible
    data: Vec<u8>,
}
#[binread]
#[derive(Debug)]
pub struct FrameHeader {
    prefix: [u8; 16],
    time_value: u64,
    total_size: u32,
    protocol: ConnectionType,
    count: u16,
    version: u8,
    compression: CompressionType,
    unk: u16,
    decompressed_length: u32,
}

#[binread]
#[derive(Debug)]
pub struct Frame {
    header: FrameHeader,
    #[br(count = header.count)]
    packets: Vec<Packet>,
}

pub mod chronofoil {
    include!(concat!(env!("OUT_DIR"), "/chronofoil.rs"));
}

fn write_packet(
    path: PathBuf,
    data: &[u8],
    index: &mut usize,
    direction: &str,
    opcode_list: &Vec<KnownOpCode>,
) {
    let mut cursor = Cursor::new(data);

    let header = Frame::read_le(&mut cursor).unwrap();
    for (i, packet) in header.packets.iter().enumerate() {
        let mut path = path.clone();

        if let Some(ipc_header) = &packet.ipc_header {
            let matched_opcodes: Vec<&KnownOpCode> = opcode_list
                .iter()
                .filter(|x| x.opcode == ipc_header.opcode as i32)
                .take(1)
                .collect();
            if matched_opcodes.len() == 1 {
                path.push(format!(
                    "{}-{}-{} (to {}) ({})",
                    index,
                    packet.header.packet_type.to_string(),
                    matched_opcodes[0].name,
                    direction,
                    i
                ));

                if matched_opcodes[0].size != packet.header.size - 16 - 16 {
                    println!(
                        "Packet size mismatch at {}! It should be {}, not {}!",
                        path.to_str().unwrap(),
                        packet.header.size - 16 - 16,
                        matched_opcodes[0].size
                    );
                }
            } else {
                // try to guess unknown IPC codes by their size
                for known_ipcode in opcode_list {
                    if known_ipcode.size == packet.header.size - 16 - 16 {
                        println!("{} ({direction}) may be {}, based on the size.", ipc_header.opcode, known_ipcode.name);
                        break;
                    }
                }

                path.push(format!(
                    "{}-{}-{} (to {}) ({})",
                    index,
                    packet.header.packet_type.to_string(),
                    ipc_header.opcode,
                    direction,
                    i
                ));
            }
        } else {
            path.push(format!(
                "{}-{} (to {}) ({})",
                index,
                packet.header.packet_type.to_string(),
                direction,
                i
            ));
        }

        fs::create_dir_all(&path).unwrap();

        path.push("data.bin");

        fs::write(path.to_str().unwrap(), &packet.data).unwrap();

        if let Some(ipc_header) = &packet.ipc_header {
            let mut buf = Cursor::new(Vec::new());
            ipc_header.write_le(&mut buf).unwrap();

            path.pop();
            path.push("ipc_header.bin");
            fs::write(path.to_str().unwrap(), buf.into_inner()).unwrap();
        }

        path.pop();
        path.push("source_actor.bin");
        fs::write(
            path.to_str().unwrap(),
            packet.header.src_entity.to_le_bytes(),
        )
        .unwrap();

        path.pop();
        path.push("target_actor.bin");
        fs::write(
            path.to_str().unwrap(),
            packet.header.dst_entity.to_le_bytes(),
        )
        .unwrap();

        *index += 1;
    }
}

fn read_data_entry<B: prost::bytes::Buf>(
    capture_id: &str,
    buffer: B,
    known_opcodes: &KnownOpCodes,
) {
    let mut buffer = buffer;
    let mut i = 0;
    while buffer.has_remaining() {
        let capture_info = chronofoil::CaptureFrame::decode_length_delimited(&mut buffer).unwrap();

        let header = capture_info.header.unwrap();

        // Then save it
        let protocol_dir = match header.protocol() {
            chronofoil::Protocol::None => "none",
            chronofoil::Protocol::Zone => "zone",
            chronofoil::Protocol::Chat => "chat",
            chronofoil::Protocol::Lobby => "lobby",
        };

        let direction_dir = match header.direction() {
            chronofoil::Direction::None => "none",
            chronofoil::Direction::Rx => "client",
            chronofoil::Direction::Tx => "server",
        };

        let default_vec: Vec<KnownOpCode> = Vec::default();
        let opcode_list = match header.direction() {
            chronofoil::Direction::None => &default_vec,
            chronofoil::Direction::Rx => match header.protocol() {
                chronofoil::Protocol::None => &default_vec,
                chronofoil::Protocol::Zone => &known_opcodes.server_zone_ipc_type,
                chronofoil::Protocol::Chat => &known_opcodes.server_chat_ipc_type,
                chronofoil::Protocol::Lobby => &known_opcodes.server_lobby_ipc_type,
            },
            chronofoil::Direction::Tx => match header.protocol() {
                chronofoil::Protocol::None => &default_vec,
                chronofoil::Protocol::Zone => &known_opcodes.client_zone_ipc_type,
                chronofoil::Protocol::Chat => &known_opcodes.client_chat_ipc_type,
                chronofoil::Protocol::Lobby => &known_opcodes.client_lobby_ipc_type,
            },
        };

        let mut path = current_exe().unwrap().parent().unwrap().to_path_buf();
        path.push(capture_id);
        path.push(protocol_dir);

        fs::create_dir_all(&path).unwrap();

        write_packet(
            path,
            &capture_info.frame.unwrap(),
            &mut i,
            direction_dir,
            opcode_list,
        );
    }
}

#[derive(Deserialize, Debug)]
struct KnownOpCode {
    name: String,
    opcode: i32,
    size: u32,
}

#[derive(Deserialize, Default, Debug)]
struct KnownOpCodes {
    #[serde(default, rename = "ServerZoneIpcType")]
    server_zone_ipc_type: Vec<KnownOpCode>,
    #[serde(default, rename = "ClientZoneIpcType")]
    client_zone_ipc_type: Vec<KnownOpCode>,

    #[serde(default, rename = "ServerLobbyIpcType")]
    server_lobby_ipc_type: Vec<KnownOpCode>,
    #[serde(default, rename = "ClientLobbyIpcType")]
    client_lobby_ipc_type: Vec<KnownOpCode>,

    #[serde(default, rename = "ServerChatIpcType")]
    server_chat_ipc_type: Vec<KnownOpCode>,
    #[serde(default, rename = "ClientChatIpcType")]
    client_chat_ipc_type: Vec<KnownOpCode>,
}

fn main() {
    let args: Vec<String> = std::env::args().collect();

    let mut known_opcodes = KnownOpCodes::default();
    if let Ok(ok) = fs::exists("opcodes.json") {
        if ok {
            known_opcodes =
                serde_json::from_str(&fs::read_to_string("opcodes.json").unwrap()).unwrap();
        }
    }

    let file = fs::File::open(&args[1]).unwrap();
    let mut archive = zip::ZipArchive::new(file).unwrap();

    let capture_id;
    // read capture file header
    {
        let mut version_info_entry = archive.by_name("VersionInfo").unwrap();

        let mut buf = Vec::new();
        version_info_entry.read_to_end(&mut buf).unwrap();
        let _ = chronofoil::VersionInfo::decode_length_delimited(buf.as_slice()).unwrap();
    }

    // read capture info
    {
        let mut version_info_entry = archive.by_name("CaptureInfo").unwrap();

        let mut buf = Vec::new();
        version_info_entry.read_to_end(&mut buf).unwrap();
        let capture_info =
            chronofoil::CaptureInfo::decode_length_delimited(buf.as_slice()).unwrap();

        capture_id = capture_info.capture_id.unwrap();
    }

    // read data
    {
        let mut version_info_entry = archive.by_name("Data").unwrap();

        let mut buf = Vec::new();
        version_info_entry.read_to_end(&mut buf).unwrap();

        let mut decoder = zstd::stream::Decoder::new(&*buf).unwrap();

        let mut buf = Vec::new();
        decoder.read_to_end(&mut buf).unwrap();

        read_data_entry(&capture_id, &*buf, &known_opcodes);
    }

    println!("Successfully extracted capture {capture_id}!")
}
