use tokio::sync::mpsc;
use crate::randomx::memory::VmMemory;
use crate::randomx::vm::new_vm;
use crate::{block,request,hexbytes};
use std::sync::{Arc,Mutex};

pub fn mine_monero(
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
        } else {
            nonce += skip;
        }
    }
}
