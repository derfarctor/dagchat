use super::wordlist::WORD_LIST;
use bitreader::BitReader;
use sha2::{Digest, Sha256};

pub fn validate_mnemonic(mnemonic: &str) -> Option<[u8; 32]> {
    let (mnemonic, valid) = get_num_equivalent(mnemonic);
    if !valid {
        return None;
    }
    let mut bits = [false; 24 * 11];
    for i in 0..24 {
        for j in 0..11 {
            bits[i * 11 + j] = mnemonic[i] >> (10 - j) & 1 == 1;
        }
    }

    let mut entropy = [0u8; 32];
    for i in 0..32 {
        for j in 0..8 {
            if bits[i * 8 + j] {
                entropy[i] += 1 << (7 - j);
            }
        }
    }

    let mut hasher = Sha256::new();
    Digest::update(&mut hasher, &entropy);
    let check = hasher.finalize();
    for i in 0..8 {
        if bits[8 * 32 + i] != ((check[i / 8] & (1 << (7 - (i % 8)))) > 0) {
            return None;
        }
    }

    Some(entropy)
}

pub fn seed_to_mnemonic(seed_bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    Digest::update(&mut hasher, seed_bytes);
    let check = hasher.finalize();

    let mut combined = Vec::from(seed_bytes);
    combined.extend(&check);

    let mut reader = BitReader::new(&combined);

    let mut words: Vec<&str> = Vec::new();
    for _ in 0..24 {
        let n = reader.read_u16(11);
        words.push(WORD_LIST[n.unwrap() as usize].as_ref());
    }
    words.join(" ")
}

pub fn wordlist_position(word: &str) -> u16 {
    let index = WORD_LIST.iter().position(|&w| w == word).unwrap();
    index as u16
}

pub fn get_num_equivalent(mnemonic: &str) -> ([u16; 24], bool) {
    let mut num_mnemonic: [u16; 24] = [0u16; 24];
    let words: Vec<&str> = mnemonic.split(' ').collect();
    for i in 0..24 {
        if WORD_LIST.contains(&words[i]) {
            num_mnemonic[i] = wordlist_position(words[i]);
        } else {
            return (num_mnemonic, false);
        }
    }
    (num_mnemonic, true)
}
