use blake2::digest::{Update, VariableOutput};
use blake2::Blake2bVar;
use rand::RngCore;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;

type Terminated = Arc<AtomicBool>;

pub fn generate_work(input_hash: &[u8; 32], coin: &str) -> String {
    let cpus = num_cpus::get();
    let terminated = Arc::new(AtomicBool::new(false));
    let threshold: u64;

    if coin == "nano" {
        threshold = u64::from_str_radix("FFFFFFF800000000", 16).unwrap();
    } else if coin == "banano" {
        threshold = u64::from_str_radix("FFFFFE0000000000", 16).unwrap();
    } else {
        panic!("Unknown coin threshold");
    }

    let mut threads = vec![];

    for _ in 0..cpus {
        let mut input_copy = [0u8; 32];
        input_copy.clone_from_slice(input_hash);
        let threshold_copy = threshold.clone();
        let terminator = terminated.clone();
        let thread_handle = thread::spawn(move || {
            let (success, work) = compute_work(terminator, &input_copy, threshold_copy);
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
    let hash = &mut work_n_hash[8..];
    let mut diff = [0u8; 32];
    for i in 0..32 {
        hash[i] = input_hash[i];
    }
    let work = &mut work_n_hash[0..8];
    rand::thread_rng().fill_bytes(work);

    loop {
        let idx = (rand::random::<u8>() % 8) as usize;
        let c = work_n_hash[idx];
        work_n_hash[idx] = if c == 0xff { 0 } else { c + 1 };
        new_diff(&work_n_hash, &mut diff);

        if slice_to_u64(&diff[0..8]) > threshold {
            println!("Found work {} > {}", slice_to_u64(&diff[0..8]), threshold);
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

pub fn slice_to_u64(full: &[u8]) -> u64 {
    let mut diff = [0u8; 8];
    diff.copy_from_slice(full);
    u64::from_le_bytes(diff)
}

fn new_diff(work_and_hash: &[u8; 40], diff: &mut [u8; 32]) {
    let mut hasher = Blake2bVar::new(32).unwrap();
    hasher.update(work_and_hash);
    hasher.finalize_variable(diff).unwrap()
}
