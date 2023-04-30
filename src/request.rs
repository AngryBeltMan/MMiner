use crate::{
    block,
    randomx::{self, memory::VmMemory},
    LIGHTMODE, THREADS,
};
use serde::{Deserialize as TraitDeserialize, Serialize as TraitSerialize};
use serde_derive::{Deserialize, Serialize};
use std::{
    io::{self, BufRead, BufReader},
    net::TcpStream,
    sync::Arc,
};

#[derive(Debug, Deserialize, Serialize)]
pub struct Login {
    pub login: String,
    pub pass: String,
    pub agent: String,
}

#[derive(Debug, Serialize)]
pub struct Request<'a, T>
where
    T: TraitDeserialize<'a> + TraitSerialize + std::fmt::Debug,
{
    pub id: u32,
    pub method: String,
    pub params: &'a T,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Share {
    pub id: String,
    pub job_id: String,
    pub nonce: String,
    pub result: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct KeepAlive {
    pub id: String,
}
#[derive(Debug)]
pub enum MessageType {
    Submit(Share),
    KeepAlive(KeepAlive),
}
pub fn job_listener(
    mut stream_r: BufReader<TcpStream>,
    sender: tokio::sync::mpsc::UnboundedSender<(block::Block, Arc<VmMemory>)>,
) {
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
                }
                Err(err)
                    if (err.kind() == io::ErrorKind::WouldBlock)
                        | (err.kind() == io::ErrorKind::TimedOut) =>
                {
                    println!("no data found");
                    continue;
                }
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
                    if sender
                        .send((
                            block.clone().to_block(),
                            Arc::clone(&vm_mem_alloc.vm_memory),
                        ))
                        .is_ok()
                    {
                    } else {
                        println!("couldn't send")
                    }
                }
            }
            Err(_err) => {}
        }
    }
}
