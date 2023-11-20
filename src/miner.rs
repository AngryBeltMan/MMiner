use crate::randomx::memory::VmMemory;
use crate::randomx::vm::new_vm;
use crate::{block, hexbytes, request, LIGHTMODE};
use std::sync::{Arc, Mutex};
use stopwatch::Stopwatch;
use tokio::sync::mpsc;

pub fn mine_rx(
    mut block: block::BlockResult,
    sender: Arc<mpsc::UnboundedSender<request::MessageType>>,
    recv: Arc<Mutex<mpsc::UnboundedReceiver<(block::Block, Arc<VmMemory>)>>>,
    start: u32,
    skip: u32,
) {
    let mut num_target: u64 = hexbytes::job_target_value(&block.job.target);
    println!("Difficulty: {}", num_target);
    let mut nonce: u32 = start;

    let mut last: i64 = 0;
    let cycles_const: u8 = 10;
    let mut cycles: u8 = 0;
    let mut hashes: f64 = 0.0;

    let seed: Vec<u8> = hexbytes::string_to_u8_array(&block.job.seed_hash);
    let memory: VmMemory = if LIGHTMODE {
        VmMemory::full(&seed)
    } else {
        VmMemory::light(&seed)
    };
    let mut vm: crate::randomx::vm::Vm = new_vm(memory.into());
    let mut sw: Stopwatch = Stopwatch::start_new();

    loop {
        let nonce_hex: String = hexbytes::nonce_hex(nonce);
        let hash_in: String = hexbytes::with_nonce(&block.job.blob, &nonce_hex);
        let bytes_in: Vec<u8> = hexbytes::string_to_u8_array(&hash_in);

        let hash_result: arrayvec::ArrayString<128> = vm.calculate_hash(&bytes_in).to_hex();
        let hash_val: u64 = hexbytes::hash_target_value(&hash_result);
        //println!("Nonce: {nonce} from {start} thread");
        if last > 0 && cycles == cycles_const {
            println!(
                "{} H/s from {start} thread",
                hashes / ((sw.elapsed_ms() - last) as f64 * 0.001)
            ); //from XMRig code
            sw.restart();
            cycles = 0;
            hashes = 0.0;
        }

        last = sw.elapsed_ms();
        cycles += 1;
        hashes += 1.0;

        if hash_val <= num_target {
            //<
            println!("found share");
            let share: request::Share = request::Share {
                id: block.job.id.clone(),
                job_id: block.job.job_id.clone(),
                nonce: nonce_hex,
                result: hash_result.to_string(),
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
