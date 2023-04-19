use byte_order::NumberReader;
use randomx4r::{RandomxCache,RandomxError,RandomxFlags,RandomxVm};
use std::{
    sync::{Arc,atomic::{Ordering, AtomicUsize},Mutex},
    net::TcpStream,
    thread::sleep,
    time::Duration,
    io::{self,Write,Cursor, Read, BufWriter,BufRead, BufReader},
    cmp::min, fmt::format
};
use rust_randomx::*;
use hex::FromHex;
const POOLADDR:&str = "pool.hashvault.pro:80";
const WALLET:&str =  "49sygkbkGRYgBRywwhovJp75gUFPwqqepfLyCuaVv4VbAVRSdtRd1ggMbZdzVRnQF3EVhcu2Ekz9n3YepBtFSJbW17U7h1z";
mod block;
mod request;
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
    let mut id = 0;
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
    mine_monero(block.result,&mut stream,&mut id);
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
}

fn mine_monero(
    block:block::BlockResult,
    mut socket:&mut TcpStream,
    id:&mut u32
    ) {
    let mut nonce = 0;
    let seed = block.job.seed_hash;
    let seed_hash =  <[u8;32]>::from_hex(&seed).unwrap();
    println!("{}",seed_hash.len());
    // let context = Context::new(b"hello", false);
    // let mut hasher = Hasher::new(Arc::new(context));
    let blob = block.job.blob;
    let difficulty = target_to_u32(&block.job.target);
    let flags = RandomxFlags::default();
    let cache = RandomxCache::new(flags,&seed_hash).unwrap();
    let vm = RandomxVm::new(flags,&cache).unwrap();
    loop {
        let nonce_hex = format!("{:08x}",nonce);
        let input = input_generator(&blob, &nonce_hex);
        let hash = vm.hash(input.as_bytes());
        let c = Cursor::new(hash);
        let mut reader = NumberReader::with_order(byte_order::ByteOrder::LE, c);
        let out = reader.read_u32().unwrap();
        if out <  difficulty {
            println!("found a share");
            let share = request::Share {
                id:block.job.id,
                job_id:block.job.job_id,
                nonce,
                result:hash
            };
            let request = request::Request {
                id:*id,
                method:"submit".to_string(),
                params:&share
            };
            serde_json::to_writer(&mut socket, &request).unwrap();
            writeln!(&mut socket).unwrap();
            socket.flush().unwrap();
            *id +=1;
            break;
        }
        nonce += 1;
    }
}
pub fn input_generator(blob:&str,nonce:&str) -> String {
    let part1 = &blob[..76];
    let part2 = &blob[86..];
    format!("{}{}{}",part1,nonce,part2)
}
pub fn target_to_u32(hex:&str) -> u32 {
    let c = Cursor::new(hex);
    let mut reader = NumberReader::with_order(byte_order::ByteOrder::LE, c);
    let diff = reader.read_u32().unwrap();
    println!("difficulty: {diff}");
    diff
}

