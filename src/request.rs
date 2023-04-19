use serde::{Deserialize as TraitDeserialize , Serialize as TraitSerialize };
use serde_derive::{Serialize,Deserialize};

#[derive(Debug,Deserialize,Serialize)]
pub struct Login {
    pub login:String,
    pub pass:String,
    pub agent:String,
}
#[derive(Debug,Serialize)]
pub struct Request<'a,T>
where T:TraitDeserialize<'a> + TraitSerialize + std::fmt::Debug
{
    pub id:u32,
    pub method:String,
    pub params:&'a T
}

#[derive(Debug,Deserialize,Serialize)]
pub struct Share {
    pub id:String,
    pub job_id:String,
    pub nonce:u32,
    pub result:[u8;32],
}
