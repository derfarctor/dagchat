use super::blockinfo::Block;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
struct ProcessRequest {
    action: String,
    json_block: String,
    do_work: bool,
    subtype: String,
    block: Block,
}

#[derive(Serialize, Deserialize, Debug)]
struct ProcessResponse {
    hash: String,
}

pub fn post_node(body: String, node_url: &str) -> String {
    let client = reqwest::blocking::Client::new();
    let res = client
        .post(node_url)
        .header("Content-Type", "application/json")
        .header("Accept", "application/json")
        .body(body)
        .send()
        .unwrap();

    if res.status().is_success() {
        //eprintln!("Successfully communicated with node");
        let response_str = res.text().unwrap();
        return response_str;
    } else {
        //eprintln!("Issue. Status: {}", res.status());
    }
    String::from("Failed")
}

pub fn publish_block(block: Block, sub: String, node_url: &str) -> String {
    let request = ProcessRequest {
        action: String::from("process"),
        json_block: String::from("true"),
        do_work: true,
        subtype: sub,
        block,
    };

    let body = serde_json::to_string(&request).unwrap();
    post_node(body, node_url)
}
