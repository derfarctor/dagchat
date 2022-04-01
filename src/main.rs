use std::str;
use rand::thread_rng;
pub mod util;
fn main() {
        let mut csprng = thread_rng();
        let (bob_secret, bob_public) = ecies_ed25519::generate_keypair(&mut csprng);

        let mut message = String::from("To be, or not to be, that is the question:
        Whether 'tis nobler in the mind to suffer
        The slings and arrows of outrageous fortune,
        Or to take arms against a sea of troubles
        And by opposing end them. To die—to sleep,
        No more; and by a sleep to say we end
        The heart-ache and the thousand natural shocks
        That flesh is heir to: 'tis a consummation
        Devoutly to be wish'd. To die, to sleep;
        To sleep, perchance to dream—ay, there's the rub:
        For in that sleep of death what dreams may come,
        When we have shuffled off this mortal coil,
        Must give us pause—");

        // 32 byte pub key + 12 byte nonce + 16 byte gcm tag = 60 byte overhead
        // total bytes must be multiple of 32 for conversion to addresses so pad
        let pad = (message.len()+28) % 32;
        for _ in 0..(32-pad) {
                message.push_str(" ");
        }

        let alice_encrypted_bytes = ecies_ed25519::encrypt(&bob_public, message.as_bytes(), &mut csprng).unwrap();

        let blocks = (60 + message.len()) / 32;
        println!("Blocks needed: {}", blocks);

        let mut addresses = vec![];
        for block in 0..blocks {   
                let start = 32 * block; 
                let end = 32 * (block + 1);
                let block_data = &alice_encrypted_bytes[start..end];
                let addr = util::get_address(block_data);
                println!("{}", addr);
                addresses.push(addr);
        }

        let mut bob_encrypted_bytes = vec![];
        for addr in addresses {
                let block_data = util::to_public_key(addr);
                bob_encrypted_bytes.extend(&block_data);
        }

        // Decrypt the message with the secret key
        let decrypted = ecies_ed25519::decrypt(&bob_secret, &bob_encrypted_bytes[..]).unwrap();
        println!("{:?}", str::from_utf8(&decrypted).unwrap().trim());
}
