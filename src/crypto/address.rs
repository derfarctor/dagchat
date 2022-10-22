use blake2::digest::{Update, VariableOutput};
use blake2::Blake2bVar;
use data_encoding::Encoding;
use data_encoding_macro::new_encoding;

pub const ADDR_ENCODING: Encoding = new_encoding! {
    symbols: "13456789abcdefghijkmnopqrstuwxyz",
    check_trailing_bits: false,
};

pub fn get_address(pub_key_bytes: &[u8], prefix: Option<&str>) -> String {
    let mut pub_key_vec = pub_key_bytes.to_vec();
    let mut h = [0u8; 3].to_vec();
    h.append(&mut pub_key_vec);
    let checksum = ADDR_ENCODING.encode(&compute_address_checksum(pub_key_bytes));
    let address = {
        let encoded_addr = ADDR_ENCODING.encode(&h);

        let mut addr = String::from("");
        if prefix.is_some() {
            addr = String::from(prefix.unwrap());
        }
        addr.push_str(encoded_addr.get(4..).unwrap());
        addr.push_str(&checksum);
        addr
    };
    address
}

// Could be to_public_key and have return value tuple with bool
// like get_num_equivalent but this is faster since to_public_key
// is used on confirmed valid addresses frequently in lib
pub fn validate_address(addr: &str) -> bool {
    let mut encoded_addr: String;

    if !addr.contains("_") {
        return false;
    };
    let parts: Vec<&str> = addr.split("_").collect();

    // Minimum viable public representation
    if parts[1].len() < 52 {
        return false;
    };
    let checksum = String::from(parts[1].get(52..).unwrap());
    encoded_addr = String::from(parts[1].get(0..52).unwrap());
    encoded_addr.insert_str(0, "1111");

    let pub_key_vec = ADDR_ENCODING.decode(encoded_addr.as_bytes());

    // Catch decoding error - return false
    let mut pub_key_vec = match pub_key_vec {
        Ok(pub_key_vec) => pub_key_vec,
        Err(_) => return false,
    };
    pub_key_vec.drain(0..3);
    let public_key_bytes: [u8; 32] = pub_key_vec.as_slice().try_into().unwrap();
    let real_checksum_bytes = compute_address_checksum(&public_key_bytes);
    let real_checksum = ADDR_ENCODING.encode(&real_checksum_bytes);

    //println!("Told: {} Real: {}", checksum, real_checksum);
    if checksum == real_checksum {
        return true;
    } else {
        return false;
    }
}

fn compute_address_checksum(pub_key_bytes: &[u8]) -> [u8; 5] {
    let mut hasher = Blake2bVar::new(5).unwrap();
    let mut buf = [0u8; 5];
    hasher.update(pub_key_bytes);
    hasher.finalize_variable(&mut buf).unwrap();
    buf.reverse();
    buf
}
