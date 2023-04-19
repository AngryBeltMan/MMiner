use serde_derive::{Serialize,Deserialize};
#[derive(Debug,Deserialize,Serialize)]
pub struct Block {
    pub id:u32,
    pub jsonrpc:String,
    pub error:Option<String>,
    pub result:BlockResult,
}
#[derive(Debug,Deserialize,Serialize)]
pub struct BlockResult {
    pub id:String,
    pub job:Job,
    pub extensions:Vec<String>,
    pub status:String,
}
#[derive(Debug,Deserialize,Serialize)]
pub struct Job {
    pub blob:String,
    pub job_id:String,
    pub target:String,
    pub id:String,
    pub timestamp:u64,
    pub height:u64,
    pub algo:String,
    pub variant:String,
    pub seed_hash:String,
    pub motd:String,
}
