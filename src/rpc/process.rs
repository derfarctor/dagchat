use super::blockinfo::Block;
use crate::app::components::settings::structs::*;
use crate::app::constants::REQ_TIMEOUT;
use serde;
use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Serialize, Deserialize, Debug)]
struct ProcessRequest {
    action: String,
    json_block: String,
    subtype: String,
    block: Block,
}

#[derive(Serialize, Deserialize, Debug)]
struct BoomPowProcessRequest {
    action: String,
    json_block: String,
    subtype: String,
    do_work: bool,
    block: Block,
}

#[derive(Serialize, Deserialize, Debug)]
struct ProcessResponse {
    hash: String,
}

pub fn post_node(body: String, node_url: &str, timeout: u64) -> Result<String, String> {
    let client = reqwest::blocking::Client::new();
    let res = client
        .post(node_url)
        .timeout(Duration::from_secs(timeout))
        .header("Content-Type", "application/json")
        .header("Accept", "application/json")
        .body(body)
        .send();

    //let x = res.unwrap().text().unwrap();
    //eprintln!("Request:{}\n\nResponse:{}\n\n", body, x);
    //return Ok(x);
    if let Ok(res) = res {
        if !res.status().is_success() {
            //eprintln!("Issue posting to node. Status: {}", res.status());
            return Err(res.status().to_string());
        }
        Ok(res.text().unwrap())
    } else {
        Err(res.err().unwrap().to_string())
    }
}

pub fn publish_block(block: Block, sub: String, network: &Network) -> Result<String, String> {
    if network.work_type == WorkType::CPU || network.work_type == WorkType::WORK_SERVER {
        let body = serde_json::to_string(&ProcessRequest {
            action: String::from("process"),
            json_block: String::from("true"),
            subtype: sub,
            block,
        })
        .unwrap();
        post_node(body, &network.node_url, REQ_TIMEOUT)
    } else {
        let body = serde_json::to_string(&BoomPowProcessRequest {
            action: String::from("process"),
            json_block: String::from("true"),
            subtype: sub,
            do_work: true,
            block,
        })
        .unwrap();
        post_node(body, &network.appditto_node_url, REQ_TIMEOUT)
    }
}
