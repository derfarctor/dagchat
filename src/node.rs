use serde_json::{json, Value};
use blake2::digest::{Update, VariableOutput};
use blake2::Blake2bVar;
use crate::util::{get_address};
use crate::pow::{generate_work};
pub fn change_rep(/*hash: &[u8; 32],*/ priv_k: [u8; 32], rep: &[u8; 32], previous: &[u8; 32], balance: u128, node_url: String) {

        
        //let hash = hex::encode(hash);

        let block = get_block(priv_k, rep, previous, balance);

        let body_json = json!({
                "action": "process",
                "json_block": "true",
                "do_work": true,
                "subtype": "change",
                "block": block
            });

        let body = body_json.to_string();

        println!("{}", body);

        
        let client = reqwest::blocking::Client::new();
        let res = client.post(node_url)
        .header("Content-Type", "application/json")
        .header("Accept", "application/json")
        .body(body)
        .send().unwrap();

        let text = res.text().unwrap();
        println!("{}", text);
        
}

pub fn get_block(priv_k: [u8; 32], rep: &[u8; 32], previous: &[u8; 32], balance: u128) -> Value {
        let secret = ed25519_dalek::SecretKey::from_bytes(&priv_k).unwrap();
        let public = ed25519_dalek::PublicKey::from(&secret);
        let expanded_secret = ed25519_dalek::ExpandedSecretKey::from(&secret);

        let mut hasher = Blake2bVar::new(32).unwrap();
        let mut buf = [0u8; 32];

        let link = [0u8; 32];
        hasher.update(&hex::decode("0000000000000000000000000000000000000000000000000000000000000006").unwrap());
        hasher.update(public.as_bytes());
        hasher.update(previous);
        hasher.update(rep);
        
        let mut x = format!("{:x}", &balance);
        //println!("Non padded: {}", x);
        while x.len() < 32 {
                x = format!("0{}", x);
        }
        //println!("{}", x);
        //println!("{}", x.len());
        //println!("{:?}", hex::decode(&x).unwrap());
        hasher.update(&hex::decode(x).unwrap());
        hasher.update(&link);
        hasher.finalize_variable(&mut buf).unwrap();
        println!("{}", hex::encode(buf));
        let internal_signed = expanded_secret.sign(&buf[..], &ed25519_dalek::PublicKey::from(&secret));
        //println!("{:?}", internal_signed);
        let y = internal_signed.to_bytes();
        let z = hex::encode(&y);
        //println!("{}", z);


        //let work = generate_work(&previous, "banano");
        let block = json!({
                "type": "state",
                "account": get_address(public.as_bytes()),
                "previous": hex::encode(previous),
                "representative": get_address(rep),
                "balance": balance.to_string(),
                "link": hex::encode(link),
                "signature": z
            });

        block
}