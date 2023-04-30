use hex::FromHex;
use byteorder::{LE,ByteOrder};
use std::str;
/// Deserialize a value of up to 64 bits, reporting number of hex bytes it contained
pub fn pack_nonce(blob:&mut [u8],nonce:&[u8;4]) {
    blob[39] = nonce[0];
    blob[40] = nonce[1];
    blob[41] = nonce[2];
    blob[42] = nonce[3];
}
pub fn target_to_u64(hex:&str) -> u64 {
    let diff = LE::read_u32(hex.as_bytes());
    u64::MAX / ((u32::MAX as u64) / diff as u64)
}

pub fn hex2(hex: &str) -> Vec<u8> {
    let mut bytes = Vec::new();
    for i in 0..(hex.len() / 2) {
        let res = u8::from_str_radix(&hex[2 * i..2 * i + 2], 16);
        match res {
            Ok(v) => bytes.push(v),
            Err(e) => {
                println!("Problem with hex: {}", e);
                return bytes;
            }
        };
    }
    bytes
}
pub fn nonce_hex(nonce: u32) -> String {
    format!("{:08x}", nonce)
}

pub fn with_nonce(blob: &str, nonce: &str) -> String {
    let (a, _) = blob.split_at(78);
    let (_, b) = blob.split_at(86);
    return format!("{}{}{}", a, nonce, b);
}


pub fn job_target_value(hex_str: &str) -> u64 {
    let t = hex2_u32_le(hex_str);
    u64::max_value() / (u64::from(u32::max_value()) / u64::from(t))
}
pub fn hex2_u32_le(hex: &str) -> u32 {
    let mut result: u32 = 0;
    for k in (0..8).step_by(2) {
        let p = u32::from_str_radix(&hex[(8 - k - 2)..(8 - k)], 16).unwrap();
        result <<= 8;
        result |= p;
    }
    result
}

pub fn hash_target_value(hex_str: &str) -> u64 {
    hex2_u64_le(&hex_str[48..])
}
pub fn bytes_to_hex(bytes:&[u8]) -> String {
    let mut s = String::new();
    let table = b"0123456789abcdef";
    for &b in bytes {
        s.push(table[(b >> 4) as usize] as char);
        s.push(table[(b & 0xf) as usize] as char);
    }
    s
}
pub fn hex2_u64_le(hex: &str) -> u64 {
    let mut result: u64 = 0;
    for k in (0..hex.len()).step_by(2) {
        let p = u64::from_str_radix(&hex[(hex.len() - k - 2)..(hex.len() - k)], 16).unwrap();
        result <<= 8;
        result |= p;
    }
    result
}
pub fn unhexlify(hexstr: &str) -> Result<[u8; 32], hex::FromHexError> {
    <[u8; 32]>::from_hex(hexstr)
}
pub fn string_to_u8_array(hex: &str) -> Vec<u8> {
    let mut bytes = Vec::new();
    for i in 0..(hex.len() / 2) {
        let res = u8::from_str_radix(&hex[2 * i..2 * i + 2], 16);
        match res {
            Ok(v) => bytes.push(v),
            Err(e) => {
                println!("Problem with hex: {}", e);
                return bytes;
            }
        };
    }
    bytes
}

