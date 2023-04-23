use serde::{Deserialize as TraitDeserialize , Serialize as TraitSerialize };
use serde_derive::{Serialize,Deserialize};
use crate::hexbytes;

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
    #[serde(serialize_with = "hexbytes::u32_to_hex_padded")]
    pub nonce:u32,
    #[serde(serialize_with = "hexbytes::byte32_to_hex")]
    pub result:[u8;32],
}
