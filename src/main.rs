use tokio::sync::mpsc;
use randomx::memory::VmMemory;
use randomx::vm::new_vm;
use std::{
    sync::{Arc,Mutex},
    time::Duration,
    io::{self,BufRead, BufReader},
};

const POOLADDR:&str = "pool.hashvault.pro:80";
const WALLET:&str =  "49sygkbkGRYgBRywwhovJp75gUFPwqqepfLyCuaVv4VbAVRSdtRd1ggMbZdzVRnQF3EVhcu2Ekz9n3YepBtFSJbW17U7h1z";
const THREADS:usize = 4;
const LIGHTMODE:bool = false;
mod block;
mod request;
mod hexbytes;
mod randomx;
mod tcp;
mod hashing;
mod miner;

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
            miner::mine_monero(block,s,r,i as u32,THREADS as u32);
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
        request::job_listener(stream_r, sender);
    });

    for i in threads { i.join().unwrap(); }
}

pub fn keep_alive(stream:Arc<mpsc::UnboundedSender<request::MessageType>>,id:&str) {
    let keep_alive = request::KeepAlive { id:id.to_string() };
    stream.send(request::MessageType::KeepAlive(keep_alive)).unwrap();
}
