use std::vec;

use serde::Deserialize as TraitDeserialize;
use serde_derive::{Serialize,Deserialize};
use crate::hexbytes;
#[derive(Debug,Deserialize,Serialize)]
pub struct Block {
    pub id:Option<u32>,
    pub jsonrpc:String,
    pub error:Option<String>,
    pub result:BlockResult,
}
#[derive(Debug,Deserialize,Serialize,Clone)]
pub struct JobBlock {
    pub jsonrpc:String,
    pub error:Option<String>,
    pub params:JobBlockResult,
}

#[derive(Debug,Deserialize,Serialize,Clone)]
pub struct JobBlockResult {
    pub blob:String,
    pub job_id:String,
    pub target:String,
    pub id:String,
    pub timestamp:u64,
    pub height:u32,
    pub algo:String,
    pub variant:String,
    pub seed_hash:String
}
impl JobBlock {
    pub fn to_block(self) -> Block {
        Block { id: None, jsonrpc: self.jsonrpc, error: self.error, result: self.params.to_block_res() }
    }
}
impl JobBlockResult {
    pub fn to_block_res(self) -> BlockResult {
        let job = Job {
            blob:self.blob,
            job_id:self.job_id,
            target:self.target,
            id:self.id,
            timestamp:self.timestamp,
            algo:self.algo,
            height:self.height as u64,
            motd:String::new(),
            seed_hash:self.seed_hash,
            variant:self.variant
        };
        BlockResult { id:None , job, extensions: vec![], status: String::new() }
    }
}
#[derive(Debug,Deserialize,Serialize,Clone)]
pub struct BlockResult {
    pub id:Option<String>,
    pub job:Job,
    pub extensions:Vec<String>,
    pub status:String,
}
#[derive(Debug,Deserialize,Serialize,Clone)]
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
