use crate::app::constants::{banano, nano};
use blake2::digest::{Update, VariableOutput};
use blake2::Blake2bVar;
use rand::RngCore;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
type Terminated = Arc<AtomicBool>;

pub fn generate_work(input_hash: &[u8; 32], prefix: &str) -> String {
    let cpus = num_cpus::get();

    /*
    eprintln!(
        "Found {} CPUS. Generating work for hash: {}",
        cpus,
        hex::encode(input_hash)
    );
    */

    let terminated = Arc::new(AtomicBool::new(false));
    let threshold: u64;

    if prefix == "nano_" {
        threshold = u64::from_str_radix(nano::DIFFICULTY_THRESHOLD, 16).unwrap();
    } else if prefix == "ban_" {
        threshold = u64::from_str_radix(banano::DIFFICULTY_THRESHOLD, 16).unwrap();
    } else {
        panic!("Unknown coin threshold");
    }

    let mut threads = vec![];

    for i in 0..cpus {
        let mut input_copy = [0u8; 32];
        input_copy.clone_from_slice(input_hash);
        let threshold_copy = threshold;
        let terminator = terminated.clone();
        let thread_handle = thread::spawn(move || {
            let (success, work) = compute_work(terminator, &input_copy, threshold_copy);
            //eprintln!("Thread {} success: {}", i.clone(), success);
            (success, work)
        });
        threads.push(thread_handle);
    }

    let mut work_res: String = String::from("");
    for thread in threads.into_iter() {
        let (success, work) = thread.join().unwrap();
        if success {
            work_res = work;
        }
    }
    work_res
}

fn compute_work(terminated: Terminated, input_hash: &[u8; 32], threshold: u64) -> (bool, String) {
    let mut work_n_hash = [0u8; 40];
    let mut diff = [0u8; 8];
    work_n_hash[8..].clone_from_slice(input_hash);
    rand::thread_rng().fill_bytes(&mut work_n_hash[..8]);

    loop {
        let idx = (rand::random::<u8>() % 8) as usize;
        let c = work_n_hash[idx];
        work_n_hash[idx] = if c == 0xff { 0 } else { c + 1 };
        new_diff(&work_n_hash, &mut diff);
        if u64::from_le_bytes(diff) > threshold {
            //eprintln!("Found work ({} > {})", u64::from_le_bytes(diff), threshold);
            terminated.store(true, Ordering::Relaxed);
            break;
        }
        if terminated.load(Ordering::Relaxed) {
            return (false, String::from("nope"));
        }
    }

    let work = &work_n_hash[0..8];
    let mut work_vec = Vec::from(work);
    work_vec.reverse();
    let work_hex = hex::encode(work_vec);
    (true, work_hex)
}

fn new_diff(work_and_hash: &[u8; 40], diff: &mut [u8; 8]) {
    let mut hasher = Blake2bVar::new(8).unwrap();
    hasher.update(work_and_hash);
    hasher.finalize_variable(diff).unwrap()
}
