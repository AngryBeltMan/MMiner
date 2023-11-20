use std::{
    io::{self, BufRead, BufReader},
    sync::{Arc, Mutex},
    time::Duration,
};
use tokio::sync::mpsc;

const POOLADDR: &str = "pool.hashvault.pro:80"; //POOL ADDRESS
const WALLET: &str = "49sygkbkGRYgBRywwhovJp75gUFPwqqepfLyCuaVv4VbAVRSdtRd1ggMbZdzVRnQF3EVhcu2Ekz9n3YepBtFSJbW17U7h1z"; //WALLET ADDRESS
const THREADS: u16 = 4; //Number of threads of your CPU
const LIGHTMODE: bool = false;

mod block;
mod hashing;
mod hexbytes;
mod miner;
mod randomx;
mod request;
mod tcp;

fn main() {
    avx2() //unsafe {avx2()}
}

pub fn keep_alive(stream: Arc<mpsc::UnboundedSender<request::MessageType>>, id: &str) {
    let keep_alive: request::KeepAlive = request::KeepAlive { id: id.to_string() };
    stream
        .send(request::MessageType::KeepAlive(keep_alive))
        .unwrap();
}

//#[target_feature(enable = "avx2")]
//AVX2 is off
fn avx2() {
    let login: request::Login = request::Login {
        login: WALLET.to_string(),
        pass: "nobaki".to_string(),
        agent: "pow#er/0.2.0".to_string(),
    };

    let request: request::Request<'_, request::Login> = request::Request {
        method: "login".to_string(),
        params: &login,
        id: 1,
    };

    let stream: std::net::TcpStream = tcp::connect(POOLADDR, request.clone()).unwrap();
    let mut stream_r: BufReader<std::net::TcpStream> =
        BufReader::with_capacity(1500, stream.try_clone().unwrap());
    let mut buffer: String = String::new();
    loop {
        match stream_r.read_line(&mut buffer) {
            Ok(_o) => {
                if buffer.is_empty() {
                    println!("disconnected");
                    continue;
                }
                break;
            }
            Err(err)
                if (err.kind() == io::ErrorKind::WouldBlock)
                    | (err.kind() == io::ErrorKind::TimedOut) =>
            {
                println!("no data found");
                continue;
            }
            Err(err) => {
                panic!("{err:?}");
            }
        }
    }
    println!("{buffer}");
    let block: block::Block = serde_json::from_str::<block::Block>(&buffer).unwrap();
    let mut threads: Vec<std::thread::JoinHandle<()>> = vec![];
    let (send, reciever) = mpsc::unbounded_channel();
    let send: Arc<mpsc::UnboundedSender<request::MessageType>> = Arc::new(send);
    let mut client: tcp::Client = tcp::Client { stream, reciever };
    let (sender, recv) = mpsc::unbounded_channel();
    let recv: Arc<Mutex<mpsc::UnboundedReceiver<(block::Block, Arc<randomx::memory::VmMemory>)>>> =
        Arc::new(Mutex::new(recv));
    for i in 1..=THREADS {
        let block: block::BlockResult = block.result.clone();
        let r: Arc<Mutex<mpsc::UnboundedReceiver<(block::Block, Arc<randomx::memory::VmMemory>)>>> =
            Arc::clone(&recv);
        let s: Arc<mpsc::UnboundedSender<request::MessageType>> = Arc::clone(&send);
        let miner: std::thread::JoinHandle<()> = std::thread::spawn(move || {
            println!("skip on {i} thread");
            miner::mine_rx(block, s, r, i as u32, THREADS as u32);
        });
        threads.push(miner);
    }
    let id: String = block.result.id.unwrap();
    std::thread::spawn(move || {
        let stream: Arc<mpsc::UnboundedSender<request::MessageType>> = Arc::clone(&send);
        loop {
            keep_alive(Arc::clone(&stream), &id);
            std::thread::sleep(Duration::from_secs(15));
        }
    });
    std::thread::spawn(move || client.message_listener());

    std::thread::spawn(move || {
        request::job_listener(stream_r, sender);
    });

    for i in threads {
        i.join().unwrap();
    }
}
