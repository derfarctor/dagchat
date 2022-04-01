use rand::thread_rng;
pub mod util;

fn main() {
    let mut csprng = rand::thread_rng();
    let (secret, public) = ecies_ed25519::generate_keypair(&mut csprng);

    let mut message = String::from("ntnneanorsietnh");
    //32 + 12 IV + 16 BLOCK
    let spair = (message.len()-4) % 32;
    println!("{}", spair);
    for i in 0..(32-spair) {
        message.push_str(" ");
    }

    // Encrypt the message with the public key such that only the holder of the secret key can decrypt.
    let encrypted = ecies_ed25519::encrypt(&public, message.as_bytes(), &mut csprng).unwrap();
    
    println!("{:?}", encrypted);
    println!("Length: {}", encrypted.len());
    let addr = util::get_address(&encrypted[..32]);

    // Decrypt the message with the secret key
    let decrypted = ecies_ed25519::decrypt(&secret, &encrypted);
}
