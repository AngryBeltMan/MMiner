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
const THREADS:usize = 1;
const LIGHTMODE:bool = false;
mod block;
mod request;
mod hexbytes;
mod randomx;
mod tcp;
mod hashing;

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
    for i in 1..=THREADS {
        let block = block.result.clone();
        let r = Arc::clone(&recv);
        let s = Arc::clone(&send);
        let miner = std::thread::spawn(move || {
            println!("skip on {i}");
            mine_monero(block,s,r,i as u32,THREADS as u32);
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
                    if LIGHTMODE {
                        vm_mem_alloc.reallocate_light(block.params.seed_hash.clone());
                    } else {
                        vm_mem_alloc.reallocate(block.params.seed_hash.clone());
                    }
                    println!("new job");
                    for _ in 0..THREADS {
                        if sender.send((block.clone().to_block(),Arc::clone(&vm_mem_alloc.vm_memory))).is_ok() { } else { println!("couldn't send") }
                    }
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
    let mut num_target = hexbytes::job_target_value(&block.job.target);
    println!("{num_target}");
    let mut nonce =  start;

    let seed = hexbytes::string_to_u8_array(&block.job.seed_hash);
    let memory = VmMemory::light(&seed);
    let mut vm = new_vm(memory.into());

    loop {
        let nonce_hex = hexbytes::nonce_hex(nonce);
        let hash_in = hexbytes::with_nonce(&block.job.blob, &nonce_hex);
        let bytes_in = hexbytes::string_to_u8_array(&hash_in);

        let hash_result = vm.calculate_hash(&bytes_in).to_hex();
        let hash_val = hexbytes::hash_target_value(&hash_result);
        println!("{nonce}");
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
            num_target = hexbytes::job_target_value(&b.0.result.job.target);
            block = b.0.result;
        }
        nonce += skip;
    }
}
pub fn keep_alive(stream:Arc<mpsc::UnboundedSender<request::MessageType>>,id:&str) {
    let keep_alive = request::KeepAlive { id:id.to_string() };
    stream.send(request::MessageType::KeepAlive(keep_alive)).unwrap();
}
#[test]
fn test() {
    let seed = "epiccoolseed";
    let key = hexbytes::string_to_u8_array(seed);
    let mut vm_mem_alloc = randomx::memory::VmMemoryAllocator::initial();
    let mem = randomx::memory::VmMemory::light(&key);
    let memfull = randomx::memory::VmMemory::full(&key);
    vm_mem_alloc.reallocate(seed.to_string());
    let mut vm_light = new_vm(mem.into());
    let mut vm_full = new_vm(memfull.into());
    let mut vm_alloc = new_vm(vm_mem_alloc.vm_memory.into());
    for i in 0..=2 {
        let hash = format!("seed{i}");
        let light_hash = vm_light.calculate_hash(hash.as_bytes());
        let full_hash = vm_full.calculate_hash(hash.as_bytes());
        let alloc_hash = vm_alloc.calculate_hash(hash.as_bytes());
        assert_eq!(light_hash,full_hash);
        assert_eq!(light_hash,alloc_hash);
        assert_eq!(full_hash,alloc_hash);
    }
}
