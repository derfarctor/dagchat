use super::address::ADDR_ENCODING;
use blake2::digest::{Update, VariableOutput};
use blake2::Blake2bVar;

pub fn get_private_key(seed_bytes: &[u8; 32], idx: u32) -> [u8; 32] {
    let mut hasher = Blake2bVar::new(32).unwrap();
    let mut buf = [0u8; 32];
    hasher.update(seed_bytes);
    hasher.update(&idx.to_be_bytes());
    hasher.finalize_variable(&mut buf).unwrap();
    buf
}

pub fn to_public_key(addr: &str) -> [u8; 32] {
    let mut encoded_addr: String;
    let parts: Vec<&str> = addr.split("_").collect();
    encoded_addr = String::from(parts[1].get(0..52).unwrap());
    encoded_addr.insert_str(0, "1111");
    let mut pub_key_vec = ADDR_ENCODING.decode(encoded_addr.as_bytes()).unwrap();
    pub_key_vec.drain(0..3);
    pub_key_vec.as_slice().try_into().unwrap()
}
