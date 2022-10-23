use crate::app::constants::crypto::{IV_LENGTH, SALT_LENGTH};
use aes_gcm::aead::{generic_array::GenericArray, Aead, NewAead};
use aes_gcm::{Aes256Gcm, Nonce};
use argon2::{self, Config};
use rand::RngCore;
use std::str;

pub fn derive_key(password: &str, salt: &[u8]) -> Vec<u8> {
    let config = Config::default();
    let hash = argon2::hash_raw(password.as_bytes(), salt, &config).unwrap();
    hash
}

pub fn decrypt_bytes(encrypted_bytes: &[u8], password: &str) -> Result<Vec<u8>, String> {
    let salt = &encrypted_bytes[..SALT_LENGTH];

    let key_bytes = derive_key(password, salt);
    let key = GenericArray::from_slice(&key_bytes);

    let aead = Aes256Gcm::new(key);
    let nonce = Nonce::from_slice(&encrypted_bytes[SALT_LENGTH..SALT_LENGTH + IV_LENGTH]);
    let encrypted = &encrypted_bytes[SALT_LENGTH + IV_LENGTH..];
    let decrypted = aead.decrypt(nonce, encrypted);
    match decrypted {
        Ok(decrypted) => Ok(decrypted),
        Err(e) => Err(format!("Failed to decrypt bytes. Error: {}", e)),
    }
}

pub fn encrypt_bytes(bytes: &[u8], password: &str) -> Vec<u8> {
    let mut csprng = rand::thread_rng();
    let mut salt = [0u8; SALT_LENGTH];
    csprng.fill_bytes(&mut salt);
    let key_bytes = derive_key(password, &salt);
    let key = GenericArray::from_slice(&key_bytes);

    let aead = Aes256Gcm::new(key);

    let mut nonce_bytes = [0u8; IV_LENGTH];
    csprng.fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);

    let ciphertext = aead.encrypt(nonce, bytes).unwrap();
    let mut encrypted_bytes = Vec::with_capacity(SALT_LENGTH + IV_LENGTH + ciphertext.len());
    encrypted_bytes.extend(salt);
    encrypted_bytes.extend(nonce);
    encrypted_bytes.extend(ciphertext);
    encrypted_bytes
}
