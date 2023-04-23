use serde::Deserialize as TraitDeserialize;
use serde_derive::{Serialize,Deserialize};
use crate::hexbytes;
#[derive(Debug,Deserialize,Serialize)]
pub struct Block {
    pub id:u32,
    pub jsonrpc:String,
    pub error:Option<String>,
    pub result:BlockResult,
}
#[derive(Debug,Deserialize,Serialize,Clone)]
pub struct BlockResult {
    pub id:String,
    pub job:Job,
    pub extensions:Vec<String>,
    pub status:String,
}
#[derive(Debug,Deserialize,Serialize,Clone)]
pub struct Job {
    #[serde(deserialize_with = "hexbytes::hex_to_varbyte")]
    pub blob:Vec<u8>,
    pub job_id:String,
    #[serde(deserialize_with = "deserialize_target")]
    pub target:u64,
    pub id:String,
    pub timestamp:u64,
    pub height:u64,
    pub algo:String,
    pub variant:String,
    pub seed_hash:String,
    pub motd:String,
}
pub fn deserialize_target<'de, D>(deserializer: D) -> Result<u64, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let (mut val, hexlen) = hexbytes::hex64le_to_int(deserializer)?;
    // unpack compact format
    // XXX: this is what other miners do. It doesn't seem right...
    if hexlen <= 8 {
        val |= val << 0x20;
    }
    Ok(val)
}
