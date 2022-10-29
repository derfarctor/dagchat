use super::blockinfo::Block;
use serde;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
struct ProcessRequest {
    action: String,
    json_block: String,
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

    if !res.status().is_success() {
        eprintln!("Issue posting to node. Status: {}", res.status());
    }

    let response_string = res.text().unwrap();
    //eprintln!("Request:{}\n\nResponse:{}\n\n", body, response_string);
    response_string
}

pub fn publish_block(block: Block, sub: String, node_url: &str) -> String {
    let request = ProcessRequest {
        action: String::from("process"),
        json_block: String::from("true"),
        subtype: sub,
        block,
    };

    let body = serde_json::to_string(&request).unwrap();
    post_node(body, node_url)
}
