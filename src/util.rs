use blake2::digest::{Update, VariableOutput};
use blake2::Blake2bVar;
use data_encoding::Encoding;
use data_encoding_macro::new_encoding;

const ADDR_ENCODING: Encoding = new_encoding! {
    symbols: "13456789abcdefghijkmnopqrstuwxyz",
    check_trailing_bits: false,
};

const PREFIX: &str = "nano_";

pub fn get_address(pub_key_bytes: &[u8]) -> String {
    let mut pub_key_vec = pub_key_bytes.to_vec();
    let mut h = [0u8; 3].to_vec();
    h.append(&mut pub_key_vec);
    let checksum = ADDR_ENCODING.encode(&compute_address_checksum(pub_key_bytes));
    let address = {
        let encoded_addr = ADDR_ENCODING.encode(&h);
        let mut addr = String::from(PREFIX);
        addr.push_str(encoded_addr.get(4..).unwrap());
        addr.push_str(&checksum);
        addr
    };
    address
}

pub fn to_public_key(addr: String) -> Vec<u8> {
    let mut encoded_addr = String::from(addr.get(5..57).unwrap());
    encoded_addr.insert_str(0, "1111");
    let mut pub_key_vec = ADDR_ENCODING.decode(encoded_addr.as_bytes()).unwrap();
    pub_key_vec.drain(0.. 3);
    pub_key_vec
}

fn compute_address_checksum(pub_key_bytes: &[u8]) -> [u8; 5] {
    let mut hasher = Blake2bVar::new(5).unwrap();
    let mut buf = [0u8; 5];
    hasher.update(pub_key_bytes);
    hasher.finalize_variable(&mut buf).unwrap();
    buf.reverse();
    buf
}
