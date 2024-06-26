use bytestring::ByteString;
use serde::{Deserialize, Serialize};
use serde_json::json;
use skynet::HyUuid;

#[derive(Serialize, Deserialize, Debug)]
pub struct Message {
    pub id: HyUuid,
    pub data: DataType,
}

impl Message {
    #[must_use]
    pub fn new(data: DataType) -> Self {
        Self {
            id: HyUuid::new(),
            data,
        }
    }

    #[must_use]
    pub const fn new_rsp(id: &HyUuid, data: DataType) -> Self {
        Self { id: *id, data }
    }

    #[must_use]
    pub fn json(&self) -> String {
        json!(self).to_string()
    }
}

impl From<Message> for String {
    fn from(value: Message) -> Self {
        value.json()
    }
}

impl From<Message> for ByteString {
    fn from(value: Message) -> Self {
        value.json().into()
    }
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "type")]
pub enum DataType {
    Login(Login),
    Update(Update),
    Status(Status),
    ShellConnect(ShellConnect),
    ShellResize(ShellResize),
    ShellInput(ShellInput),
    ShellDisconnect(ShellDisconnect),
    Reconnect,
    Quit,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ShellConnect {
    pub id: HyUuid,
    pub cmd: String,
    pub rows: u16,
    pub cols: u16,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ShellDisconnect {
    pub token: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ShellResize {
    pub token: String,
    pub rows: u16,
    pub cols: u16,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ShellInput {
    pub token: String,
    pub data: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Status {
    pub time: i64,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Update {
    pub data: String,
    pub crc32: u32,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Login {
    pub code: i32,
    pub msg: String,
}
