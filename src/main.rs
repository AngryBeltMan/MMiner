use byteorder::{LE,ByteOrder};
use tokio::sync::mpsc;
use hex::FromHex;
use randomx::memory::VmMemory;
use randomx::vm::new_vm;
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
mod tcp;

fn main() {
    let login = request::Login {
        login:WALLET.to_string(),
        pass: "rustyminer".to_string(),
        agent:"pow#er/0.2.0".to_string(),
    };

    let request = request::Request {
        method:"login".to_string(),
        params:&login,
        id:1,
    };

    let stream = tcp::connect(POOLADDR, request).unwrap();
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
    let mut threads = vec![];
    let (send,reciever) = mpsc::unbounded_channel();
    let send = Arc::new(send);
    let mut client = tcp::Client { stream, reciever };
    let (sender,recv) = mpsc::unbounded_channel();
    let recv = Arc::new(Mutex::new(recv));
    for i in 1..=3 {
        let block = block.result.clone();
        let r = Arc::clone(&recv);
        let s = Arc::clone(&send);
        let miner = std::thread::spawn(move || {
            println!("skip on {i}");
            mine_monero(block,s,r,i,3);
        });
        threads.push(miner);
    }
    let id = block.result.id.unwrap();
    std::thread::spawn(move || {
        let stream = Arc::clone(&send);
        loop {
            keep_alive(Arc::clone(&stream), &id);
            std::thread::sleep(Duration::from_secs(15));
        }
    });
    std::thread::spawn(move || { client.message_listener() });

    std::thread::spawn(move ||  {
        let mut vm_mem_alloc = randomx::memory::VmMemoryAllocator::initial();
        loop {
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
                    Err(_err) => {
                        println!("{buffer}");
                    }
                }
            }
            match serde_json::from_str::<block::JobBlock>(&buffer) {
                Ok(block) => {
                    vm_mem_alloc.reallocate(block.params.seed_hash.clone());
                    println!("new job");
                    if sender.send((block.to_block(),Arc::clone(&vm_mem_alloc.vm_memory))).is_ok() { } else { println!("couldn't send") }
                },
                Err(_err) => { }
            }
        }
    });

    for i in threads { i.join().unwrap(); }
}

fn mine_monero(
    mut block:block::BlockResult,
    sender:Arc<mpsc::UnboundedSender<request::MessageType>>,
    recv:Arc<Mutex<mpsc::UnboundedReceiver<(block::Block,Arc<VmMemory>)>>>,
    start:u32,
    skip:u32
    ) {
    let mut num_target = job_target_value(&block.job.target);
    println!("{num_target}");
    let mut nonce =  start;

    let seed = string_to_u8_array(&block.job.seed_hash);
    let memory = VmMemory::light(&seed);
    let mut vm = new_vm(memory.into());

    while nonce <= 65535 {
        let nonce_hex = nonce_hex(nonce);
        let hash_in = with_nonce(&block.job.blob, &nonce_hex);
        let bytes_in = string_to_u8_array(&hash_in);

        let hash_result = vm.calculate_hash(&bytes_in).to_hex();
        let hash_val = hash_target_value(&hash_result);
        if hash_val <= num_target {
            println!("found share");
            let share = request::Share {
                id:block.job.id.clone(),
                job_id:block.job.job_id.clone(),
                nonce:nonce_hex,
                result:hash_result.to_string()
            };
            sender.send(request::MessageType::Submit(share)).unwrap();
            println!("sent share");
        }
        if let Ok(b) = recv.lock().unwrap().try_recv() {
            vm = new_vm(b.1);
            nonce = start;
            num_target = job_target_value(&b.0.result.job.target);
            block = b.0.result;
        }
        nonce += skip;
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
pub fn keep_alive(stream:Arc<mpsc::UnboundedSender<request::MessageType>>,id:&str) {
    let keep_alive = request::KeepAlive { id:id.to_string() };
    stream.send(request::MessageType::KeepAlive(keep_alive)).unwrap();
}
#[test]
fn randomx_test() {
    use rust_randomx::*;
    let key = b"hello key";
    let blob = b"blobby blob blob";
    let blob2 = b"ifjeojoijlfdslk";
    let memory = VmMemory::light(key);
    let mut vm = new_vm(memory.into());
    let hash_result1 = vm.calculate_hash(blob).to_hex();
    let hash_result3 = vm.calculate_hash(blob2).to_hex();
    let flag = RandomXFlag::default();
    let memory = RandomXCache::new(flag,key).unwrap();
    let rx_dataset = RandomXDataset::new(flag, &memory, 0).unwrap();
    let vm = RandomXVM::new(flag,Some(&memory),Some(&rx_dataset)).unwrap();
    let hash_result2 = bytes_to_hex(&vm.calculate_hash(blob).unwrap());
    let hash_result4 = bytes_to_hex(&vm.calculate_hash(blob).unwrap());
    assert_eq!(hash_result1.to_string(),hash_result2.to_string());
    assert_eq!(hash_result3.to_string(),hash_result4.to_string());
}

