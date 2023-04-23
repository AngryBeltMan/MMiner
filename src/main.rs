use byteorder::{LE,ByteOrder};
use crate::randomx::{*, vm::new_vm};
use hex::FromHex;
// use randomx4r::{RandomxCache,RandomxError,RandomxFlags,RandomxVm, RandomxDataset};
use rust_randomx::*;
use serde::__private::de::Content;
use std::{
    sync::{Arc,atomic::{Ordering, AtomicUsize},Mutex},
    net::TcpStream,
    thread::sleep,
    time::Duration,
    io::{self,Write,Cursor, Read, BufWriter,BufRead, BufReader},
    cmp::min, fmt::format, ops::{Deref, Index}
};
const POOLADDR:&str = "pool.hashvault.pro:80";
const WALLET:&str =  "49sygkbkGRYgBRywwhovJp75gUFPwqqepfLyCuaVv4VbAVRSdtRd1ggMbZdzVRnQF3EVhcu2Ekz9n3YepBtFSJbW17U7h1z";
mod block;
mod request;
mod hexbytes;
mod randomx;
fn main() {
    let mut stream = TcpStream::connect(POOLADDR).unwrap();
    stream.set_nodelay(true).unwrap();
    let login = request::Login {
        login:WALLET.to_string(),
        pass: "jifkd94843".to_string(),
        agent:"pow#er/0.2.0".to_string(),
    };
    let request = request::Request {
        method:"login".to_string(),
        params:&login,
        id:1,
    };
    serde_json::to_writer(&mut stream, &request).unwrap();
    writeln!(&mut stream).unwrap();
    stream.flush().unwrap();
    let mut stream_r = BufReader::with_capacity(1500, stream.try_clone().unwrap());
    let mut buffer = String::new();
    loop {
        match stream_r.read_line(&mut buffer) {
            Ok(_o) => {
                if buffer.is_empty() {
                    println!("disconnected");
                    continue;
                }
                break;
            },
            Err(err) if (err.kind() == io::ErrorKind::WouldBlock) | (err.kind() == io::ErrorKind::TimedOut) => {
                println!("no data found");
                continue;
            },
            Err(err) => {
                panic!("{err:?}");
            }
        }
    }
    println!("{buffer}");
    let block = serde_json::from_str::<block::Block>(&buffer).unwrap();
    mine_monero(block.result.clone(),&mut stream);
    buffer.clear();
    loop {
        match stream_r.read_line(&mut buffer) {
            Ok(_o) => {
                if buffer.is_empty() {
                    println!("disconnected");
                    continue;
                }
                break;
            },
            Err(err) if (err.kind() == io::ErrorKind::WouldBlock) | (err.kind() == io::ErrorKind::TimedOut) => {
                println!("no data found");
                continue;
            },
            Err(err) => {
                panic!("{err:?}");
            }
        }
    }
    println!("{buffer:?}");
}

fn mine_monero(
    block:block::BlockResult,
    mut socket:&mut TcpStream,
    ) {
    let difficulty = block.job.target;

    let seed = block.job.seed_hash;
    let seed_hash = <[u8;32]>::from_hex(&seed).unwrap();

    let mut nonce:u32 =  0;

    let flags = RandomXFlag::FLAG_DEFAULT;
    let cache = RandomXCache::new(flags,&seed_hash).unwrap();
    let dataset = RandomXDataset::new(flags, &cache, 0).unwrap();

    let vm = RandomXVM::new(flags,Some(&cache),Some(&dataset)).unwrap();
    // let memory = crate::randomx::memory::VmMemory::light(bytes_to_hex(seed.as_bytes()).as_bytes());
    // let mut vm = new_vm(memory.into());
    let target = block.job.target;
    let mut blob = block.job.blob;
    let start = u32::from(blob[42]) << 24;
    let mut hash_res = [0;32];
    loop {
        pack_nonce(&mut blob, &nonce.to_le_bytes());
        hash_res = vm.calculate_hash(&mut blob).unwrap();
        println!("new hash");
        if LE::read_u64(&hash_res[24..]) <= target {
            println!("found a share at nonce: {nonce}");
            let share = request::Share {
                id:block.job.id,
                job_id:block.job.job_id,
                nonce,
                result:hash_res
            };
            let request = request::Request {
                id:1,
                method:"submit".to_string(),
                params:&share
            };
            serde_json::to_writer(&mut socket, &request).unwrap();
            writeln!(&mut socket).unwrap();
            socket.flush().unwrap();
            break;
        }
        // let nonce_hex = format!("{:08x}",nonce);
        // let input = with_nonce(&block.job.blob, &nonce_hex);
        // let input = hex2(&input);
        //
        // // let hash = &vm.calculate_hash(&input).to_hex() as &str;
        // let hash = bytes_to_hex(&vm.calculate_hash(&input).unwrap());
        //
        // let hash_value = hex2_u64_le(&hash[48..]);
        //
        // if  hash_value <=  difficulty {
        // }
        // nonce += 1;
    }
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
pub fn with_nonce(blob: &str, nonce: &str) -> String {
    let a = &blob[..78];
    let b = &blob[86..];
    return format!("{}{}{}", a, nonce, b);
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
