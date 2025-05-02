use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug)]
pub struct KnownOpCode {
    pub name: String,
    pub opcode: i32,
    pub size: u32,
    #[serde(skip)]
    pub dirty: bool,
}

#[derive(Deserialize, Serialize, Default, Debug)]
pub struct KnownOpCodes {
    #[serde(default, rename = "ServerZoneIpcType")]
    pub server_zone_ipc_type: Vec<KnownOpCode>,
    #[serde(default, rename = "ClientZoneIpcType")]
    pub client_zone_ipc_type: Vec<KnownOpCode>,

    #[serde(default, rename = "ServerLobbyIpcType")]
    pub server_lobby_ipc_type: Vec<KnownOpCode>,
    #[serde(default, rename = "ClientLobbyIpcType")]
    pub client_lobby_ipc_type: Vec<KnownOpCode>,

    #[serde(default, rename = "ServerChatIpcType")]
    pub server_chat_ipc_type: Vec<KnownOpCode>,
    #[serde(default, rename = "ClientChatIpcType")]
    pub client_chat_ipc_type: Vec<KnownOpCode>,
}
