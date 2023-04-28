use arrayvec::ArrayVec;
use hex::FromHex;
use byteorder::{LE,ByteOrder};
use serde::{self, Deserializer, Serializer};
use std::str;

fn nibble_to_hex(x: u8) -> Result<u8, ()> {
    match x {
        0x0..=0x9 => Ok(x + b'0'),
        0xa..=0xf => Ok(x - 0xa + b'a'),
        _ => Err(()),
    }
}

fn hex_to_nibble(x: u8) -> Result<u8, ()> {
    match x {
        b'0'..=b'9' => Ok(x - b'0'),
        b'a'..=b'f' => Ok(x - b'a' + 0xa),
        _ => Err(()),
    }
}

pub fn buffer_to_hex_string(buffer: &[u8]) -> String {
    let mut buf = Vec::with_capacity(2 * buffer.len());
    for c in buffer.iter() {
        buf.push(nibble_to_hex((c >> 4) & 0xf as u8).unwrap());
        buf.push(nibble_to_hex(c & 0xf as u8).unwrap());
    }
    String::from_utf8(buf).unwrap()
}

pub fn u32_to_hex_string_bytes_padded(n: u32) -> ArrayVec<u8,8> {
    let mut buf = ArrayVec::new();
    for i in 0..4 {
        let x0 = (n >> (8 * i + 4)) & 0xfu32;
        let x1 = (n >> (8 * i)) & 0xfu32;
        buf.push(nibble_to_hex(x0 as u8).unwrap());
        buf.push(nibble_to_hex(x1 as u8).unwrap());
    }
    buf
}



use serde::de::{self, Visitor};
use std::fmt;
struct Hex64leStrVisitor {}
impl<'de> Visitor<'de> for Hex64leStrVisitor {
    type Value = (u64, usize);

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("hex string")
    }

    fn visit_str<E>(self, hex_in: &str) -> Result<(u64, usize), E>
    where
        E: de::Error,
    {
        use serde::de::Error;
        let hex_in = hex_in.as_bytes();
        let hexlen = hex_in.len();
        if hexlen > 16 {
            return Err(Error::custom("too many input bytes for hex64le"));
        }
        let mut out = 0u64;
        for (i, xs) in hex_in.chunks_exact(2).enumerate() {
            let nib0 = u64::from(
                hex_to_nibble(xs[0]).map_err(|_| Error::custom("non-hex char in input"))?,
            );
            let nib1 = u64::from(
                hex_to_nibble(xs[1]).map_err(|_| Error::custom("non-hex char in input"))?,
            );
            out |= nib0 << (i * 8 + 4);
            out |= nib1 << (i * 8);
        }
        Ok((out, hexlen))
    }
}

/// Deserialize a value of up to 64 bits, reporting number of hex bytes it contained
pub fn hex64le_to_int<'de, D>(deserializer: D) -> Result<(u64, usize), D::Error>
where
    D: Deserializer<'de>,
{
    deserializer.deserialize_str(Hex64leStrVisitor {})
}
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

